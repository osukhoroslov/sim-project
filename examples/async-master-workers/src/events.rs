use dslab_compute::multicore::{CompFinished, CompStarted};
use dslab_core::{async_core::await_details::EventKey, event::EventData};
use dslab_network::model::DataTransferCompleted;
use dslab_storage::events::{DataReadCompleted, DataWriteCompleted};

pub fn get_data_transfer_completed_details(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<DataTransferCompleted>().unwrap();
    event.dt.id as EventKey
}

pub fn get_data_read_completed_details(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<DataReadCompleted>().unwrap();
    event.request_id
}

pub fn get_data_write_completed_details(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<DataWriteCompleted>().unwrap();
    event.request_id
}

pub fn get_compute_start_details(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<CompStarted>().unwrap();
    event.id
}

pub fn get_compute_finished_details(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<CompFinished>().unwrap();
    event.id
}
