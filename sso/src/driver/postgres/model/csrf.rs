use crate::{
    csrf::{Csrf, CsrfCreate},
    driver::postgres::schema::sso_csrf,
    DriverResult,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_csrf"]
#[primary_key(key)]
pub struct ModelCsrf {
    created_at: DateTime<Utc>,
    key: String,
    value: String,
    ttl: DateTime<Utc>,
    service_id: Uuid,
}

impl From<ModelCsrf> for Csrf {
    fn from(csrf: ModelCsrf) -> Self {
        Self {
            created_at: csrf.created_at,
            key: csrf.key,
            value: csrf.value,
            ttl: csrf.ttl,
            service_id: csrf.service_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_csrf"]
struct ModelCsrfInsert<'a> {
    created_at: &'a DateTime<Utc>,
    key: &'a str,
    value: &'a str,
    ttl: &'a DateTime<Utc>,
    service_id: &'a Uuid,
}

impl<'a> ModelCsrfInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, create: &'a CsrfCreate) -> Self {
        Self {
            created_at: now,
            key: &create.key,
            value: &create.value,
            ttl: &create.ttl,
            service_id: &create.service_id,
        }
    }
}

impl ModelCsrf {
    pub fn create(conn: &PgConnection, create: &CsrfCreate) -> DriverResult<Csrf> {
        let now = Utc::now();
        let value = ModelCsrfInsert::from_create(&now, create);
        diesel::insert_into(sso_csrf::table)
            .values(&value)
            .get_result::<ModelCsrf>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read(conn: &PgConnection, key: &str) -> DriverResult<Option<Csrf>> {
        Self::delete_by_ttl(conn)?;

        let csrf = Self::read_inner(conn, key)?;
        if csrf.is_some() {
            Self::delete_by_key(conn, key)?;
        }
        Ok(csrf)
    }

    fn read_inner(conn: &PgConnection, key: &str) -> DriverResult<Option<Csrf>> {
        sso_csrf::table
            .filter(sso_csrf::dsl::key.eq(key))
            .get_result::<ModelCsrf>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn delete_by_key(conn: &PgConnection, key: &str) -> DriverResult<usize> {
        diesel::delete(sso_csrf::table.filter(sso_csrf::dsl::key.eq(key)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn delete_by_ttl(conn: &PgConnection) -> DriverResult<usize> {
        let now = Utc::now();
        diesel::delete(sso_csrf::table.filter(sso_csrf::dsl::ttl.le(now)))
            .execute(conn)
            .map_err(Into::into)
    }
}
