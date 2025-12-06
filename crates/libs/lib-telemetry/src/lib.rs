mod error;
mod init;
mod levels;

// Re-export main types for easier access
pub use error::{TelemetryError, TelemetryResult};

// Re-export log level types
pub use levels::TelemetryLevels;

// Reexport init module
pub use init::init;