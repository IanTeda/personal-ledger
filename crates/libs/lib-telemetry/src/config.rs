
//! # Telemetry Configuration
//!
//! This module provides configuration structures for the telemetry system in the Personal Ledger application.
//!
//! The telemetry configuration allows users to customize logging verbosity and behavior through
//! configuration files, environment variables, or programmatic settings. It serves as the
//! bridge between user preferences and the telemetry initialization system.
//!
//! ## Configuration Structure
//!
//! The `TelemetryConfig` struct encapsulates all telemetry-related settings:
//! - **Log Level**: Controls the verbosity of telemetry output (OFF, ERROR, WARN, INFO, DEBUG, TRACE)
//! - **Default Behavior**: Provides sensible defaults for production use
//!
//! ## Usage
//!
//! ```rust,ignore
//! use lib_telemetry::TelemetryConfig;
//!
//! // Create default configuration
//! let config = TelemetryConfig::default();
//!
//! // Access the configured telemetry level
//! let level = config.telemetry_level();
//!
//! // Use with telemetry initialization
//! lib_telemetry::init(Some(&level))?;
//!
//! # Ok::<(), lib_telemetry::TelemetryError>(())
//! ```
//!
//! ## Configuration File Example
//!
//! ```json
//! {
//!   "telemetry": {
//!     "telemetry_level": "debug"
//!   }
//! }
//! ```
//!
//! ## Environment Variables
//!
//! The telemetry level can also be overridden at runtime using the `RUST_LOG` environment variable:
//!
//! ```bash
//! RUST_LOG=debug cargo run
//! RUST_LOG=lib_telemetry=trace,backend=info cargo run
//! ```

/// Default telemetry level for production use.
///
/// This constant defines the baseline logging verbosity when no specific configuration
/// is provided. INFO level provides a good balance between visibility and performance
/// for production deployments.
const DEFAULT_TELEMETRY_LEVEL: super::TelemetryLevels = super::TelemetryLevels::INFO;

/// Configuration structure for telemetry settings.
///
/// This struct encapsulates all configurable aspects of the telemetry system,
/// providing a clean interface for applications to customize logging behavior.
/// The configuration is designed to be serializable, allowing it to be loaded
/// from various sources like configuration files, environment variables, or
/// programmatically constructed.
///
/// # Serialization
///
/// The struct supports both JSON and TOML serialization formats, making it
/// compatible with common configuration file formats.
///
/// # Default Values
///
/// When created with `Default::default()`, the configuration uses production-ready
/// defaults that balance observability with performance.
///
/// # Examples
///
/// ```rust,ignore
/// use lib_telemetry::TelemetryConfig;
///
/// // Default configuration
/// let config = TelemetryConfig::default();
///
/// // Custom configuration
/// let custom_config = TelemetryConfig {
///     telemetry_level: lib_telemetry::TelemetryLevels::DEBUG,
/// };
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct TelemetryConfig {
    /// The telemetry logging level for the application.
    ///
    /// This field controls the verbosity of telemetry output across the entire
    /// application. Lower levels (like ERROR) show fewer messages but may miss
    /// important debugging information. Higher levels (like TRACE) provide
    /// detailed insights but can impact performance and generate large log files.
    ///
    /// Available levels (from least to most verbose):
    /// - `OFF`: No telemetry output
    /// - `ERROR`: Only error conditions
    /// - `WARN`: Errors and warnings
    /// - `INFO`: General information (default)
    /// - `DEBUG`: Detailed debugging information
    /// - `TRACE`: Very detailed execution tracing
    pub telemetry_level: super::TelemetryLevels,
}

impl Default for TelemetryConfig {
    /// Creates a default telemetry configuration.
    ///
    /// The default configuration uses `INFO` level logging, which provides
    /// a good balance between observability and performance for production use.
    /// This level shows general application flow, important events, and
    /// non-critical warnings while avoiding excessive detail.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib_telemetry::TelemetryConfig;
    ///
    /// let config = TelemetryConfig::default();
    /// assert_eq!(config.telemetry_level(), lib_telemetry::TelemetryLevels::INFO);
    /// ```
    fn default() -> Self {
        Self {
            telemetry_level: DEFAULT_TELEMETRY_LEVEL,
        }
    }
}

impl TelemetryConfig {
    /// Get the configured telemetry log level.
    ///
    /// Returns the telemetry level that should be used for initializing
    /// the telemetry system. This level determines which log messages
    /// will be processed and displayed.
    ///
    /// # Returns
    ///
    /// The configured `TelemetryLevels` value that can be passed directly
    /// to `lib_telemetry::init()`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lib_telemetry::TelemetryConfig;
    ///
    /// let config = TelemetryConfig::default();
    /// let level = config.telemetry_level();
    ///
    /// // Use with telemetry initialization
    /// lib_telemetry::init(Some(&level))?;
    ///
    /// # Ok::<(), lib_telemetry::TelemetryError>(())
    /// ```
    pub fn telemetry_level(&self) -> super::TelemetryLevels {
        self.telemetry_level
    }
}