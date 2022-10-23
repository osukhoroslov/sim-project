//! Virtual machine placement algorithms.

use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::core::common::Allocation;
use crate::core::common::AllocationVerdict;
use crate::core::monitoring::Monitoring;
use crate::core::resource_pool::ResourcePoolState;

/// Trait for implementation of VM placement algorithms.
///
/// The algorithm is defined as a function of VM allocation request and current resource pool state, which returns an
/// ID of host selected for VM placement or `None` if there is not suitable host.
///
/// The reference to monitoring service is also passed to the algorithm so that it can use the information about
/// current host load.
///
/// It is possible to implement arbitrary placement algorithm and use it in scheduler.
pub trait VMPlacementAlgorithm {
    /// parse load model arguments from .yaml config string
    fn parse_config_args(&mut self, _config_string: String) {}

    fn select_host(&self, alloc: &Allocation, pool_state: &ResourcePoolState, monitoring: &Monitoring) -> Option<u32>;
}

#[derive(Clone, Debug, PartialEq, EnumString, Serialize, Deserialize)]
pub enum VmPlacementAlgorithmType {
    FirstFit,
    BestFit,
    WorstFit,
    BestFitThreshold,
}

pub fn parse_placement_algorithm(raw_data: String) -> Box<dyn VMPlacementAlgorithm> {
    let cleanup = raw_data.replace("]", "").replace("\"", "");
    let split = cleanup.split("[").collect::<Vec<&str>>();
    let model_type: VmPlacementAlgorithmType = split.get(0).unwrap().parse().unwrap();
    let model_args = split.get(1).unwrap().to_string();

    match model_type {
        VmPlacementAlgorithmType::FirstFit => {
            let mut result = FirstFit::new();
            result.parse_config_args(model_args);
            Box::new(result)
        }
        VmPlacementAlgorithmType::BestFit => {
            let mut result = BestFit::new();
            result.parse_config_args(model_args);
            Box::new(result)
        }
        VmPlacementAlgorithmType::WorstFit => {
            let mut result = WorstFit::new();
            result.parse_config_args(model_args);
            Box::new(result)
        }
        VmPlacementAlgorithmType::BestFitThreshold => {
            let mut result = BestFitThreshold::new_fwd();
            result.parse_config_args(model_args);
            Box::new(result)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// FirstFit algorithm, which returns the first suitable host.
pub struct FirstFit;

impl FirstFit {
    pub fn new() -> Self {
        Self {}
    }
}

impl VMPlacementAlgorithm for FirstFit {
    fn select_host(&self, alloc: &Allocation, pool_state: &ResourcePoolState, _monitoring: &Monitoring) -> Option<u32> {
        for host in pool_state.get_hosts_list() {
            if pool_state.can_allocate(&alloc, host) == AllocationVerdict::Success {
                return Some(host);
            }
        }
        return None;
    }
}

////////////////////////////////////////////////////////////////////////////////

/// BestFit algorithm, which returns the most loaded (by CPU) suitable host.
pub struct BestFit;

impl BestFit {
    pub fn new() -> Self {
        Self {}
    }
}

impl VMPlacementAlgorithm for BestFit {
    fn select_host(&self, alloc: &Allocation, pool_state: &ResourcePoolState, _monitoring: &Monitoring) -> Option<u32> {
        let mut result: Option<u32> = None;
        let mut min_available_cpu: u32 = u32::MAX;

        for host in pool_state.get_hosts_list() {
            if pool_state.can_allocate(&alloc, host) == AllocationVerdict::Success {
                if pool_state.get_available_cpu(host) < min_available_cpu {
                    min_available_cpu = pool_state.get_available_cpu(host);
                    result = Some(host);
                }
            }
        }
        return result;
    }
}

////////////////////////////////////////////////////////////////////////////////

/// WorstFit algorithm, which returns the least loaded (by CPU) suitable host.
pub struct WorstFit;

impl WorstFit {
    pub fn new() -> Self {
        Self {}
    }
}

impl VMPlacementAlgorithm for WorstFit {
    fn select_host(&self, alloc: &Allocation, pool_state: &ResourcePoolState, _monitoring: &Monitoring) -> Option<u32> {
        let mut result: Option<u32> = None;
        let mut max_available_cpu: u32 = 0;

        for host in pool_state.get_hosts_list() {
            if pool_state.can_allocate(&alloc, host) == AllocationVerdict::Success {
                if pool_state.get_available_cpu(host) > max_available_cpu {
                    max_available_cpu = pool_state.get_available_cpu(host);
                    result = Some(host);
                }
            }
        }
        return result;
    }
}

////////////////////////////////////////////////////////////////////////////////

/// BestFit algorithm, which returns the most loaded (by actual CPU load) suitable host.
pub struct BestFitThreshold {
    threshold: f64,
}

impl BestFitThreshold {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn new_fwd() -> Self {
        Self { threshold: 0. }
    }
}

impl VMPlacementAlgorithm for BestFitThreshold {
    fn parse_config_args(&mut self, config_str: String) {
        let variables = config_str.split(",");
        for variable in variables {
            let split = variable.split("=").collect::<Vec<&str>>();
            if split.get(0).unwrap().to_string() == "threshold" {
                self.threshold = split.get(1).unwrap().to_string().parse::<f64>().unwrap();
            }
        }
    }

    fn select_host(&self, alloc: &Allocation, _pool_state: &ResourcePoolState, monitoring: &Monitoring) -> Option<u32> {
        let mut result: Option<u32> = None;
        let mut best_cpu_load: f64 = 0.;
        for host in monitoring.get_hosts_list() {
            let state = monitoring.get_host_state(*host);
            let cpu_used = state.cpu_load * state.cpu_total as f64;
            let memory_used = state.memory_load * state.memory_total as f64;

            let cpu_load_new = (cpu_used + alloc.cpu_usage as f64) / state.cpu_total as f64;
            let memory_load_new = (memory_used + alloc.memory_usage as f64) / state.memory_total as f64;

            if best_cpu_load < cpu_load_new {
                if cpu_load_new < self.threshold && memory_load_new < self.threshold {
                    best_cpu_load = cpu_load_new;
                    result = Some(*host);
                }
            }
        }
        return result;
    }
}
