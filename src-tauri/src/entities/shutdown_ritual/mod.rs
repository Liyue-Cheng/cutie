//! Daily Shutdown Ritual entities
//!
//! - Steps: persistent templates for the ritual
//! - Progress: per-day completion state (reset by date)

mod model;
mod request_dtos;
mod response_dtos;

pub use model::{ShutdownRitualStep, ShutdownRitualStepRow};
pub use request_dtos::{
    CreateShutdownRitualStepRequest, ToggleShutdownRitualRequest, UpdateShutdownRitualStepRequest,
    UpdateShutdownRitualStepSortRequest,
};
pub use response_dtos::{
    ShutdownRitualProgressDto, ShutdownRitualStateDto, ShutdownRitualStepDto,
    UpdateShutdownRitualStepSortResponse,
};


