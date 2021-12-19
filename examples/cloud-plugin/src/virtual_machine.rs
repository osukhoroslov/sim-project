use core::match_event;
use core::actor::{ActorId, Actor, Event, ActorContext};

use crate::host::ReleaseVmResourses;
use crate::host::VM_FINISH_TIME;

// ACTORS //////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    pub id: String,
    pub cpu_usage: u32,
    pub ram_usage: u32,
    lifetime: f64,
    pub actor_id: ActorId
}

impl VirtualMachine {
    pub fn new(id: String, cpu: u32, ram: u32, lifetime: f64) -> Self {
        Self {
            id: id.clone(),
            cpu_usage: cpu,
            ram_usage: ram,
            lifetime: lifetime,
            actor_id: ActorId::from(&id)
        }
    }
}

// VM EVENTS ///////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct VMStart {
}

#[derive(Debug)]
pub struct VMAllocationFailed {
    pub reason: String,
}

#[derive(Debug)]
pub struct VMFinish {
    host_actor_id: ActorId
}

impl Actor for VirtualMachine {
    fn on(&mut self, event: Box<dyn Event>, 
                     from: ActorId, ctx: &mut ActorContext) {
        match_event!( event {
            VMStart { } => {
                println!("[time = {}] vm #{} initialized and started", ctx.time(), self.id);
                ctx.emit(VMFinish { host_actor_id: from }, ctx.id.clone(), self.lifetime);
            },
            VMAllocationFailed { reason } => {
                println!("[time = {}] vm #{} allocation failed due to: {}",
                          ctx.time(), self.id, reason);
            },
            VMFinish { host_actor_id } => {
                println!("[time = {}] vm #{} stopped due to lifecycle end", ctx.time(), self.id);
                ctx.emit(ReleaseVmResourses { vm_id: self.id.clone() },
                    host_actor_id.clone(),
                    VM_FINISH_TIME
                );
            },
        })
    }

    fn is_active(&self) -> bool {
        true
    }
}
