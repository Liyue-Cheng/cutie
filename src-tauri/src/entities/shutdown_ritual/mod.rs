//! Daily Shutdown Ritual entities
//!
//! - Steps: persistent templates for the ritual
//! - Progress: per-day completion state (reset by date)

mod model;
mod request_dtos;
mod response_dtos;

pub use model::{
    ShutdownRitualSettings, ShutdownRitualSettingsRow, ShutdownRitualStep, ShutdownRitualStepRow,
};
pub use request_dtos::{
    CreateShutdownRitualStepRequest, ToggleShutdownRitualRequest, UpdateShutdownRitualSettingsRequest,
    UpdateShutdownRitualStepRequest, UpdateShutdownRitualStepSortRequest,
};
pub use response_dtos::{
    ShutdownRitualProgressDto, ShutdownRitualSettingsDto, ShutdownRitualStateDto,
    ShutdownRitualStepDto, UpdateShutdownRitualStepSortResponse,
};


