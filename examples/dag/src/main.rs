mod simple_scheduler;

use sugars::{rc, refcell};

use compute::multicore::*;
use core::simulation::Simulation;
use dag::dag::DAG;
use dag::parsers::yaml_parser::*;
use dag::runner::*;
use network::constant_bandwidth_model::ConstantBandwidthNetwork;
use network::network::Network;

use crate::simple_scheduler::SimpleScheduler;

fn run_simulation(dag: DAG, resources_file: &str, network_model: ConstantBandwidthNetwork, trace_file: &str) {
    let mut sim = Simulation::new(123);

    let resources = load_resources(resources_file, &mut sim);

    let network_model = rc!(refcell!(network_model));
    let network = rc!(refcell!(Network::new(network_model, sim.create_context("net"))));
    sim.add_handler("net", network.clone());

    let scheduler = SimpleScheduler::new();
    let runner_id = "runner";
    let runner = rc!(refcell!(DAGRunner::new(
        dag,
        network,
        resources,
        scheduler,
        sim.create_context(runner_id)
    )));
    sim.add_handler(runner_id, runner.clone());

    let mut client = sim.create_context("client");
    client.emit_now(Start {}, runner_id);
    sim.step_until_no_events();
    runner.borrow().trace_log().save_to_file(trace_file).unwrap();
}

fn map_reduce() {
    let mut dag = DAG::new();

    let data_part1 = dag.add_data_item("part1", 128);
    let data_part2 = dag.add_data_item("part2", 64);

    let map1 = dag.add_task("map1", 100, 512, 1, 2, CoresDependency::Linear);
    dag.add_data_dependency(data_part1, map1);
    let map1_out1 = dag.add_task_output(map1, "map1_out1", 10);
    let map1_out2 = dag.add_task_output(map1, "map1_out2", 10);
    let map1_out3 = dag.add_task_output(map1, "map1_out3", 10);
    let map1_out4 = dag.add_task_output(map1, "map1_out4", 10);

    let map2 = dag.add_task("map2", 120, 512, 2, 4, CoresDependency::Linear);
    dag.add_data_dependency(data_part2, map2);
    let map2_out1 = dag.add_task_output(map2, "map2_out1", 10);
    let map2_out2 = dag.add_task_output(map2, "map2_out2", 10);
    let map2_out3 = dag.add_task_output(map2, "map2_out3", 10);
    let map2_out4 = dag.add_task_output(map2, "map2_out4", 10);

    let reduce1 = dag.add_task("reduce1", 60, 128, 2, 3, CoresDependency::Linear);
    dag.add_data_dependency(map1_out1, reduce1);
    dag.add_data_dependency(map2_out1, reduce1);

    let reduce2 = dag.add_task("reduce2", 50, 128, 1, 1, CoresDependency::Linear);
    dag.add_data_dependency(map1_out2, reduce2);
    dag.add_data_dependency(map2_out2, reduce2);

    let reduce3 = dag.add_task("reduce3", 100, 128, 1, 2, CoresDependency::Linear);
    dag.add_data_dependency(map1_out3, reduce3);
    dag.add_data_dependency(map2_out3, reduce3);

    let reduce4 = dag.add_task("reduce4", 110, 128, 1, 1, CoresDependency::Linear);
    dag.add_data_dependency(map1_out4, reduce4);
    dag.add_data_dependency(map2_out4, reduce4);

    dag.add_task_output(reduce1, "result1", 32);
    dag.add_task_output(reduce2, "result2", 32);
    dag.add_task_output(reduce3, "result3", 32);
    dag.add_task_output(reduce4, "result4", 32);

    run_simulation(
        dag,
        "resources/map_reduce.yaml",
        ConstantBandwidthNetwork::new(10.0, 0.1),
        "traces/trace_map_reduce.json",
    );
}

fn epigenomics() {
    run_simulation(
        DAG::from_dax("graphs/Epigenomics_100.xml", 1000.),
        "resources/montage.yaml",
        ConstantBandwidthNetwork::new(100000.0, 10.),
        "traces/trace_epigenomics.json",
    );
}

fn montage() {
    run_simulation(
        DAG::from_dot("graphs/Montage.dot"),
        "resources/montage.yaml",
        ConstantBandwidthNetwork::new(0.01, 1.),
        "traces/trace_montage.json",
    );
}

fn diamond() {
    run_simulation(
        DAG::from_yaml("graphs/diamond.yaml"),
        "resources/diamond.yaml",
        ConstantBandwidthNetwork::new(100., 0.1),
        "traces/trace_diamond.json",
    );
}

fn main() {
    map_reduce();
    epigenomics(); // dax
    montage(); // dot
    diamond(); // yaml
}
