
use lib_database as database;
use lib_domain as domain;

use crate::categories::proto;

pub async fn activate_category(
    service: &super::CategoriesService,
    request: tonic::Request<proto::CategoryActivateRequest>,
) -> Result<tonic::Response<proto::CategoryActivateResponse>, tonic::Status> {
    // Extract the inner request
    let activate_request = request.into_inner();

    // Parse the ID from string to RowID
    let category_id = match activate_request.id.parse::<domain::RowID>() {
        Ok(id) => id,
        Err(_) => {
            return Err(tonic::Status::invalid_argument("Invalid category ID format"));
        }
    };

    // Update the category's active status to true
    let updated_category = match database::Categories::update_active_status(category_id, true, service.database_ref()).await {
        Ok(category) => category,
        Err(database::DatabaseError::NotFound(_)) => {
            return Err(tonic::Status::not_found(format!("Category with ID '{}' not found", activate_request.id)));
        }
        Err(db_error) => {
            tracing::error!("Failed to activate category {}: {}", activate_request.id, db_error);
            return Err(tonic::Status::internal("Failed to activate category"));
        }
    };

    // Convert to RPC category and return response
    let rpc_category: proto::Category = updated_category.into();
    let response = proto::CategoryActivateResponse {
        category: Some(rpc_category),
    };

    Ok(tonic::Response::new(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;
    use lib_domain::{RowID, UrlSlug, HexColor, CategoryTypes};
    use lib_database::Categories;
    use sqlx::SqlitePool;
    use chrono::Utc;

    /// Helper function to create a mock category for testing.
    ///
    /// Generates realistic test data using the fake crate, ensuring
    /// optional fields are randomised probabilistically.
    fn create_mock_category() -> Categories {
        let name: String = fake::faker::lorem::en::Words(1..3).fake::<Vec<String>>().join(" ");
        let description = if Boolean(60).fake() {
            Some(fake::faker::lorem::en::Sentences(3..8).fake::<Vec<String>>().join(" "))
        } else {
            None
        };
        let url_slug = Some(UrlSlug::parse(name.replace(" ", "-")).unwrap());
        let color = if Boolean(70).fake() {
            Some(HexColor::parse("#FF5733").unwrap()) // Example fixed color for simplicity
        } else {
            None
        };
        let icon = if Boolean(50).fake() {
            Some(fake::faker::lorem::en::Word().fake::<String>())
        } else {
            None
        };

        Categories {
            id: RowID::new(),
            code: format!("{:03}.{:03}.{:03}",
                rand::random::<u8>() % 100,
                rand::random::<u8>() % 100,
                rand::random::<u8>() % 100,
            ),
            name,
            description,
            url_slug,
            category_type: CategoryTypes::Expense, // Default for simplicity
            color,
            icon,
            is_active: false, // Start inactive for activation tests
            created_on: Utc::now(),
            updated_on: Utc::now(),
        }
    }

    /// Helper function to insert a test category into the database.
    ///
    /// Converts complex types to strings for insertion, matching
    /// patterns from other test modules.
    async fn insert_test_category(pool: &SqlitePool, category: &Categories) -> RowID {
        // Create the categories table if it doesn't exist
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                url_slug TEXT,
                category_type TEXT NOT NULL,
                color TEXT,
                icon TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                created_on TEXT NOT NULL,
                updated_on TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        let id_str = category.id.to_string();
        let url_slug_str = category.url_slug.as_ref().map(|s| s.to_string());
        let category_type_str = category.category_type.as_str();
        let color_str = category.color.as_ref().map(|c| c.to_string());
        let created_on_str = category.created_on.to_rfc3339();
        let updated_on_str = category.updated_on.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO categories (
                id, code, name, description, url_slug, category_type,
                color, icon, is_active, created_on, updated_on
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            category.code,
            category.name,
            category.description,
            url_slug_str,
            category_type_str,
            color_str,
            category.icon,
            category.is_active,
            created_on_str,
            updated_on_str
        )
        .execute(pool)
        .await
        .unwrap();

        category.id
    }

    mod activation_logic {
        use super::*;

        /// Tests successful activation of an inactive category.
        ///
        /// Verifies that the service correctly activates a category
        /// and returns the updated instance.
        #[sqlx::test]
        async fn test_activate_inactive_category(pool: SqlitePool) {
            let mut category = create_mock_category();
            category.is_active = false;
            let id = insert_test_category(&pool, &category).await;

            // Assuming the activate service calls database update
            let result = Categories::update_active_status(id, true, &pool).await;
            assert!(result.is_ok());
            let updated = result.unwrap();
            assert_eq!(updated.id, id);
            assert!(updated.is_active);
        }

        /// Tests activation of an already active category.
        ///
        /// Ensures no errors occur and the category remains active.
        #[sqlx::test]
        async fn test_activate_already_active_category(pool: SqlitePool) {
            let mut category = create_mock_category();
            category.is_active = true;
            let id = insert_test_category(&pool, &category).await;

            let result = Categories::update_active_status(id, true, &pool).await;
            assert!(result.is_ok());
            let updated = result.unwrap();
            assert!(updated.is_active);
        }

        /// Tests activation with a nonexistent category ID.
        ///
        /// Verifies proper error handling for invalid IDs.
        #[sqlx::test]
        async fn test_activate_nonexistent_category(pool: SqlitePool) {
            // Create the table without inserting any category
            sqlx::query!(
                r#"
                CREATE TABLE IF NOT EXISTS categories (
                    id TEXT PRIMARY KEY,
                    code TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    url_slug TEXT,
                    category_type TEXT NOT NULL,
                    color TEXT,
                    icon TEXT,
                    is_active BOOLEAN NOT NULL DEFAULT 0,
                    created_on TEXT NOT NULL,
                    updated_on TEXT NOT NULL
                )
                "#,
            )
            .execute(&pool)
            .await
            .unwrap();

            let fake_id = RowID::mock();
            let result = Categories::update_active_status(fake_id, true, &pool).await;
            assert!(result.is_err());
            // Assuming DatabaseError::NotFound is returned
            match result.unwrap_err() {
                lib_database::DatabaseError::NotFound(msg) => {
                    assert!(msg.contains(&fake_id.to_string()));
                }
                err => panic!("Expected NotFound error, got {:?}", err),
            }
        }
    }

    mod edge_cases {
        use super::*;

        /// Tests activation with randomised category configurations.
        ///
        /// Runs multiple iterations to ensure robustness across
        /// different optional field combinations.
        #[sqlx::test]
        async fn test_activate_with_randomised_fields(pool: SqlitePool) {
            for _ in 0..10 {
                let mut category = create_mock_category();
                category.is_active = false;
                let id = insert_test_category(&pool, &category).await;

                let result = Categories::update_active_status(id, true, &pool).await;
                assert!(result.is_ok());
                let updated = result.unwrap();
                assert!(updated.is_active);
                // Ensure other fields are preserved
                assert_eq!(updated.name, category.name);
                assert_eq!(updated.code, category.code);
            }
        }

        /// Tests that timestamps are updated correctly on activation.
        ///
        /// Verifies updated_on is set to a later time.
        #[sqlx::test]
        async fn test_activation_updates_timestamp(pool: SqlitePool) {
            let mut category = create_mock_category();
            category.is_active = false;
            let original_updated = category.updated_on;
            let id = insert_test_category(&pool, &category).await;

            let result = Categories::update_active_status(id, true, &pool).await;
            assert!(result.is_ok());
            let updated = result.unwrap();
            assert!(updated.updated_on > original_updated);
        }
    }
}