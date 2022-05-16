use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use serde::Serialize;
use sugars::{rc, refcell};

use crate::core::common::VmStatus;
use crate::core::config::SimulationConfig;
use crate::core::events::allocation::MigrationRequest;
use crate::core::monitoring::HostState;
use crate::core::monitoring::Monitoring;
use crate::core::resource_pool::Allocation;
use crate::core::vm::VirtualMachine;
use crate::custom_component::CustomComponent;
use simcore::cast;
use simcore::context::SimulationContext;
use simcore::event::Event;
use simcore::handler::EventHandler;
use simcore::log_info;
use simcore::log_trace;
use simcore::log_warn;

#[derive(Serialize)]
pub struct PerformMigrations {}

pub struct VmMigrator {
    interval: f64,

    #[allow(dead_code)]
    overload_threshold: f64,

    underload_threshold: f64,
    monitoring: Option<Rc<RefCell<Monitoring>>>,
    allocations: Rc<RefCell<HashMap<u32, Allocation>>>,
    vms: Rc<RefCell<BTreeMap<u32, VirtualMachine>>>,
    sim_config: Option<Rc<SimulationConfig>>,
    ctx: SimulationContext,
}

impl VmMigrator {
    pub fn patch_custom_args(
        &mut self,
        interval: f64,
        monitoring: Rc<RefCell<Monitoring>>,
        allocations: Rc<RefCell<HashMap<u32, Allocation>>>,
        vms: Rc<RefCell<BTreeMap<u32, VirtualMachine>>>,
        sim_config: Rc<SimulationConfig>,
    ) {
        self.interval = interval;
        self.monitoring = Some(monitoring);
        self.allocations = allocations;
        self.vms = vms;
        self.sim_config = Some(sim_config);
    }

    fn schedule_migration(&mut self, vm_id: u32, source_host: u32, target_host: u32) {
        let mut vms = self.vms.borrow_mut();
        let mut vm = vms.get_mut(&vm_id).unwrap();
        vm.lifetime -= self.ctx.time() - vm.start_time;
        vm.start_time = self.ctx.time() + vm.start_duration() + self.sim_config.as_ref().unwrap().message_delay;

        log_info!(
            self.ctx,
            "schedule migration of vm {} from host {} to host {}",
            vm_id,
            source_host,
            target_host
        );

        self.ctx.emit(
            MigrationRequest {
                source_host: source_host.clone(),
                alloc: self.allocations.borrow().get(&vm_id).unwrap().clone(),
                vm: vm.clone(),
            },
            target_host,
            self.sim_config.as_ref().unwrap().message_delay,
        );
    }

    fn perform_migrations(&mut self) {
        if self.monitoring.is_none() {
            log_warn!(self.ctx, "cannot perform migrations as there`s no monitoring");
            self.ctx.emit_self(PerformMigrations {}, self.interval);
            return;
        }

        log_trace!(self.ctx, "perform migrations");
        let mon_rc = self.monitoring.clone().unwrap();
        let mon = mon_rc.borrow_mut();
        let mut host_states = mon.get_host_states().clone();
        let allocations_rc = self.allocations.clone();
        let allocations = allocations_rc.borrow();

        // select underloaded VMs to migrate ===================================

        let mut vms_to_migrate = Vec::<u32>::new();
        let mut min_load: f64 = 1.;
        for (&host, state) in host_states.iter_mut() {
            if state.cpu_load == 0. {
                // host turned off
                continue;
            }
            if state.cpu_load < self.underload_threshold || state.memory_load < self.underload_threshold {
                min_load = min_load.min(state.cpu_load).min(state.memory_load);
                for vm_id in mon.get_host_vms(host) {
                    let vm_status = mon.vm_status(vm_id);
                    if vm_status != VmStatus::Running {
                        continue;
                    }
                    vms_to_migrate.push(vm_id);
                }
            }

            if state.cpu_load > self.overload_threshold || state.memory_load > self.overload_threshold {
                let mut vms = mon.get_host_vms(host);
                while state.cpu_load > self.overload_threshold || state.memory_load > self.overload_threshold {
                    let vm_id = *vms.iter().next().unwrap();
                    vms.remove(&vm_id);
                    let vm_status = mon.vm_status(vm_id);
                    if vm_status != VmStatus::Running {
                        continue;
                    }
                    vms_to_migrate.push(vm_id);

                    let cpu_usage = state.cpu_load * (state.cpu_total as f64);
                    let memory_usage = state.memory_load * (state.memory_total as f64);
                    let cpu_load_new = (cpu_usage - (allocations[&vm_id].cpu_usage as f64)) / (state.cpu_total as f64);
                    let memory_load_new =
                        (memory_usage - (allocations[&vm_id].memory_usage as f64)) / (state.memory_total as f64);

                    *state = HostState {
                        cpu_load: cpu_load_new,
                        memory_load: memory_load_new,
                        cpu_total: state.cpu_total,
                        memory_total: state.memory_total,
                    };
                }
            }
        }

        // build migration schema using Best Fit ===============================

        // target hosts, cannot migrate from them as some VM(s) are migrating and will increase their load rate
        let mut target_hosts = HashSet::<u32>::new();

        for vm_id in vms_to_migrate {
            let current_host = mon.find_host_by_vm(vm_id);
            if target_hosts.contains(&current_host) {
                continue;
            }

            let mut best_host_opt: Option<u32> = None;
            let mut best_cpu_load = 0.;

            for (&host, state) in host_states.iter() {
                if host == current_host {
                    continue;
                }
                if min_load < 1. && (state.cpu_load < min_load && state.memory_load < min_load) {
                    continue;
                }

                let cpu_usage = state.cpu_load * (state.cpu_total as f64);
                let memory_usage = state.memory_load * (state.memory_total as f64);
                let cpu_load_new = (cpu_usage + (allocations[&vm_id].cpu_usage as f64)) / (state.cpu_total as f64);
                let memory_load_new =
                    (memory_usage + (allocations[&vm_id].memory_usage as f64)) / (state.memory_total as f64);
                if cpu_load_new < self.overload_threshold && memory_load_new < self.overload_threshold {
                    if cpu_load_new > best_cpu_load {
                        best_cpu_load = cpu_load_new;
                        best_host_opt = Some(host);
                    }
                }
            }

            if let Some(best_host) = best_host_opt {
                target_hosts.insert(best_host);
                self.schedule_migration(vm_id, current_host, best_host);
            }
        }

        // schedule new migration attempt ======================================
        self.ctx.emit_self(PerformMigrations {}, self.interval);
    }
}

impl CustomComponent for VmMigrator {
    fn new(ctx: SimulationContext) -> Self {
        Self {
            interval: 1.,
            overload_threshold: 0.8,
            underload_threshold: 0.4,
            monitoring: None,
            allocations: rc!(refcell!(HashMap::new())),
            vms: rc!(refcell!(BTreeMap::new())),
            sim_config: None,
            ctx,
        }
    }

    fn init(&mut self) {
        self.ctx.emit_self(PerformMigrations {}, 0.);
    }
}

impl EventHandler for VmMigrator {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            PerformMigrations {} => {
                self.perform_migrations();
            }
        })
    }
}
