use std::ops::{Index, IndexMut, Range};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InvocationStatus {
    /// Invocation is registered, but simulation time has not reached its arrival time yet.
    NotArrived,
    /// Invocation is queued at one of the invokers.
    Queued,
    /// Invocation is waiting for the assigned container to start.
    WaitingForContainer,
    /// Invocation is running.
    Running,
    /// Invocation is finished.
    Finished
}

#[derive(Copy, Clone)]
pub struct Invocation {
    pub id: usize,
    pub func_id: usize,
    pub duration: f64,
    pub arrival_time: f64,
    pub status: InvocationStatus,
    pub host_id: Option<usize>,
    pub container_id: Option<usize>,
    pub started: Option<f64>,
    pub finished: Option<f64>,
}

#[derive(Default)]
pub struct InvocationRegistry {
    invocations: Vec<Invocation>,
}

impl InvocationRegistry {
    pub fn new_invocation(&mut self, func_id: usize, duration: f64, arrival_time: f64) -> usize {
        let id = self.invocations.len();
        let invocation = Invocation {
            id,
            func_id,
            duration,
            arrival_time,
            status: InvocationStatus::NotArrived,
            host_id: None,
            container_id: None,
            started: None,
            finished: None,
        };
        self.invocations.push(invocation);
        id
    }

    pub fn len(&self) -> usize {
        self.invocations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.invocations.is_empty()
    }
}

impl Index<usize> for InvocationRegistry {
    type Output = Invocation;

    fn index(&self, index: usize) -> &Self::Output {
        &self.invocations[index]
    }
}

impl Index<Range<usize>> for InvocationRegistry {
    type Output = [Invocation];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.invocations[index]
    }
}

impl IndexMut<usize> for InvocationRegistry {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.invocations[index]
    }
}
