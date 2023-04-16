//! CPU power model trait.

use dyn_clone::{clone_trait_object, DynClone};

/// A model for estimating the power consumption of CPU based on its utilization.
/// Can also be used to model the power consumption of a whole host based on CPU utilization.
pub trait CpuPowerModel: DynClone {
    /// Returns CPU power consumption in W.
    ///
    /// CPU utilization should be passed as a float in 0.0-1.0 range.
    fn get_power(&self, utilization: f64) -> f64;

    /// Returns CPU power consumption in W.
    /// In addition, CPU frequency is passed
    ///
    /// CPU utilization should be passed as a float in 0.0-1.0 range.
    /// CPU frequency should be passed as a float in 0.0-1.0 range.
    fn get_power_with_freq(&self, utilization: f64, _frequency: f64) -> f64 {
        self.get_power(utilization)
    }
}

clone_trait_object!(CpuPowerModel);
