//! Constant CPU power model.

use crate::power::cpu::CpuPowerModel;

/// A power model using a constant power consumption value.
#[derive(Clone)]
pub struct ConstantCpuPowerModel {
    power: f64,
}

impl ConstantCpuPowerModel {
    /// Creates a constant power model.
    ///
    /// * `power` - The power consumption in Watts.
    pub fn new(power: f64) -> Self {
        Self { power }
    }
}

impl CpuPowerModel for ConstantCpuPowerModel {
    fn get_power(&self, _utilization: f64, _frequency: Option<f64>, _state: Option<usize>) -> f64 {
        self.power
    }
}
