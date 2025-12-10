# Comprehensive Tracing Instructions

This guide provides instructions for adding comprehensive tracing from DEBUG to WARN levels in the Personal Ledger Rust application. Follow these patterns to ensure consistent, useful telemetry across the codebase.

## Overview

The application uses the `tracing` crate with custom telemetry levels defined in `lib_telemetry`. Tracing levels from highest to lowest priority:

- **ERROR**: System errors that require immediate attention
- **WARN**: Potentially harmful situations or important notifications
- **INFO**: Informational messages about normal application flow
- **DEBUG**: Detailed information for troubleshooting
- **TRACE**: Very detailed execution information

## Function Instrumentation

### When to Instrument Functions

Instrument functions that:

- Perform database operations
- Handle business logic
- Make external API calls
- Process user input
- Have complex logic or multiple steps

### Instrumentation Pattern

```rust
/// Function documentation here
#[tracing::instrument(
    name = "Descriptive operation name",
    level = "debug",
    skip(parameter_to_skip),  // Skip large objects or sensitive data
    fields(
        parameter_name = %parameter_value,  // Include relevant parameters
        user_id = %user.id,                 // Extract fields from structs
        operation_type = "create"           // Static values
    ),
    err  // Include error information in spans
)]
pub async fn my_function(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<ReturnType, Error> {
    // Implementation
}
```

### Field Guidelines

- Use `%` for `Display` formatting (user-friendly)
- Use `?` for `Debug` formatting (detailed)
- Skip sensitive data (passwords, tokens, PII)
- Include IDs, names, and operation types
- Extract relevant fields from complex parameters

## Log Level Usage

### ERROR Level

Use for system errors that require immediate attention:

```rust
tracing::error!(
    error = %error,
    user_id = %user_id,
    operation = "user_registration",
    "Failed to create user account"
);
```

**When to use ERROR:**

- Database connection failures
- External service unavailability
- Data corruption
- Security violations
- System resource exhaustion

### WARN Level

Use for potentially harmful situations:

```rust
tracing::warn!(
    user_id = %user_id,
    attempt_count = %failed_attempts,
    ip_address = %client_ip,
    "Multiple failed login attempts detected"
);
```

**When to use WARN:**

- Rate limiting triggers
- Unusual patterns (multiple failures, timeouts)
- Deprecated API usage
- Configuration issues
- Performance degradation

### INFO Level

Use for important application events:

```rust
tracing::info!(
    user_id = %user_id,
    category_id = %category.id,
    category_name = %category.name,
    "Category updated successfully"
);
```

**When to use INFO:**

- Successful operations (create, update, delete)
- User authentication events
- Important state changes
- Business logic milestones
- Startup/shutdown events

### DEBUG Level

Use for troubleshooting information:

```rust
tracing::debug!(
    user_id = %user_id,
    query_params = ?search_params,
    result_count = %results.len(),
    execution_time_ms = %start_time.elapsed().as_millis(),
    "Executed category search query"
);
```

**When to use DEBUG:**

- Function entry/exit with parameters
- SQL query execution details
- Cache hits/misses
- Performance metrics
- Detailed operation steps

## Structured Logging Patterns

### Database Operations

```rust
#[tracing::instrument(
    name = "Database query execution",
    level = "debug",
    skip(pool),
    fields(
        table = "categories",
        operation = "select",
        query_type = "find_by_id"
    ),
    err
)]
pub async fn find_category_by_id(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    id: i64
) -> Result<Option<Category>, DatabaseError> {
    tracing::debug!(category_id = %id, "Starting category lookup");

    let result = sqlx::query_as!(
        Category,
        "SELECT * FROM categories WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(DatabaseError::from)?;

    match &result {
        Some(category) => {
            tracing::debug!(
                category_id = %category.id,
                category_name = %category.name,
                "Category found"
            );
        }
        None => {
            tracing::debug!(category_id = %id, "Category not found");
        }
    }

    Ok(result)
}
```

### Error Handling

```rust
#[tracing::instrument(
    name = "User authentication",
    level = "debug",
    skip(password),
    fields(
        username = %credentials.username,
        attempt_count = %attempt_count
    ),
    err
)]
pub async fn authenticate_user(
    &self,
    credentials: &UserCredentials,
    attempt_count: u32
) -> Result<User, AuthError> {
    // Authentication logic...

    if !password_valid {
        tracing::warn!(
            username = %credentials.username,
            attempt_count = %attempt_count,
            client_ip = %client_ip,
            "Invalid password provided"
        );
        return Err(AuthError::InvalidCredentials);
    }

    tracing::info!(
        username = %credentials.username,
        user_id = %user.id,
        "User authenticated successfully"
    );

    Ok(user)
}
```

### Business Logic Flow

```rust
#[tracing::instrument(
    name = "Process financial transaction",
    skip(transaction_data),
    fields(
        user_id = %user_id,
        transaction_type = %transaction.transaction_type,
        amount = %transaction.amount
    ),
    err
)]
pub async fn process_transaction(
    &self,
    user_id: Uuid,
    transaction: &TransactionData
) -> Result<TransactionResult, ProcessingError> {
    tracing::debug!("Starting transaction validation");

    // Validation logic...
    self.validate_transaction(transaction).await?;
    tracing::debug!("Transaction validation passed");

    // Balance check...
    let balance = self.check_balance(user_id).await?;
    tracing::debug!(current_balance = %balance, "Balance check completed");

    if balance < transaction.amount {
        tracing::warn!(
            user_id = %user_id,
            required_amount = %transaction.amount,
            available_balance = %balance,
            "Insufficient funds for transaction"
        );
        return Err(ProcessingError::InsufficientFunds);
    }

    // Process transaction...
    let result = self.execute_transaction(user_id, transaction).await?;
    tracing::info!(
        user_id = %user_id,
        transaction_id = %result.transaction_id,
        amount = %transaction.amount,
        "Transaction processed successfully"
    );

    Ok(result)
}
```

## Performance Considerations

### Avoid in Hot Paths

- Don't add DEBUG/TRACE logging in tight loops
- Use conditional compilation for expensive operations:

```rust
#[cfg(debug_assertions)]
tracing::debug!(expensive_debug_info = ?compute_expensive_debug_data());
```

### Span Management

- Keep span fields minimal and relevant
- Use `skip` for large parameters
- Close spans appropriately in async code

### Sensitive Data

- Never log passwords, tokens, or PII
- Use field skipping for sensitive parameters
- Consider data sanitation for user inputs

## Configuration

Tracing levels are configured through the telemetry system:

```rust
use lib_telemetry::{TelemetryConfig, TelemetryLevels};

// Configure for development (DEBUG level)
let config = TelemetryConfig {
    level: TelemetryLevels::DEBUG,
    // ... other config
};

// Configure for production (WARN level)
let prod_config = TelemetryConfig {
    level: TelemetryLevels::WARN,
    // ... other config
};
```

## Best Practices Checklist

- [ ] All public API functions are instrumented
- [ ] Database operations include relevant IDs and operation types
- [ ] Error conditions are logged at appropriate levels
- [ ] Sensitive data is never logged
- [ ] Performance-critical paths avoid expensive logging
- [ ] Span fields are minimal and relevant
- [ ] Log messages are clear and actionable
- [ ] Structured logging uses consistent field names
- [ ] Async operations properly propagate span context

## Common Patterns

### Request/Response Logging

```rust
tracing::info!(
    method = %request.method(),
    path = %request.path(),
    status = %response.status(),
    duration_ms = %start_time.elapsed().as_millis(),
    "HTTP request completed"
);
```

### Cache Operations

```rust
tracing::debug!(
    cache_key = %key,
    cache_hit = %was_hit,
    cache_ttl_seconds = %ttl.as_secs(),
    "Cache operation completed"
);
```

### Background Tasks

```rust
tracing::info!(
    task_name = "cleanup_expired_sessions",
    sessions_cleaned = %cleaned_count,
    duration_ms = %start_time.elapsed().as_millis(),
    "Background cleanup task completed"
);
```

Remember: Good tracing provides observability without impacting performance. Focus on the 80/20 rule - log the 20% of information that helps with 80% of debugging scenarios.
