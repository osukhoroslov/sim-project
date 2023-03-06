//! Disk model.
//!
//! It has two main methods - [`read`](Disk::read) and [`write`](Disk::write),
//! and some utility functions as [`mark_free`](Disk::mark_free) or [`get_used_space`](Disk::get_used_space).
//! It can be created by [`new_simple`](Disk::new_simple) function if bandwidths are fixed.
//!
//! This model supports concurrent execution of requests with bandwidth sharing.
//! It uses the generic fair throughput sharing model from `dslab-models` to compute the request completion times.
//! Methods set is the same as for simple disk model.
//!
//! There is also support for [throughput factor functions](crate::throughput_factor)
//! that dynamically compute per-request bandwidth based on the request size, current simulation time, etc.
//! Several implementations of these models are included in this crate, and other user-defined models can also be used.
//!
//! Usage examples can be found in `/examples/storage-disk` and `/examples/storage-shared-disk`.
//! Benchmark can be found in `/examples/storage-shared-disk-benchmark` and `/examples-other/simgrid/storage`

use serde::Serialize;
use sugars::boxed;

use dslab_core::cast;
use dslab_core::component::Id;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;
use dslab_core::{context::SimulationContext, log_debug, log_error};

use crate::events::{DataReadCompleted, DataReadFailed, DataWriteCompleted, DataWriteFailed};
use crate::storage::{Storage, StorageInfo};
use dslab_models::throughput_sharing::throughput_factor::{ConstantThroughputFactorFunction, ThroughputFactorFunction};
use dslab_models::throughput_sharing::{
    make_constant_throughput_function, FairThroughputSharingModel, ThroughputFunction, ThroughputSharingModel,
};

/// Struct for a disk activity.
#[derive(Clone)]
pub struct DiskActivity {
    request_id: u64,
    requester: Id,
    size: u64,
}

#[derive(Serialize)]
struct DiskReadActivityCompleted {}

#[derive(Serialize)]
struct DiskWriteActivityCompleted {}

/// Representation of disk.
///
/// Disk is characterized by its capacity, read/write throughput models
/// and read/write throughput factor functions.
///
/// Disk state includes the amount of used disk space and state of throughput models.
pub struct Disk {
    capacity: u64,
    used: u64,
    read_throughput_model: FairThroughputSharingModel<DiskActivity>,
    write_throughput_model: FairThroughputSharingModel<DiskActivity>,
    next_request_id: u64,
    next_read_event: u64,
    next_write_event: u64,
    ctx: SimulationContext,
}

impl Disk {
    /// Creates new disk with fixed read and write throughput and default throughput factor functions.
    pub fn new_simple(capacity: u64, read_bandwidth: f64, write_bandwidth: f64, ctx: SimulationContext) -> Self {
        Self {
            capacity,
            used: 0,
            read_throughput_model: FairThroughputSharingModel::new(
                make_constant_throughput_function(read_bandwidth),
                boxed!(ConstantThroughputFactorFunction::new(1.)),
            ),
            write_throughput_model: FairThroughputSharingModel::new(
                make_constant_throughput_function(write_bandwidth),
                boxed!(ConstantThroughputFactorFunction::new(1.)),
            ),
            next_request_id: 0,
            next_read_event: u64::MAX,
            next_write_event: u64::MAX,
            ctx,
        }
    }

    /// Creates new disk with given read and write throughput functions and default throughput factor functions.
    pub fn new(
        capacity: u64,
        read_throughput_function: ThroughputFunction,
        write_throughput_function: ThroughputFunction,
        read_throughput_factor_function: Box<dyn ThroughputFactorFunction<DiskActivity>>,
        write_throughput_factor_function: Box<dyn ThroughputFactorFunction<DiskActivity>>,
        ctx: SimulationContext,
    ) -> Self {
        Self {
            capacity,
            used: 0,
            read_throughput_model: FairThroughputSharingModel::new(
                read_throughput_function,
                read_throughput_factor_function,
            ),
            write_throughput_model: FairThroughputSharingModel::new(
                write_throughput_function,
                write_throughput_factor_function,
            ),
            next_request_id: 0,
            next_read_event: u64::MAX,
            next_write_event: u64::MAX,
            ctx,
        }
    }

    fn make_unique_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        request_id
    }

    fn schedule_next_read_event(&mut self) {
        if let Some((time, _)) = self.read_throughput_model.peek() {
            self.next_read_event = self.ctx.emit_self(DiskReadActivityCompleted {}, time - self.ctx.time());
        }
    }

    fn schedule_next_write_event(&mut self) {
        if let Some((time, _)) = self.write_throughput_model.peek() {
            self.next_write_event = self
                .ctx
                .emit_self(DiskWriteActivityCompleted {}, time - self.ctx.time());
        }
    }

    fn on_read_completed(&mut self) {
        let (_, activity) = self.read_throughput_model.pop().unwrap();
        self.ctx.emit_now(
            DataReadCompleted {
                request_id: activity.request_id,
                size: activity.size,
            },
            activity.requester,
        );
        self.schedule_next_read_event();
    }

    fn on_write_completed(&mut self) {
        let (_, activity) = self.write_throughput_model.pop().unwrap();
        self.ctx.emit_now(
            DataWriteCompleted {
                request_id: activity.request_id,
                size: activity.size,
            },
            activity.requester,
        );
        self.schedule_next_write_event();
    }
}

impl Storage for Disk {
    fn read(&mut self, size: u64, requester: Id) -> u64 {
        log_debug!(
            self.ctx,
            "Received read request, size: {}, requester: {}",
            size,
            requester
        );
        let request_id = self.make_unique_request_id();
        if size > self.capacity {
            let error = format!(
                "requested read size is {} but only {} is available",
                size, self.capacity
            );
            log_error!(self.ctx, "Failed reading: {}", error,);
            self.ctx.emit_now(DataReadFailed { request_id, error }, requester);
        } else {
            self.read_throughput_model.insert(
                DiskActivity {
                    request_id,
                    requester,
                    size,
                },
                size as f64,
                &mut self.ctx,
            );
            self.ctx.cancel_event(self.next_read_event);
            self.schedule_next_read_event();
        }
        request_id
    }

    fn write(&mut self, size: u64, requester: Id) -> u64 {
        let request_id = self.make_unique_request_id();
        log_debug!(
            self.ctx,
            "Received write request, size: {}, requester: {}",
            size,
            requester
        );
        let available = self.capacity - self.used;
        if available < size {
            let error = format!("requested write size is {} but only {} is available", size, available);
            log_error!(self.ctx, "Failed writing: {}", error,);
            self.ctx.emit_now(DataWriteFailed { request_id, error }, requester);
        } else {
            self.used += size;
            self.write_throughput_model.insert(
                DiskActivity {
                    request_id,
                    requester,
                    size,
                },
                size as f64,
                &mut self.ctx,
            );
            self.ctx.cancel_event(self.next_write_event);
            self.schedule_next_write_event();
        }
        request_id
    }

    fn mark_free(&mut self, size: u64) -> Result<(), String> {
        if size <= self.used {
            self.used -= size;
            return Ok(());
        }
        Err(format!("invalid size: {}", size))
    }

    fn used_space(&self) -> u64 {
        self.used
    }

    fn free_space(&self) -> u64 {
        self.capacity - self.used
    }

    fn capacity(&self) -> u64 {
        self.capacity
    }

    fn id(&self) -> Id {
        self.ctx.id()
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            capacity: self.capacity(),
            used_space: self.used_space(),
            free_space: self.free_space(),
        }
    }
}

impl EventHandler for Disk {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            DiskReadActivityCompleted {} => {
                self.on_read_completed();
            }
            DiskWriteActivityCompleted {} => {
                self.on_write_completed();
            }
        })
    }
}
