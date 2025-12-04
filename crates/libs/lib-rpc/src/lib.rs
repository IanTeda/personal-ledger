//! lib-rpc - gRPC services and types for the personal ledger.
//!
//! This crate provides re-exports of generated protobuf types and gRPC clients/servers
//! for categories and utilities services. It serves as the main interface for interacting
//! with the personal ledger's gRPC APIs.
//!
//! ## Services
//!
//! - **CategoriesService**: Handles CRUD operations for financial categories.
//! - **UtilitiesService**: Provides utility operations like health checks.
//!
//! ## Usage
//!
//! Use the re-exported clients and servers to build gRPC clients or implement servers.
//! Message types are available for constructing requests and handling responses.

mod generated;

mod categories;

mod utilities;

// Re-export categories module to maintain flat API
pub use categories::*;

// Re-export utilities module to maintain flat API
pub use utilities::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_reexports() {
        // Test that categories types can be instantiated
        let category = Category {
            id: "test-id".to_string(),
            code: "TEST".to_string(),
            name: "Test Category".to_string(),
            description: Some("A test category".to_string()),
            url_slug: Some("test-category".to_string()),
            category_type: 1,
            color: Some("#FF0000".to_string()),
            icon: Some("test-icon".to_string()),
            is_active: true,
            created_on: None,
            updated_on: None,
        };

        let request = CategoryCreateRequest {
            category: Some(category.clone()),
        };

        // Basic assertions
        assert_eq!(category.code, "TEST");
        assert!(category.is_active);
        assert!(request.category.is_some());
    }

    #[test]
    fn test_utilities_reexports() {
        // Test that utilities types can be instantiated
        let ping_request = PingRequest {};

        let ping_response = PingResponse {
            message: "pong".to_string(),
        };

        // Basic assertions
        assert_eq!(ping_response.message, "pong");
        // PingRequest is empty, so just check it exists
        let _ = ping_request;
    }
}
