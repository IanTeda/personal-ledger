//! # Categories Database Module
//!
//! Provides data access helpers, builders, and models for working with
//! category records in the persistence layer. The module exposes the
//! database representation of a category alongside utilities for creating
//! and inserting records during tests or data seeding.

#![allow(unused)] // For development only

mod builder;
mod delete;
mod find;
mod insert;
mod model;
mod update;

/// Database row model representing a persisted category.
pub use model::Categories;

/// Fluent builder for constructing `Category` instances in tests and fixtures.
pub use builder::CategoriesBuilder;