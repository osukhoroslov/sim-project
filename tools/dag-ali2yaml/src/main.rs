use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, write, File};
use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;
use csv::Reader;
use dslab_dag::parsers::yaml_parser::{DataItem, Task, Yaml};
use itertools::Itertools;
use serde_yaml::to_string;

#[derive(Parser)]
struct Args {
    /// Path to the trace directory.
    #[arg(short, long)]
    trace: PathBuf,

    /// Path to the output directory.
    #[arg(short, long)]
    out: PathBuf,

    /// Machine speed in Gflop/s. Used to convert durations to flops.
    #[arg(long)]
    machine_speed: f64,

    /// Maximum number of instances in a job.
    #[arg(long, default_value_t = u64::MAX)]
    max_ins: u64,

    /// Minimum number of instances in a job.
    #[arg(long, default_value_t = 0)]
    min_ins: u64,

    /// Maximum number of tasks in a job. Note that tasks in alibaba trace do not directly correspond to the DAG tasks, instances do.
    #[arg(long, default_value_t = u64::MAX)]
    max_tasks: u64,

    /// Minimum number of tasks in a job. Note that tasks in alibaba trace do not directly correspond to the DAG tasks, instances do.
    #[arg(long, default_value_t = 0)]
    min_tasks: u64,

    /// Maximum number of extracted DAGs (DAG = job).
    #[arg(long, default_value_t = u64::MAX)]
    max_dags: u64,

    /// Duration of each task will be max(real_duration, min_runtime). This option exists to fix zero-length tasks.
    #[arg(long, default_value_t = 1.)]
    min_runtime: f64,
}

fn select_dags(args: &Args) -> HashSet<String> {
    let mut path = args.trace.clone();
    path.push("batch_task.csv");
    let file = File::open(path).unwrap();
    let mut reader = Reader::from_reader(BufReader::with_capacity(1000000, file));
    let mut rows = Vec::new();
    for record in reader.records() {
        let row = record.unwrap();
        if row[0].starts_with("task") {
            continue;
        }
        if &row[4] != "Terminated" {
            rows.push((row[0].to_string(), u64::MAX, row[2].to_string()));
        } else {
            rows.push((
                row[0].to_string(),
                row[1].parse::<u64>().unwrap_or(u64::MAX),
                row[2].to_string(),
            ));
        }
    }
    rows.sort_by(|a, b| a.2.cmp(&b.2));
    let mut jobs = HashSet::new();
    for (job, data) in &rows.into_iter().chunk_by(|row| row.2.clone()) {
        let (ins, tasks, has) = data
            .map(|row| (row.1, row.1 == u64::MAX))
            .fold((0, 0, false), |acc, x| (acc.0 + x.0, acc.1 + 1, acc.2 || x.1));
        if !has && ins >= args.min_ins && ins <= args.max_ins && tasks >= args.min_tasks && tasks <= args.max_tasks {
            jobs.insert(job.clone());
        }
        if jobs.len() == args.max_dags as usize {
            break;
        }
    }
    jobs
}

fn extract_dags(dags: HashSet<String>, args: &Args) {
    create_dir_all(&args.out).unwrap();
    let mut path = args.trace.clone();
    path.push("batch_instance.csv");
    let file = File::open(path).unwrap();
    let mut reader = Reader::from_reader(BufReader::with_capacity(1000000, file));
    let mut dag_idx = HashMap::<String, usize>::with_capacity(dags.len());
    let mut dag_names = Vec::with_capacity(dags.len());
    for (i, dag) in dags.into_iter().enumerate() {
        dag_names.push(dag.clone());
        dag_idx.insert(dag, i);
    }
    let mut rows = vec![Vec::new(); dag_names.len()];
    for record in reader.records() {
        let row = record.unwrap();
        if let Some(i) = dag_idx.get(&row[2]).copied() {
            if &row[4] != "Terminated" {
                continue;
            }
            rows[i].push((
                row[0].to_string(),
                row[1].to_string(),
                row[5].parse::<f64>().unwrap(),
                row[6].parse::<f64>().unwrap(),
                row[10].parse::<f64>().unwrap(),
                row[11].parse::<f64>().unwrap(),
            ));
        }
    }
    for (name, rows) in dag_names.into_iter().zip(rows.into_iter()) {
        let mut yaml = Yaml::default();
        let mut task_instances = HashMap::<String, Vec<String>>::new();
        for row in &rows {
            let task = row
                .1
                .split('_')
                .next()
                .unwrap()
                .chars()
                .skip_while(|c| !c.is_digit(10))
                .collect::<String>();
            task_instances.entry(task).or_default().push(row.0.clone());
        }
        let mut pred = HashMap::<String, Vec<String>>::new();
        let mut succ = HashMap::<String, Vec<String>>::new();
        for row in &rows {
            let pred_this = pred.entry(row.0.to_string()).or_default();
            for id in row.1.split('_').skip(1) {
                for instance in task_instances.entry(id.to_string()).or_default() {
                    pred_this.push(format!("from_{}_to_{}", instance, row.0));
                    succ.entry(instance.to_string())
                        .or_default()
                        .push(format!("from_{}_to_{}", instance, row.0));
                }
            }
        }
        for row in rows.into_iter() {
            yaml.tasks.push(Task {
                name: row.0.clone(),
                flops: args.min_runtime.max((row.3 - row.2) * row.4 / 100.0) * args.machine_speed,
                memory: 0,
                min_cores: 1,
                max_cores: (row.5 / 100.0).ceil() as u32,
                cores_dependency: None,
                inputs: pred.remove(&row.0).unwrap_or_default(),
                outputs: succ
                    .remove(&row.0)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| DataItem { name, size: 0. })
                    .collect::<Vec<_>>(),
            });
        }
        let mut out = args.out.clone();
        out.push(format!("{}.yaml", &name));
        write(out, to_string(&yaml).unwrap()).unwrap();
    }
}

fn main() {
    let args = Args::parse();
    let dags = select_dags(&args);
    extract_dags(dags, &args);
}
