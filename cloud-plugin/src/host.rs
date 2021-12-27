use log::info;
use std::collections::HashMap;

use core::actor::{Actor, ActorContext, ActorId, Event};
use core::cast;

use crate::monitoring::HostStateUpdate;
use crate::network::MESSAGE_DELAY;
use crate::scheduler::VMAllocationFailed;
use crate::virtual_machine::VMInit;
use crate::virtual_machine::VirtualMachine;

#[derive(PartialEq)]
enum AllocationVerdict {
    NotEnoughCPU,
    NotEnoughRAM,
    Success,
}

pub static STATS_SEND_PERIOD: f64 = 0.5;

// ACTORS //////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct EnergyManager {
    energy_consumed: f64,
    prev_milestone: f64,
    current_load: f64,
}

#[derive(Debug, Clone)]
pub struct HostManager {
    pub id: String,

    cpu_total: u32,
    cpu_available: u32,
    
    #[allow(dead_code)]
    ram_total: u32,
    ram_available: u32,

    vms: HashMap<String, VirtualMachine>,
    energy_manager: EnergyManager,
    monitoring: ActorId,
}

impl EnergyManager {
    pub fn new() -> Self {
        Self {
            prev_milestone: 0.0,
            energy_consumed: 0.0,
            current_load: 0.0,
        }
    }

    pub fn update_energy(&mut self, time: f64, new_load: f64) {
        self.energy_consumed += (time - self.prev_milestone) * self.current_load;
        self.current_load = new_load;
        self.prev_milestone = time;
    }

    pub fn get_total_consumed(&self) -> f64 {
        return self.energy_consumed;
    }
}

impl HostManager {
    pub fn new(cpu_total: u32, ram_total: u32, id: String, monitoring: ActorId) -> Self {
        Self {
            id,
            cpu_total,
            ram_total,
            cpu_available: cpu_total,
            ram_available: ram_total,
            vms: HashMap::new(),
            energy_manager: EnergyManager::new(),
            monitoring: monitoring.clone(),
        }
    }

    fn can_allocate(&self, vm: &VirtualMachine) -> AllocationVerdict {
        if self.cpu_available < vm.cpu_usage {
            return AllocationVerdict::NotEnoughCPU;
        }
        if self.ram_available < vm.ram_usage {
            return AllocationVerdict::NotEnoughRAM;
        }
        return AllocationVerdict::Success;
    }

    fn get_energy_load(&self) -> f64 {
        let cpu_used = (self.cpu_total - self.cpu_available) as f64;
        if cpu_used == 0. {
            return 0.;
        }
        return 0.4 + 0.6 * cpu_used / (self.cpu_total as f64);
    }

    fn place_vm(&mut self, time: f64, vm: &VirtualMachine) {
        self.energy_manager.update_energy(time, self.get_energy_load());

        self.cpu_available -= vm.cpu_usage;
        self.ram_available -= vm.ram_usage;
        self.vms.insert(vm.id.clone(), vm.clone());
    }

    fn remove_vm(&mut self, time: f64, vm_id: &str) {
        self.energy_manager.update_energy(time, self.get_energy_load());

        self.cpu_available += self.vms[vm_id].cpu_usage;
        self.ram_available += self.vms[vm_id].ram_usage;
        self.vms.remove(vm_id);
    }

    pub fn get_total_consumed(&mut self, time: f64) -> f64 {
        self.energy_manager.update_energy(time, self.get_energy_load());
        return self.energy_manager.get_total_consumed();
    }
}

// EVENTS //////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TryAllocateVM {
    pub vm: VirtualMachine,
}

#[derive(Debug)]
pub struct SendHostState {}

#[derive(Debug)]
pub struct ReleaseVmResources {
    pub vm_id: String,
}

impl Actor for HostManager {
    fn on(&mut self, event: Box<dyn Event>, from: ActorId, ctx: &mut ActorContext) {
        cast!(match event {
            TryAllocateVM { vm } => {
                if self.can_allocate(vm) == AllocationVerdict::Success {
                    self.place_vm(ctx.time(), vm);
                    info!("[time = {}] vm #{} allocated on host #{}", ctx.time(), vm.id, self.id);
                    ctx.emit_now(VMInit { scheduler_id: from }, vm.actor_id.clone());
                } else {
                    info!(
                        "[time = {}] not enough space for vm #{} on host #{}",
                        ctx.time(),
                        vm.id,
                        self.id
                    );
                    ctx.emit(VMAllocationFailed { vm: vm.clone() }, from.clone(), MESSAGE_DELAY);
                }
            }
            SendHostState {} => {
                info!(
                    "[time = {}] host #{} sends it`s data to monitoring",
                    ctx.time(),
                    self.id
                );
                ctx.emit(
                    HostStateUpdate {
                        host_id: ctx.id.clone(),
                        cpu_available: self.cpu_available,
                        ram_available: self.ram_available,
                    },
                    self.monitoring.clone(),
                    MESSAGE_DELAY,
                );

                ctx.emit(SendHostState {}, ctx.id.clone(), STATS_SEND_PERIOD);
            }
            ReleaseVmResources { vm_id } => {
                info!(
                    "[time = {}] release resources from vm #{} in host #{}",
                    ctx.time(),
                    vm_id,
                    self.id
                );
                self.remove_vm(ctx.time(), vm_id)
            }
        })
    }

    fn is_active(&self) -> bool {
        true
    }
}
