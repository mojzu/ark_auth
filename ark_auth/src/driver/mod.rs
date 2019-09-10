#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use crate::core::{Audit, AuditCreate, Csrf, Key, Service, User};
#[cfg(feature = "postgres")]
pub use crate::driver::postgres::*;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::*;

use chrono::{DateTime, Utc};
use diesel::result::Error as DieselResultError;
use diesel_migrations::RunMigrationsError;
use r2d2::Error as R2d2Error;
use uuid::Uuid;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "DriverError:DieselResult {}", _0)]
    DieselResult(#[fail(cause)] DieselResultError),

    #[fail(display = "DriverError:DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] RunMigrationsError),

    #[fail(display = "DriverError:R2d2 {}", _0)]
    R2d2(#[fail(cause)] R2d2Error),
}

impl From<DieselResultError> for DriverError {
    fn from(e: DieselResultError) -> Self {
        Self::DieselResult(e)
    }
}

impl From<RunMigrationsError> for DriverError {
    fn from(e: RunMigrationsError) -> Self {
        Self::DieselMigrations(e)
    }
}

impl From<R2d2Error> for DriverError {
    fn from(e: R2d2Error) -> Self {
        Self::R2d2(e)
    }
}

/// Driver result wrapper type.
pub type DriverResult<T> = Result<T, DriverError>;

/// Driver closure function type.
pub type DriverLockFn = Box<dyn FnOnce(&dyn Driver) -> ()>;

/// Driver interface trait.
pub trait Driver: Send + Sync {
    /// Return a boxed trait containing clone of self.
    fn box_clone(&self) -> Box<dyn Driver>;

    /// Run closure with an exclusive lock.
    fn exclusive_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()>;

    /// Run closure with a shared lock.
    fn shared_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()>;

    /// List audit logs where ID is less than.
    fn audit_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List audit logs where ID is greater than.
    fn audit_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List audit logs where ID is greater than and less than.
    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List audit logs where created datetime is less than.
    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List audit logs where created datetime is greater than.
    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List audit logs where created datetime is greater than and less than.
    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// Create one audit log.
    fn audit_create(&self, data: &AuditCreate) -> DriverResult<Audit>;

    /// Read one audit log by ID.
    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>>;

    /// Read audit metrics, returns array of counts for distinct audit paths.
    fn audit_read_metrics(&self, service_id_mask: Option<Uuid>)
        -> DriverResult<Vec<(String, i64)>>;

    /// Delete many audit logs by created at time.
    fn audit_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> DriverResult<usize>;

    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        service_id: Uuid,
    ) -> DriverResult<Csrf>;

    /// Read one CSRF key, value pair.
    fn csrf_read_by_key(&self, key: &str) -> DriverResult<Option<Csrf>>;

    /// Delete one CSRF key, value pair.
    fn csrf_delete_by_key(&self, key: &str) -> DriverResult<usize>;

    /// Delete many CSRF key, value pairs by time to live timestamp.
    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> DriverResult<usize>;

    /// List keys where ID is less than.
    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List keys where ID is greater than.
    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// Create key.
    fn key_create(
        &self,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        service_id: Option<Uuid>,
        user_id: Option<Uuid>,
    ) -> DriverResult<Key>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: Uuid) -> DriverResult<Option<Key>>;

    /// Read key by service and user ID.
    fn key_read_by_user_id(
        &self,
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
    ) -> DriverResult<Option<Key>>;

    /// Read key by root key value.
    fn key_read_by_root_value(&self, value: &str) -> DriverResult<Option<Key>>;

    /// Read key by service key value.
    fn key_read_by_service_value(&self, value: &str) -> DriverResult<Option<Key>>;

    /// Read key by service ID and user key value.
    fn key_read_by_user_value(&self, service_id: Uuid, value: &str) -> DriverResult<Option<Key>>;

    /// Update key by ID.
    fn key_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<Key>;

    /// Update many keys by user ID.
    fn key_update_many_by_user_id(
        &self,
        user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<usize>;

    /// Delete key by ID.
    fn key_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;

    /// Delete root keys.
    fn key_delete_root(&self) -> DriverResult<usize>;

    /// List services where ID is less than.
    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List services where ID is greater than.
    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// Create service.
    fn service_create(&self, is_enabled: bool, name: &str, url: &str) -> DriverResult<Service>;

    /// Read service by ID.
    fn service_read_by_id(&self, id: Uuid) -> DriverResult<Option<Service>>;

    /// Update service by ID.
    fn service_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<Service>;

    /// Delete service by ID.
    fn service_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;

    /// List users where ID is less than.
    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List users where ID is greater than.
    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List users where email is equal.
    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// Create user.
    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> DriverResult<User>;

    /// Read user by ID.
    fn user_read_by_id(&self, id: Uuid) -> DriverResult<Option<User>>;

    /// Read user by email address.
    fn user_read_by_email(&self, email: &str) -> DriverResult<Option<User>>;

    /// Update user by ID.
    fn user_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<User>;

    /// Update user email by ID.
    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> DriverResult<usize>;

    /// Update user password by ID.
    fn user_update_password_by_id(&self, id: Uuid, password_hash: &str) -> DriverResult<usize>;

    /// Delete user by ID.
    fn user_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;
}

impl Clone for Box<dyn Driver> {
    fn clone(&self) -> Box<dyn Driver> {
        self.box_clone()
    }
}
