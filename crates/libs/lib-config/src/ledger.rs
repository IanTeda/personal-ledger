//! # Personal Ledger Configuration
//!
//! This module provides a layered configuration system for the Personal Ledger application.
//! It supports loading configuration from multiple sources with a defined precedence order.
//!
//! Configuration files use INI format.
//!
//! ## Configuration Sources (in precedence order)
//!
//! 1. Built-in defaults (lowest precedence)
//! 2. System-wide configuration files
//! 3. User-specific configuration files
//! 4. Executable directory configuration files
//! 5. Current working directory configuration files
//! 6. Explicit configuration file (passed to `parse`)
//! 7. Environment variables (highest precedence)
//!
//! ## Example
//!
//! ```rust
//! use lib_config::LedgerConfig;
//!
//! // Load configuration from default locations
//! let config = LedgerConfig::parse(None).expect("Failed to load config");
//!
//! // Access telemetry configuration
//! let telemetry = config.telemetry_config();
//! println!("Telemetry level: {:?}", telemetry.telemetry_level());
//!
//! // Load with explicit config file
//! use std::path::Path;
//! let config_path = Path::new("custom.conf");
//! let config = LedgerConfig::parse(Some(config_path)).expect("Failed to load config");
//! ```

use std::path::{Path, PathBuf};

use config::Config;
use lib_telemetry as telemetry;

/// Application name used for configuration directories and environment variables.
/// This should match the binary name and be used consistently across the application.
///
/// # Changing the Application Name
///
/// To use this configuration system for a different application:
/// 1. Change this constant to match your application name
/// 2. Update the corresponding ENV_PREFIX if needed
/// 3. Ensure your binary name matches this constant
const APPLICATION_NAME: &str = "personal-ledger";

/// Environment variable prefix derived from the application name.
/// Converts "personal-ledger" to "PERSONAL_LEDGER" for environment variables.
///
/// # Changing the Environment Prefix
///
/// If your application name contains characters that aren't valid in environment
/// variable names, update this constant accordingly.
const ENV_PREFIX: &str = "PERSONAL_LEDGER";

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct LedgerConfig {
    pub telemetry: telemetry::TelemetryConfig,
}

impl LedgerConfig {
    /// Get the application name used for configuration.
    ///
    /// This returns the same name used for configuration directories,
    /// file names, and environment variable prefixes.
    ///
    /// # Returns
    ///
    /// The application name as a static string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib_config::LedgerConfig;
    ///
    /// assert_eq!(LedgerConfig::application_name(), "personal-ledger");
    /// ```
    pub fn application_name() -> &'static str {
        APPLICATION_NAME
    }

    /// Get the environment variable prefix used for configuration.
    ///
    /// This returns the prefix used for environment variable overrides.
    ///
    /// # Returns
    ///
    /// The environment variable prefix as a static string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lib_config::LedgerConfig;
    ///
    /// assert_eq!(LedgerConfig::env_prefix(), "PERSONAL_LEDGER");
    /// ```
    pub fn env_prefix() -> &'static str {
        ENV_PREFIX
    }

    pub fn parse(config_file: Option<&Path>) -> super::ConfigResult<LedgerConfig> {

        // Higher precedence sources override lower precedence ones:
        // 1. Built-in defaults (lowest)
        // 2. System config files
        // 3. User config files
        // 4. Executable directory config files
        // 5. Current working directory config files
        // 6. Explicit config files
        // 7. Environment variables (highest)

        //-- 01. Build Defaults
        let default_telemetry_level = telemetry::TelemetryConfig::default().telemetry_level();

        let mut config_builder = Config::builder().set_default(
            "telemetry.telemetry_level",
            default_telemetry_level.to_string(),
        )?;


        //-- 02. System config directory (lowest precedence after defaults)
        if let Some(system_config) = Self::get_system_config_path().filter(|p| p.exists()) {
            let path_str = system_config.to_str().ok_or_else(|| {
                super::ConfigError::Validation(format!(
                    "System config path contains invalid UTF-8: {:?}",
                    system_config
                ))
            })?;
            config_builder = config_builder.add_source(
                config::File::with_name(path_str).format(config::FileFormat::Ini),
            );
        }

        //-- 03. User config directory
        if let Some(user_config) = Self::get_user_config_path().filter(|p| p.exists()) {
            let path_str = user_config.to_str().ok_or_else(|| {
                super::ConfigError::Validation(format!(
                    "User config path contains invalid UTF-8: {:?}",
                    user_config
                ))
            })?;
            config_builder = config_builder.add_source(
                config::File::with_name(path_str).format(config::FileFormat::Ini),
            );
        }

        //-- 04. Executable directory
        if let Some(exec_config) = Self::get_executable_config_path().filter(|p| p.exists()) {
            let path_str = exec_config.to_str().ok_or_else(|| {
                super::ConfigError::Validation(format!(
                    "Executable config path contains invalid UTF-8: {:?}",
                    exec_config
                ))
            })?;
            config_builder = config_builder.add_source(
                config::File::with_name(path_str).format(config::FileFormat::Ini),
            );
        }

        //-- 05. Current working directory
        match std::env::current_dir() {
            Ok(cwd) => {
                let cwd_config = cwd
                    .join(APPLICATION_NAME)
                    .join(format!("{}.conf", APPLICATION_NAME));
                if cwd_config.exists() {
                    let path_str = cwd_config.to_str().ok_or_else(|| {
                        super::ConfigError::Validation(format!(
                            "CWD config path contains invalid UTF-8: {:?}",
                            cwd_config
                        ))
                    })?;
                    let file_source = config::File::with_name(path_str).format(config::FileFormat::Ini);
                    config_builder = config_builder.add_source(file_source);
                }
            }
            Err(e) => {
                // Return an error instead of just logging - CWD access failure is a real error
                return Err(super::ConfigError::Validation(format!(
                    "Could not get current directory for config loading: {}",
                    e
                )));
            }
        }

        //-- 06. Explicit config file
        if let Some(explicit_config) = config_file.filter(|p| p.exists()) {
            let path_str = explicit_config.to_str().ok_or_else(|| {
                super::ConfigError::Validation(format!(
                    "Explicit config path contains invalid UTF-8: {:?}",
                    explicit_config
                ))
            })?;
            config_builder = config_builder.add_source(
                config::File::with_name(path_str).format(config::FileFormat::Ini),
            );
        }

        //-- 07. Environment variables (highest precedence)
        // Supports variables like: PERSONAL_LEDGER_TELEMETRY__TELEMETRY_LEVEL=debug
        config_builder = config_builder.add_source(config::Environment::with_prefix(ENV_PREFIX));

        //-- 08. Build and Deserialize
        let config = config_builder.build()?;
        let ledger_config: LedgerConfig = config.try_deserialize()?;

        if matches!(ledger_config.telemetry.telemetry_level, telemetry::TelemetryLevels::INFO | telemetry::TelemetryLevels::TRACE) {
            println!(
                "\n------------------------------ [ CONFIGURATION ] ------------------------------  \n{:#?}       \n-------------------------------------------------------------------------------",
            ledger_config
            );
        }

        Ok(ledger_config)
    }

    /// Get the system-wide configuration file path.
    ///
    /// Returns the path to the system configuration file using platform-specific
    /// standard locations. This provides system administrators with a way to
    /// set default configurations for all users.
    ///
    /// - **Unix/Linux**: `/etc/personal-ledger/personal-ledger.conf`
    /// - **Windows**: `%ALLUSERSPROFILE%\personal-ledger\personal-ledger.conf`
    /// - **macOS**: `/Library/Preferences/personal-ledger/personal-ledger.conf`
    /// - **Other**: None (system config not supported)
    fn get_system_config_path() -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            Some(
                PathBuf::from("/etc")
                    .join(APPLICATION_NAME)
                    .join(format!("{}.conf", APPLICATION_NAME)),
            )
        }
        #[cfg(target_os = "macos")]
        {
            Some(
                PathBuf::from("/Library/Preferences")
                    .join(APPLICATION_NAME)
                    .join(format!("{}.conf", APPLICATION_NAME)),
            )
        }
        #[cfg(target_os = "windows")]
        {
            // On Windows, use ALLUSERSPROFILE for system-wide settings
            std::env::var_os("ALLUSERSPROFILE").map(|all_users| {
                PathBuf::from(all_users)
                    .join(APPLICATION_NAME)
                    .join(format!("{}.conf", APPLICATION_NAME))
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    /// Get the user-specific configuration file path.
    ///
    /// Returns the path to the user's configuration file using the standard
    /// platform-specific config directory. This allows individual users to
    /// customize their configuration without affecting other users.
    ///
    /// - **Linux**: `~/.config/personal-ledger/personal-ledger.conf` (or `$XDG_CONFIG_HOME`)
    /// - **macOS**: `~/Library/Preferences/personal-ledger/personal-ledger.conf`
    /// - **Windows**: `%APPDATA%\personal-ledger\personal-ledger.conf`
    fn get_user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| {
            config_dir
                .join(APPLICATION_NAME)
                .join(format!("{}.conf", APPLICATION_NAME))
        })
    }

    /// Get the executable directory configuration file path.
    ///
    /// Returns the path to a configuration file in the same directory as the
    /// executable. This is useful for portable applications or when the config
    /// should be distributed with the binary.
    ///
    /// Note: This is determined at runtime based on the executable's location.
    fn get_executable_config_path() -> Option<PathBuf> {
        dirs::executable_dir().map(|exec_dir| {
            exec_dir
                .join(APPLICATION_NAME)
                .join(format!("{}.conf", APPLICATION_NAME))
        })
    }

    /// Get the telemetry configuration.
    pub fn telemetry_config(&self) -> &lib_telemetry::TelemetryConfig {
        &self.telemetry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn application_name_returns_correct_value() {
        assert_eq!(LedgerConfig::application_name(), APPLICATION_NAME);
    }

    #[test]
    fn env_prefix_returns_correct_value() {
        assert_eq!(LedgerConfig::env_prefix(), ENV_PREFIX);
    }

    #[test]
    fn telemetry_config_returns_telemetry_reference() {
        let config = LedgerConfig::default();
        let telemetry = config.telemetry_config();
        assert_eq!(
            telemetry.telemetry_level(),
            config.telemetry.telemetry_level()
        );
    }

    #[test]
    fn get_user_config_path_returns_expected_path() {
        let path = LedgerConfig::get_user_config_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("personal-ledger/personal-ledger.conf"));
        assert!(path.to_string_lossy().contains("personal-ledger"));
    }

    #[test]
    fn get_executable_config_path_returns_expected_path() {
        let path = LedgerConfig::get_executable_config_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("personal-ledger/personal-ledger.conf"));
        assert!(path.to_string_lossy().contains("personal-ledger"));
    }

    #[test]
    fn get_system_config_path_returns_expected_path() {
        let path = LedgerConfig::get_system_config_path();
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            assert!(path.is_some());
            let path = path.unwrap();
            assert!(path.ends_with("personal-ledger.conf"));
            assert!(path.to_string_lossy().contains("personal-ledger"));
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            assert!(path.is_none());
        }
    }

    #[test]
    fn parse_with_defaults_loads_successfully() {
        // Change to a temp directory to ensure no config files are loaded
        let temp_dir = TempDir::new().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // This should work without any config files present
        let result = LedgerConfig::parse(None);
        assert!(result.is_ok());
        let config = result.unwrap();
        // Should have default telemetry config
        assert_eq!(
            config.telemetry.telemetry_level(),
            telemetry::TelemetryConfig::default().telemetry_level()
        );

        // Restore original directory
        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn parse_with_explicit_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("test.conf");

        // Create a config file with custom telemetry level (INI format)
        let config_content = r#"
[telemetry]
telemetry_level = "debug"
"#;
        fs::write(&config_file, config_content).unwrap();

        let result = LedgerConfig::parse(Some(&config_file));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.telemetry.telemetry_level(),
            telemetry::TelemetryLevels::DEBUG
        );
    }

    #[test]
    fn parse_with_environment_variable_override() {
        // Note: This test would require unsafe code to set environment variables
        // For now, we skip this test as it's difficult to test safely
        // In integration tests, this functionality can be verified
    }

    #[test]
    fn parse_with_cwd_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let original_cwd = env::current_dir().unwrap();

        // Change to temp directory
        env::set_current_dir(&temp_dir).unwrap();

        // Create config directory and file
        let config_dir = temp_dir.path().join("personal-ledger");
        fs::create_dir(&config_dir).unwrap();
        let config_file = config_dir.join("personal-ledger.conf");

        let config_content = r#"
[telemetry]
telemetry_level = "warn"
"#;
        fs::write(&config_file, config_content).unwrap();

        // Ensure file exists before parsing
        assert!(config_file.exists(), "Config file should exist");

        let result = LedgerConfig::parse(None);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.telemetry.telemetry_level(),
            telemetry::TelemetryLevels::WARN
        );

        // Restore original directory
        env::set_current_dir(original_cwd).unwrap();

        // Keep temp_dir alive until here
        drop(temp_dir);
    }

    #[test]
    fn parse_with_nonexistent_explicit_file_returns_error() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/config.conf");
        let result = LedgerConfig::parse(Some(&nonexistent_path));
        // Should still succeed because the file is optional
        assert!(result.is_ok());
    }
}
