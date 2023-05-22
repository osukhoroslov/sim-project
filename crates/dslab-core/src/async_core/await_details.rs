//! TODO makogon
//!
//!

use crate::{event::EventData, Event, Id};

use super::shared_state::EmptyData;

/// type of key that represents the details of event to wait for
pub type DetailsKey = u64;

/// enum represents the await resuls of SimulationContext::async_wait_for_event...
pub enum AwaitResult<T: EventData> {
    /// contains Event with time and source that it was waited from. Id and data are empty
    Timeout(Event),
    /// contains full event without data, and data of specific type separately
    Ok((Event, T)),
}

impl<T: EventData> Default for AwaitResult<T> {
    fn default() -> Self {
        Self::Timeout(Event {
            id: 0,
            time: 0.,
            src: 0,
            dest: 0,
            data: Box::new(EmptyData {}),
        })
    }
}

impl<T: EventData> AwaitResult<T> {
    /// create a default result
    pub(crate) fn timeout_with(src: Id, dest: Id) -> Self {
        Self::Timeout(Event {
            id: 0,
            time: 0.,
            src,
            dest,
            data: Box::new(EmptyData {}),
        })
    }
}
