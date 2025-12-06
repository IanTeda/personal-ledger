mod error;
mod levels;

// Re-export main types for easier access
pub use error::{TelemetryError, TelemetryResult};

/// Re-export log level types
pub use levels::TelemetryLevels;