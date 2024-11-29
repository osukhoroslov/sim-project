use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;

use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value};

use dslab_compute::multicore::CoresDependency;

use crate::dag::*;
use crate::parsers::config::ParserConfig;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskSpec {
    id: String,
    parents: Vec<String>,
    children: Vec<String>,
    input_files: Option<Vec<String>>,
    output_files: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileSpec {
    id: String,
    size_in_bytes: u64,
}

#[derive(Serialize, Deserialize)]
struct Specification {
    tasks: Vec<TaskSpec>,
    files: Option<Vec<FileSpec>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskExec {
    // Task id
    id: String,
    // Task runtime in seconds
    runtime_in_seconds: f64,
    // Number of cores required by the task
    core_count: Option<f64>, // some files specify cores as "1.0"
    // Memory (resident set) size of the process in bytes
    memory_in_bytes: Option<u64>,
    // Machine used for task execution
    machines: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct Cpu {
    // CPU speed in MHz
    #[serde(rename = "speedInMHz")]
    speed: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct Machine {
    // Machine node name
    #[serde(rename = "nodeName")]
    name: String,
    // Machine's CPU information
    cpu: Option<Cpu>,
}

#[derive(Serialize, Deserialize)]
struct Execution {
    tasks: Vec<TaskExec>,
    machines: Option<Vec<Machine>>,
}

#[derive(Serialize, Deserialize)]
struct Workflow {
    specification: Specification,
    // According to the specification, this field is optional. But we can't properly restore the graph without it.
    execution: Execution,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    schema_version: String,
    workflow: Workflow,
}

impl DAG {
    /// Reads DAG from a file in [WfCommons json format](https://wfcommons.org/format).
    pub fn from_wfcommons<P: AsRef<Path>>(file: P, config: &ParserConfig) -> Self {
        let mut hasher = DefaultHasher::new();
        let str =
            &std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("Can't read file {}", file.as_ref().display()));
        str.hash(&mut hasher);
        let hash = hasher.finish();
        let mut rand = Pcg64::seed_from_u64(hash + config.seed.unwrap_or(123));

        let raw_json: serde_json::Value = from_str(str).unwrap_or_else(|e| {
            panic!(
                "Can't parse WfCommons json from file {}: {}",
                file.as_ref().display(),
                e
            )
        });

        // Check if the schema is too old.
        if let Some(ver) = raw_json.get("schemaVersion").and_then(|x| x.as_str()) {
            let mut tokens = ver.split(".");
            if let Some(first) = tokens.next().and_then(|x| x.parse::<i64>().ok()) {
                if first <= 1 {
                    if let Some(second) = tokens.next().and_then(|x| x.parse::<i64>().ok()) {
                        if second <= 4 {
                            return Self::from_wfcommons_legacy(file, config);
                        }
                    } else {
                        return Self::from_wfcommons_legacy(file, config);
                    }
                }
            }
        }

        let json: Json = from_value(raw_json).unwrap_or_else(|e| {
            panic!(
                "Can't parse WfCommons json from file {}: {}",
                file.as_ref().display(),
                e
            )
        });

        let workflow = json.workflow;
        let machine_speed: HashMap<String, f64> = workflow
            .execution
            .machines
            .map(|x| {
                x.iter()
                    .filter(|m| m.cpu.is_some() && m.cpu.as_ref().unwrap().speed.is_some())
                    // machine.cpu.speed in WfCommons format actually refers to CPU speed in MHz,
                    // but it seems everyone use it as Mflop/s too...
                    // here we convert it to Gflop/s
                    .map(|machine| {
                        (
                            machine.name.clone(),
                            machine.cpu.as_ref().unwrap().speed.unwrap() as f64 / 1000.,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut file_size: HashMap<String, u64> = HashMap::new();
        if let Some(files) = workflow.specification.files {
            for file in &files {
                file_size.insert(file.id.clone(), file.size_in_bytes);
            }
        }
        let mut dag = DAG::new();
        let mut data_items: HashMap<String, usize> = HashMap::new();
        let mut stage_mem: HashMap<String, u64> = HashMap::new();
        let mut stage_cores: HashMap<String, u32> = HashMap::new();
        let mut task_ids: HashMap<String, usize> = HashMap::new();
        let mut task_in_files: HashMap<String, Vec<String>> = HashMap::new();
        let mut task_out_files: HashMap<String, Vec<String>> = HashMap::new();
        for task in workflow.specification.tasks.iter() {
            if let Some(vec) = &task.input_files {
                task_in_files.insert(task.id.clone(), vec.clone());
            }
            if let Some(vec) = &task.output_files {
                task_out_files.insert(task.id.clone(), vec.clone());
            }
        }
        for task in workflow.execution.tasks.iter() {
            let stage = task.id.split('_').next().unwrap_or("none").to_string();

            let cores = if let Some(cores_conf) = &config.generate_cores {
                if cores_conf.regular && stage_cores.contains_key(&stage) {
                    stage_cores[&stage]
                } else {
                    let cores = rand.gen_range(cores_conf.min..=cores_conf.max);
                    if cores_conf.regular {
                        stage_cores.insert(stage.clone(), cores);
                    }
                    cores
                }
            } else {
                task.core_count.unwrap_or(1.) as u32
            };

            let memory = if config.ignore_memory {
                0
            } else if let Some(mem_conf) = &config.generate_memory {
                if mem_conf.regular && stage_mem.contains_key(&stage) {
                    stage_mem[&stage]
                } else {
                    let memory = (rand.gen_range(mem_conf.min..=mem_conf.max) as f64 / 1000.).ceil() as u64 * 1000;
                    if mem_conf.regular {
                        stage_mem.insert(stage, memory);
                    }
                    memory
                }
            } else if let Some(memory) = task.memory_in_bytes {
                (memory as f64 / 1e6).ceil() as u64 // convert bytes to MB (round up to nearest)
            } else {
                0
            };

            let mut flops = task.runtime_in_seconds * cores as f64;
            if let Some(machines) = task.machines.as_ref() {
                if let Some(machine_speed) = machines.iter().next().and_then(|m| machine_speed.get(m)) {
                    flops *= *machine_speed;
                } else {
                    flops *= config.reference_speed;
                }
            } else {
                flops *= config.reference_speed;
            }

            let task_id = dag.add_task(&task.id, flops, memory, cores, cores, CoresDependency::Linear);
            task_ids.insert(task.id.clone(), task_id);
            if let Some(vec) = task_out_files.get(&task.id) {
                for name in vec.iter() {
                    if let Some(size) = file_size.get(name).copied() {
                        data_items.insert(name.clone(), dag.add_task_output(task_id, name, size as f64 / 1e6));
                    }
                }
            }
        }
        for task in workflow.specification.tasks.iter() {
            let task_id = *task_ids.get(&task.id).unwrap();
            let mut predecessors: HashSet<usize> = HashSet::new();
            if let Some(vec) = task_in_files.get(&task.id) {
                for name in vec.iter() {
                    if let Some(data_item_id) = data_items.get(name) {
                        if let Some(producer) = dag.get_data_item(*data_item_id).producer {
                            predecessors.insert(producer);
                        }
                        dag.add_data_dependency(*data_item_id, task_id);
                    } else if let Some(size) = file_size.get(name) {
                        let data_item_id = dag.add_data_item(name, *size as f64 / 1e6);
                        data_items.insert(name.clone(), data_item_id);
                        dag.add_data_dependency(data_item_id, task_id);
                    }
                }
            }
            for parent in task.parents.iter() {
                if !predecessors.contains(&task_ids[parent]) {
                    let data_item_id = dag.add_task_output(task_ids[parent], &format!("{} -> {}", parent, task.id), 0.);
                    dag.add_data_dependency(data_item_id, task_id);
                    predecessors.insert(task_ids[parent]);
                }
            }
        }
        dag
    }
}
