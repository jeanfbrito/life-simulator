//! Debug and monitoring systems for the life simulator

pub mod health_checks;
pub mod api;

pub use health_checks::{
    HealthAlert, HealthChecker, HealthCheckPlugin, AlertRecord, EntityHealthState,
};
pub use api::{HealthCheckApi, HealthCheckApiPlugin};
