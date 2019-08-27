//! # Driver Interface
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use crate::core::{Audit, AuditCreate, Csrf, Key, Service, User};
#[cfg(feature = "postgres")]
pub use crate::driver::postgres::PostgresDriver;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::SqliteDriver;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// ## Driver Errors
#[derive(Debug, Fail)]
pub enum DriverError {
    /// Diesel result error wrapper.
    #[fail(display = "DriverError:Diesel {}", _0)]
    Diesel(#[fail(cause)] diesel::result::Error),
    /// Diesel migrations error wrapper.
    #[fail(display = "DriverError:DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),
    /// R2d2 error wrapper.
    #[fail(display = "DriverError:R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
}

/// ## Driver Interface
pub trait Driver: Send + Sync {
    /// Return a boxed trait containing clone of self.
    fn box_clone(&self) -> Box<dyn Driver>;

    /// List audit logs where ID is less than.
    fn audit_list_where_id_lt(
        &self,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List audit logs where ID is greater than.
    fn audit_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List audit logs where ID is greater than and less than.
    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: &str,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List audit logs where created datetime is less than.
    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List audit logs where created datetime is greater than.
    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List audit logs where created datetime is greater than and less than.
    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// Create one audit log.
    fn audit_create(&self, data: &AuditCreate) -> Result<Audit, DriverError>;

    /// Read one audit log by ID.
    fn audit_read_by_id(
        &self,
        id: &str,
        service_id_mask: Option<&str>,
    ) -> Result<Option<Audit>, DriverError>;

    /// Read audit metrics, returns array of counts for distinct audit paths.
    fn audit_read_metrics(
        &self,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<(String, i64)>, DriverError>;

    /// Delete many audit logs by created at time.
    fn audit_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<usize, DriverError>;

    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        service_id: &str,
    ) -> Result<Csrf, DriverError>;

    /// Read one CSRF key, value pair.
    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, DriverError>;

    /// Delete one CSRF key, value pair.
    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, DriverError>;

    /// Delete many CSRF key, value pairs by time to live timestamp.
    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> Result<usize, DriverError>;

    /// List keys where ID is less than.
    fn key_list_where_id_lt(
        &self,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// List keys where ID is greater than.
    fn key_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, DriverError>;

    /// Create key.
    fn key_create(
        &self,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Key, DriverError>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, DriverError>;

    /// Read key by service and user ID.
    fn key_read_by_user_id(
        &self,
        service_id: &str,
        user_id: &str,
    ) -> Result<Option<Key>, DriverError>;

    /// Read key by root key value.
    fn key_read_by_root_value(&self, value: &str) -> Result<Option<Key>, DriverError>;

    /// Read key by service key value.
    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, DriverError>;

    /// Read key by service ID and user key value.
    fn key_read_by_user_value(
        &self,
        service_id: &str,
        value: &str,
    ) -> Result<Option<Key>, DriverError>;

    /// Update key by ID.
    fn key_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, DriverError>;

    /// Update many keys by user ID.
    fn key_update_many_by_user_id(
        &self,
        user_id: &str,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<usize, DriverError>;

    /// Delete key by ID.
    fn key_delete_by_id(&self, id: &str) -> Result<usize, DriverError>;

    /// Delete root keys.
    fn key_delete_root(&self) -> Result<usize, DriverError>;

    /// List services where ID is less than.
    fn service_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, DriverError>;

    /// List services where ID is greater than.
    fn service_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, DriverError>;

    /// Create service.
    fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> Result<Service, DriverError>;

    /// Read service by ID.
    fn service_read_by_id(&self, id: &str) -> Result<Option<Service>, DriverError>;

    /// Update service by ID.
    fn service_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, DriverError>;

    /// Delete service by ID.
    fn service_delete_by_id(&self, id: &str) -> Result<usize, DriverError>;

    /// List users where ID is less than.
    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError>;

    /// List users where ID is greater than.
    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError>;

    /// List users where email is equal.
    fn user_list_where_email_eq(
        &self,
        email_eq: &str,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError>;

    /// Create user.
    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, DriverError>;

    /// Read user by ID.
    fn user_read_by_id(&self, id: Uuid) -> Result<Option<User>, DriverError>;

    /// Read user by email address.
    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, DriverError>;

    /// Update user by ID.
    fn user_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, DriverError>;

    /// Update user email by ID.
    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> Result<usize, DriverError>;

    /// Update user password by ID.
    fn user_update_password_by_id(
        &self,
        id: Uuid,
        password_hash: &str,
    ) -> Result<usize, DriverError>;

    /// Delete user by ID.
    fn user_delete_by_id(&self, id: Uuid) -> Result<usize, DriverError>;
}

impl Clone for Box<dyn Driver> {
    fn clone(&self) -> Box<dyn Driver> {
        self.box_clone()
    }
}
