//! Tools for launching experiments with multiple environment configurations

use std::cell::RefCell;
use std::rc::Rc;

use dyn_clone::{clone_trait_object, DynClone};
use sugars::{rc, refcell};
use threadpool::ThreadPool;

use dslab_core::Simulation;

use crate::core::config::exp_config::ExperimentConfig;
use crate::simulation::CloudSimulation;

/// Callbacks on simulation events
pub trait SimulationCallbacks: DynClone + Send {
    fn on_simulation_start(&mut self, _sim: Rc<RefCell<CloudSimulation>>) {
        // custom callback
    }

    // if returns false then the simulation is stopped
    fn on_step(&mut self, _sim: Rc<RefCell<CloudSimulation>>) -> bool {
        // custom callback
        true
    }

    fn on_simulation_finish(&mut self, _sim: Rc<RefCell<CloudSimulation>>) {
        // custom callback
    }
}

clone_trait_object!(SimulationCallbacks);

pub struct Experiment {
    pub callbacks: Box<dyn SimulationCallbacks>,
    pub config: ExperimentConfig,
}

impl Experiment {
    pub fn new(callbacks: Box<dyn SimulationCallbacks>, config: ExperimentConfig) -> Self
    where
        Self: Sized,
    {
        Self { callbacks, config }
    }

    pub fn run(&mut self) {
        let pool = ThreadPool::new(self.config.threads_count);

        while let Some(current_config) = self.config.get() {
            println!();
            println!();
            println!("==== New test case ====");
            println!("{0:?}", self.config);
            println!("=======================");
            println!();
            println!();

            let mut callbacks = self.callbacks.clone();
            pool.execute(move || {
                let sim = Simulation::new(123);
                let cloud_sim = rc!(refcell!(CloudSimulation::new(sim, current_config.clone())));
                callbacks.on_simulation_start(cloud_sim.clone());

                while cloud_sim.borrow_mut().current_time() < current_config.simulation_length {
                    cloud_sim.borrow_mut().steps(1);
                    let proceed = callbacks.on_step(cloud_sim.clone());
                    if !proceed {
                        break;
                    }
                }
                callbacks.on_simulation_finish(cloud_sim);
            });
        }
        pool.join();
    }
}
