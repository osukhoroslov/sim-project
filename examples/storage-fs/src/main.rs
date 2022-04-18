use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use env_logger::Builder;
use serde::Serialize;
use sugars::{rc, refcell};

use simcore::cast;
use simcore::context::SimulationContext;
use simcore::event::Event;
use simcore::handler::EventHandler;
use simcore::log_debug;
use simcore::simulation::Simulation;

use storage::disk::Disk;
use storage::events::{FileReadCompleted, FileReadFailed, FileWriteCompleted, FileWriteFailed};
use storage::fs::FileSystem;

const SEED: u64 = 16;
const CASES_COUNT: u64 = 6;

const FILESYSTEM_NAME: &str = "FileSystem-1";
const DISK_1_NAME: &str = "Disk-1";
const DISK_2_NAME: &str = "Disk-2";

const USER_NAME: &str = "User";

const FILE_1_NAME: &str = "/disk1/file1";
const FILE_2_NAME: &str = "/disk2/file2";

const DISK_1_CAPACITY: u64 = 1000000000;
const DISK_2_CAPACITY: u64 = 10000000;

const DISK_1_MOUNT_POINT: &str = "/disk1/";
const DISK_2_MOUNT_POINT: &str = "/disk2/";

const DISK_1_READ_BW: u64 = 100;
const DISK_2_READ_BW: u64 = 100000;

const DISK_1_WRITE_BW: u64 = 100;
const DISK_2_WRITE_BW: u64 = 1000;

struct User {
    fs: Rc<RefCell<FileSystem>>,
    ctx: SimulationContext,
}

#[derive(Serialize)]
struct Run {
    case: u64,
}

#[derive(Serialize)]
struct Init {}

impl User {
    fn new(fs: Rc<RefCell<FileSystem>>, ctx: SimulationContext) -> Self {
        Self { fs, ctx }
    }
}

impl EventHandler for User {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            Init {} => {}
            Run { case } => {
                match case {
                    0 => {
                        self.fs.borrow_mut().create_file(FILE_1_NAME).unwrap();
                        self.fs.borrow_mut().get_file_size(FILE_1_NAME).unwrap();
                        log_debug!(self.ctx, "Trying to read 3 bytes from empty file... should fail");
                        self.fs.borrow_mut().read(FILE_1_NAME, 3, self.ctx.id());
                    }
                    1 => {
                        log_debug!(self.ctx, "Writing 5 bytes to file [{}]", FILE_1_NAME);
                        self.fs.borrow_mut().write(FILE_1_NAME, 5, self.ctx.id());
                    }
                    2 => {
                        log_debug!(self.ctx, "Reading all from file [{}]", FILE_1_NAME);
                        self.fs.borrow_mut().read_all(FILE_1_NAME, self.ctx.id());
                    }
                    3 => {
                        log_debug!(self.ctx, "Testing another disk for file [{}]", FILE_2_NAME);
                        self.fs.borrow_mut().create_file(FILE_2_NAME).unwrap();
                        self.fs.borrow_mut().write(FILE_2_NAME, 5, self.ctx.id());
                    }
                    4 => {
                        log_debug!(self.ctx, "Deleting file [{}] and then trying to access", FILE_1_NAME);
                        self.fs.borrow_mut().delete_file(FILE_1_NAME).unwrap();
                        self.fs.borrow_mut().write(FILE_1_NAME, 1, self.ctx.id());
                        self.fs.borrow_mut().read_all(FILE_1_NAME, self.ctx.id());
                    }
                    5 => {
                        log_debug!(
                            self.ctx,
                            "Requesting some actions and trying to delete file [{}]",
                            FILE_2_NAME
                        );
                        self.fs.borrow_mut().write(FILE_2_NAME, 1, self.ctx.id());
                        self.fs.borrow_mut().read_all(FILE_2_NAME, self.ctx.id());
                        log_debug!(
                            self.ctx,
                            "Received error: {}",
                            self.fs.borrow_mut().delete_file(FILE_2_NAME).err().unwrap()
                        )
                    }
                    _ => {
                        panic!("Wrong test case number");
                    }
                }
            }
            FileReadCompleted {
                request_id: _,
                file_name,
                read_size,
            } => {
                log_debug!(
                    self.ctx,
                    "Completed reading {} bytes from file [{}]",
                    read_size,
                    file_name
                );
            }
            FileReadFailed {
                request_id: _,
                file_name,
                error,
            } => {
                log_debug!(self.ctx, "Failed reading from file [{}], error: {}", file_name, error,);
            }
            FileWriteCompleted {
                request_id: _,
                file_name,
                new_size,
            } => {
                log_debug!(
                    self.ctx,
                    "Completed writing to file [{}], new_size = {}",
                    file_name,
                    new_size
                );
            }
            FileWriteFailed {
                request_id: _,
                file_name,
                error,
            } => {
                log_debug!(self.ctx, "Failed writing to file [{}], error: {}", file_name, error,);
            }
        })
    }
}

fn main() {
    println!("Starting...");

    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let mut sim = Simulation::new(SEED);

    let disk1 = rc!(refcell!(Disk::new_simple(
        DISK_1_CAPACITY,
        DISK_1_READ_BW,
        DISK_1_WRITE_BW,
        sim.create_context(DISK_1_NAME),
    )));

    let disk2 = rc!(refcell!(Disk::new_simple(
        DISK_2_CAPACITY,
        DISK_2_READ_BW,
        DISK_2_WRITE_BW,
        sim.create_context(DISK_2_NAME),
    )));

    let fs = rc!(refcell!(FileSystem::new(sim.create_context(FILESYSTEM_NAME))));
    sim.add_handler(FILESYSTEM_NAME, fs.clone());

    fs.borrow_mut().mount_disk(DISK_1_MOUNT_POINT, disk1).unwrap();
    fs.borrow_mut().mount_disk(DISK_2_MOUNT_POINT, disk2).unwrap();

    let user = rc!(refcell!(User::new(fs.clone(), sim.create_context(USER_NAME))));
    let user_id = sim.add_handler(USER_NAME, user);

    let mut root = sim.create_context("root");

    root.emit_now(Init {}, user_id);
    sim.step_until_no_events();

    for case in 0..CASES_COUNT {
        println!("Running case {}", case);
        root.emit_now(Run { case }, user_id);
        sim.step_until_no_events();
        println!(
            "Total FS used space after case {} is {} bytes",
            case,
            fs.borrow().get_used_space()
        );
        println!("########################################################")
    }

    fs.borrow_mut().unmount_disk(DISK_1_MOUNT_POINT).unwrap();
    fs.borrow_mut().unmount_disk(DISK_2_MOUNT_POINT).unwrap();

    println!("Finish");
}
