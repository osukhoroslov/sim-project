use std::collections::{BTreeSet, HashMap};

use dslab_core::context::SimulationContext;
use dslab_core::Id;
use dslab_core::{log_error, log_info, log_warn};

use crate::dag::DAG;
use crate::data_item::{DataTransferMode, DataTransferStrategy};
use crate::runner::Config;
use crate::scheduler::{Action, Scheduler, TimeSpan};
use crate::schedulers::common::*;
use crate::system::System;
use crate::task::*;

pub struct PeftScheduler {
    data_transfer_strategy: DataTransferStrategy,
}

impl PeftScheduler {
    pub fn new() -> Self {
        PeftScheduler {
            data_transfer_strategy: DataTransferStrategy::Eager,
        }
    }

    pub fn with_data_transfer_strategy(mut self, data_transfer_strategy: DataTransferStrategy) -> Self {
        self.data_transfer_strategy = data_transfer_strategy;
        self
    }
}

impl Scheduler for PeftScheduler {
    fn start(&mut self, dag: &DAG, system: System, config: Config, ctx: &SimulationContext) -> Vec<Action> {
        assert_ne!(
            config.data_transfer_mode,
            DataTransferMode::Manual,
            "PeftScheduler doesn't support DataTransferMode::Manual"
        );

        if dag.get_tasks().iter().any(|task| task.min_cores != task.max_cores) {
            log_warn!(
                ctx,
                "some tasks support different number of cores, but HEFT will always use min_cores"
            );
        }

        let resources = system.resources;
        let network = system.network;

        let task_count = dag.get_tasks().len();

        // optimistic cost table
        let mut oct = vec![vec![0.0; resources.len()]; task_count];
        for task_id in topsort(dag).into_iter().rev() {
            for resource_id in 0..resources.len() {
                oct[task_id][resource_id] = task_successors(task_id, dag)
                    .into_iter()
                    .map(|(succ, weight)| {
                        (0..resources.len())
                            .map(|succ_resource| {
                                oct[succ][succ_resource]
                                    + dag.get_task(succ).flops as f64 / resources[succ_resource].speed as f64
                                    + weight as f64
                                        * config.data_transfer_mode.net_time(
                                            network,
                                            resources[resource_id].id,
                                            resources[succ_resource].id,
                                            ctx.id(),
                                        )
                            })
                            .min_by(|a, b| a.total_cmp(&b))
                            .unwrap_or(0.)
                    })
                    .max_by(|a, b| a.total_cmp(&b))
                    .unwrap_or(0.);
            }
        }
        let task_ranks = oct
            .iter()
            .map(|ar| ar.iter().sum::<f64>() / ar.len() as f64)
            .collect::<Vec<_>>();

        let mut task_ids = (0..task_count).collect::<Vec<_>>();
        task_ids.sort_by(|&a, &b| task_ranks[b].total_cmp(&task_ranks[a]));

        let mut scheduled_tasks = resources
            .iter()
            .map(|resource| {
                (0..resource.cores_available)
                    .map(|_| BTreeSet::<ScheduledTask>::new())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut task_finish_times = vec![0.; task_count];
        let mut scheduled = vec![false; task_count];

        let mut data_locations: HashMap<usize, Id> = HashMap::new();
        let mut task_locations: HashMap<usize, Id> = HashMap::new();

        let mut result: Vec<(f64, Action)> = Vec::new();

        for _ in 0..task_ids.len() {
            // first ready task in task_ids, which is already sorted by ranks
            let task_id = *task_ids
                .iter()
                .filter(|&task| !scheduled[*task])
                .filter(|&task| {
                    dag.get_task(*task)
                        .inputs
                        .iter()
                        .filter_map(|&id| dag.get_data_item(id).producer)
                        .all(|task| scheduled[task])
                })
                .next()
                .unwrap();
            let mut best_finish = -1.;
            let mut best_start = -1.;
            let mut best_resource = 0 as usize;
            let mut best_cores: Vec<u32> = Vec::new();
            for resource in 0..resources.len() {
                let res = evaluate_assignment(
                    task_id,
                    resource,
                    &task_finish_times,
                    &scheduled_tasks,
                    &data_locations,
                    &task_locations,
                    &self.data_transfer_strategy,
                    dag,
                    resources,
                    network,
                    &config,
                    ctx,
                );
                if res.is_none() {
                    continue;
                }
                let (start_time, finish_time, cores) = res.unwrap();

                if best_finish == -1. || best_finish > finish_time {
                    best_start = start_time;
                    best_finish = finish_time;
                    best_resource = resource;
                    best_cores = cores;
                }
            }

            if best_finish == -1. {
                log_error!(
                    ctx,
                    "couldn't schedule task {}, since every resource has less cores than minimum requirement for this task",
                    dag.get_task(task_id ).name
                );
                return Vec::new();
            }

            for &core in best_cores.iter() {
                scheduled_tasks[best_resource][core as usize].insert(ScheduledTask::new(
                    best_start,
                    best_finish,
                    task_id,
                ));
            }
            task_finish_times[task_id] = best_finish;
            result.push((
                best_start,
                Action::ScheduleTaskOnCores {
                    task: task_id,
                    resource: best_resource,
                    cores: best_cores,
                    expected_span: Some(TimeSpan::new(best_start, best_finish)),
                },
            ));
            for &output in dag.get_task(task_id).outputs.iter() {
                data_locations.insert(output, resources[best_resource].id);
            }
            task_locations.insert(task_id, resources[best_resource].id);
            scheduled[task_id] = true;
        }

        log_info!(
            ctx,
            "expected makespan: {:.3}",
            calc_makespan(&scheduled_tasks, dag, resources, network, ctx)
        );

        result.sort_by(|a, b| a.0.total_cmp(&b.0));
        result.into_iter().map(|(_, b)| b).collect()
    }

    fn on_task_state_changed(
        &mut self,
        _task: usize,
        _task_state: TaskState,
        _dag: &DAG,
        _system: System,
        _ctx: &SimulationContext,
    ) -> Vec<Action> {
        Vec::new()
    }
}
