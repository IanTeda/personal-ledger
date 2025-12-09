//! # Database Error Types
//!
//! This module defines the `DatabaseError` enum, which standardises error handling for
//! all database-related operations in the Personal Ledger backend. It wraps connection,
//! migration, SQLx, and other database errors, and provides variants for validation and
//! generic database failures.
//!
//! ## Error Variants
//!
//! - `Connection`: Database connection failures (invalid config, unreachable server, etc.)
//! - `Sqlx`: Errors from the `sqlx` crate (query, pool, etc.)
//! - `Migration`: Errors from running migrations
//! - `Validation`: Domain validation errors (constraint violations, etc.)
//! - `NotFound`: Resource not found errors
//! - `Generic`: Catch-all for miscellaneous DB errors
//!
//! ## Usage
//!
//! All database service functions should return `Result<T, DatabaseError>` for consistent error propagation.
//!
//! Example:
//! ```rust
//! use lib_database::DatabaseError;
//! fn do_db_work() -> Result<(), DatabaseError> {
//!     // ...
//!     Ok(())
//! }
//! ```
//!
//! ## Integration
//!
//! Errors are convertible to `LedgerError` for unified error handling across the backend.

/// Result type alias used across database modules.
///
/// Use `DatabaseResult<T>` for functions that return `T` or a `DatabaseError`.
/// This keeps signatures concise and makes it clear the function is database-related.
///
/// Example:
///
/// ```rust
/// use lib_database::DatabaseResult;
/// fn get_data() -> DatabaseResult<i32> {
///     Ok(42)
/// }
/// ```
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[derive(thiserror::Error, Debug)]
/// Errors produced during database operations.
///
/// This enum wraps errors from the underlying database operations and adds
/// domain-specific validation variants for database-related failures.
pub enum DatabaseError {
    /// Connection error
    #[error("Error connecting to the database: {0}")]
    Connection(String),

    /// Wrap underlying sqlx errors
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Validation errors originating from the DB layer (e.g. constraint violations)
    #[error("Validation: {0}")]
    Validation(String),

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Generic catch-all for database related errors
    #[error("Other database error: {0}")]
    Generic(String),
}

impl PartialEq for DatabaseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DatabaseError::Connection(a), DatabaseError::Connection(b)) => a == b,
            (DatabaseError::Sqlx(a), DatabaseError::Sqlx(b)) => format!("{:?}", a) == format!("{:?}", b),
            (DatabaseError::Migration(a), DatabaseError::Migration(b)) => format!("{:?}", a) == format!("{:?}", b),
            (DatabaseError::Validation(a), DatabaseError::Validation(b)) => a == b,
            (DatabaseError::NotFound(a), DatabaseError::NotFound(b)) => a == b,
            (DatabaseError::Generic(a), DatabaseError::Generic(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DatabaseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn test_database_result_type_alias() {
        // Test that DatabaseResult<T> is equivalent to Result<T, DatabaseError>
        let ok_result: DatabaseResult<i32> = Ok(42);
        assert_eq!(ok_result, Ok(42));

        let err_result: DatabaseResult<i32> = Err(DatabaseError::Validation("test".to_string()));
        assert!(err_result.is_err());
        assert!(matches!(err_result, Err(DatabaseError::Validation(_))));
    }

    #[test]
    fn test_database_error_variants() {
        // Test Connection variant
        let conn_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let conn_err = DatabaseError::Connection(conn_msg);
        assert!(matches!(conn_err, DatabaseError::Connection(_)));

        // Test Sqlx variant (via From)
        let sqlx_err = sqlx::Error::RowNotFound;
        let db_err: DatabaseError = sqlx_err.into();
        assert!(matches!(db_err, DatabaseError::Sqlx(_)));

        // Test Migration variant (via From)
        let migrate_err = sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound);
        let db_err: DatabaseError = migrate_err.into();
        assert!(matches!(db_err, DatabaseError::Migration(_)));

        // Test Validation variant
        let val_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let val_err = DatabaseError::Validation(val_msg);
        assert!(matches!(val_err, DatabaseError::Validation(_)));

        // Test NotFound variant
        let not_found_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let not_found_err = DatabaseError::NotFound(not_found_msg);
        assert!(matches!(not_found_err, DatabaseError::NotFound(_)));

        // Test Other variant
        let other_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let other_err = DatabaseError::Generic(other_msg);
        assert!(matches!(other_err, DatabaseError::Generic(_)));
    }

    #[test]
    fn test_database_error_display() {
        let conn_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let conn_err = DatabaseError::Connection(conn_msg.clone());
        assert_eq!(format!("{}", conn_err), format!("Error connecting to the database: {}", conn_msg));

        let sqlx_err = DatabaseError::Sqlx(sqlx::Error::RowNotFound);
        assert!(format!("{}", sqlx_err).contains("Database error:"));

        let migrate_err = DatabaseError::Migration(sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound));
        assert!(format!("{}", migrate_err).contains("Database migration error:"));

        let val_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let val_err = DatabaseError::Validation(val_msg.clone());
        assert_eq!(format!("{}", val_err), format!("Validation: {}", val_msg));

        let not_found_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let not_found_err = DatabaseError::NotFound(not_found_msg.clone());
        assert_eq!(format!("{}", not_found_err), format!("Not found: {}", not_found_msg));

        let other_msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let other_err = DatabaseError::Generic(other_msg.clone());
        assert_eq!(format!("{}", other_err), format!("Other database error: {}", other_msg));
    }

    #[test]
    fn test_database_error_debug() {
        let msg: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let err = DatabaseError::Connection(msg.clone());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Connection"));
        assert!(debug_str.contains(&msg));
    }

    #[test]
    fn test_database_error_from_conversions() {
        // Test From<sqlx::Error>
        let sqlx_err = sqlx::Error::RowNotFound;
        let db_err: DatabaseError = sqlx_err.into();
        assert!(matches!(db_err, DatabaseError::Sqlx(_)));

        // Test From<sqlx::migrate::MigrateError>
        let migrate_err = sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound);
        let db_err: DatabaseError = migrate_err.into();
        assert!(matches!(db_err, DatabaseError::Migration(_)));
    }

    #[test]
    fn test_database_error_partial_eq() {
        let msg1: String = fake::faker::lorem::en::Sentence(3..10).fake();
        let msg2: String = fake::faker::lorem::en::Sentence(3..10).fake();

        // Test equal Connection errors
        let err1 = DatabaseError::Connection(msg1.clone());
        let err2 = DatabaseError::Connection(msg1.clone());
        assert_eq!(err1, err2);

        // Test unequal Connection errors
        let err3 = DatabaseError::Connection(msg2.clone());
        assert_ne!(err1, err3);

        // Test different variants are not equal
        let val_err = DatabaseError::Validation(msg1.clone());
        assert_ne!(err1, val_err);

        // Test Sqlx errors (using same error)
        let sqlx_err1 = DatabaseError::Sqlx(sqlx::Error::RowNotFound);
        let sqlx_err2 = DatabaseError::Sqlx(sqlx::Error::RowNotFound);
        assert_eq!(sqlx_err1, sqlx_err2);

        // Test Migration errors
        let migrate_err1 = DatabaseError::Migration(sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound));
        let migrate_err2 = DatabaseError::Migration(sqlx::migrate::MigrateError::Execute(sqlx::Error::RowNotFound));
        assert_eq!(migrate_err1, migrate_err2);
    }
}

