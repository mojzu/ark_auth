#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::DriverPostgres;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::DriverSqlite;

use crate::core::{
    Audit, AuditCreate, AuditList, CoreError, Csrf, CsrfCreate, CsrfDelete, Key, KeyCreate,
    KeyList, KeyRead, KeyUpdate, Service, ServiceCreate, ServiceList, ServiceUpdate, User,
    UserCreate, UserList, UserRead, UserUpdate,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "DriverError:Locked {:?}", _0)]
    Locked(DriverLock),

    #[fail(display = "DriverError:LockFn {}", _0)]
    LockFn(String),

    #[fail(display = "DriverError:DieselResult {}", _0)]
    DieselResult(#[fail(cause)] diesel::result::Error),

    #[fail(display = "DriverError:DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),

    #[fail(display = "DriverError:R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
}

impl From<CoreError> for DriverError {
    fn from(e: CoreError) -> Self {
        Self::LockFn(format!("{}", e))
    }
}

impl From<diesel::result::Error> for DriverError {
    fn from(e: diesel::result::Error) -> Self {
        Self::DieselResult(e)
    }
}

impl From<diesel_migrations::RunMigrationsError> for DriverError {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Self::DieselMigrations(e)
    }
}

impl From<r2d2::Error> for DriverError {
    fn from(e: r2d2::Error) -> Self {
        Self::R2d2(e)
    }
}

/// Driver result wrapper type.
pub type DriverResult<T> = Result<T, DriverError>;

/// Driver lock keys.
#[derive(Debug, Clone, Copy)]
pub enum DriverLock {
    Transaction = 1,
}

/// Driver closure function type.
pub type DriverLockFn = Box<dyn FnOnce(&dyn DriverIf) -> DriverResult<bool>>;

/// Driver interface trait.
pub trait DriverIf {
    /// Run closure with an exclusive lock.
    fn exclusive_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool>;

    /// Run closure with a shared lock.
    fn shared_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool>;

    // ---------------
    // Audit Functions
    // ---------------

    /// List audit logs.
    fn audit_list(
        &self,
        list: &AuditList,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>>;

    /// Create audit log.
    fn audit_create(&self, data: &AuditCreate) -> DriverResult<Audit>;

    /// Read audit log (optional).
    fn audit_read_opt(
        &self,
        id: &Uuid,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Option<Audit>>;

    /// Read audit metrics, returns array of counts for distinct audit types.
    fn audit_read_metrics(
        &self,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, i64)>>;

    /// Delete many audit logs.
    fn audit_delete(&self, created_at: &DateTime<Utc>) -> DriverResult<usize>;

    // --------------
    // CSRF Functions
    // --------------

    /// Create CSRF key, value pair with time to live in seconds. Key must be unique.
    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf>;

    /// Read CSRF key, value pair (optional).
    fn csrf_read_opt(&self, key: &str) -> DriverResult<Option<Csrf>>;

    /// Delete CSRF key, value pair(s).
    fn csrf_delete(&self, delete: &CsrfDelete) -> DriverResult<usize>;

    // -------------
    // Key Functions
    // -------------

    /// List keys.
    fn key_list(&self, list: &KeyList, service_id_mask: Option<&Uuid>) -> DriverResult<Vec<Key>>;

    /// Create key.
    fn key_create(&self, create: &KeyCreate) -> DriverResult<Key>;

    /// Read key (optional).
    fn key_read_opt(&self, read: &KeyRead) -> DriverResult<Option<Key>>;

    /// Update key.
    fn key_update(&self, id: &Uuid, update: &KeyUpdate) -> DriverResult<Key>;

    /// Update many keys by user ID.
    fn key_update_many(&self, user_id: &Uuid, update: &KeyUpdate) -> DriverResult<usize>;

    /// Delete key.
    fn key_delete(&self, id: &Uuid) -> DriverResult<usize>;

    // -----------------
    // Service Functions
    // -----------------

    /// List services.
    fn service_list(&self, list: &ServiceList) -> DriverResult<Vec<Service>>;

    /// Create service.
    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service>;

    /// Read service (optional).
    fn service_read_opt(&self, id: &Uuid) -> DriverResult<Option<Service>>;

    /// Update service.
    fn service_update(&self, id: &Uuid, update: &ServiceUpdate) -> DriverResult<Service>;

    /// Delete service.
    fn service_delete(&self, id: &Uuid) -> DriverResult<usize>;

    // --------------
    // User Functions
    // --------------

    /// List users.
    fn user_list(&self, list: &UserList) -> DriverResult<Vec<User>>;

    /// Create user.
    fn user_create(&self, create: &UserCreate) -> DriverResult<User>;

    /// Read user (optional).
    fn user_read_opt(&self, read: &UserRead) -> DriverResult<Option<User>>;

    /// Update user.
    fn user_update(&self, id: &Uuid, update: &UserUpdate) -> DriverResult<User>;

    /// Delete user.
    fn user_delete(&self, id: &Uuid) -> DriverResult<usize>;
}

/// Driver trait.
pub trait Driver: DriverIf + Send + Sync {
    /// Return a boxed trait containing clone of self.
    fn box_clone(&self) -> Box<dyn Driver>;

    /// Return a reference to driver interface.
    fn as_if(&self) -> &dyn DriverIf;
}

impl Clone for Box<dyn Driver> {
    fn clone(&self) -> Box<dyn Driver> {
        self.box_clone()
    }
}