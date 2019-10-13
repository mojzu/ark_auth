use crate::{
    driver::postgres::schema::sso_audit, Audit, AuditCreate, AuditList, AuditListFilter,
    AuditListQuery, DriverResult,
};
use chrono::{DateTime, Utc};
use diesel::{dsl::sql, pg::Pg, prelude::*, sql_types::BigInt};
use serde_json::Value;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_audit"]
#[primary_key(id)]
pub struct ModelAudit {
    created_at: DateTime<Utc>,
    id: Uuid,
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
    type_: String,
    data: Value,
    key_id: Option<Uuid>,
    service_id: Option<Uuid>,
    user_id: Option<Uuid>,
    user_key_id: Option<Uuid>,
}

impl From<ModelAudit> for Audit {
    fn from(audit: ModelAudit) -> Self {
        Self {
            created_at: audit.created_at,
            id: audit.id,
            user_agent: audit.user_agent,
            remote: audit.remote,
            forwarded: audit.forwarded,
            type_: audit.type_,
            data: audit.data,
            key_id: audit.key_id,
            service_id: audit.service_id,
            user_id: audit.user_id,
            user_key_id: audit.user_key_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_audit"]
struct ModelAuditInsert<'a> {
    created_at: &'a DateTime<Utc>,
    id: &'a Uuid,
    user_agent: &'a str,
    remote: &'a str,
    forwarded: Option<&'a str>,
    type_: &'a str,
    data: &'a Value,
    key_id: Option<&'a Uuid>,
    service_id: Option<&'a Uuid>,
    user_id: Option<&'a Uuid>,
    user_key_id: Option<&'a Uuid>,
}

impl<'a> ModelAuditInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a AuditCreate) -> Self {
        Self {
            created_at: now,
            id,
            user_agent: create.meta.user_agent(),
            remote: create.meta.remote(),
            forwarded: create.meta.forwarded(),
            type_: create.type_,
            data: create.data,
            key_id: create.key_id,
            service_id: create.service_id,
            user_id: create.user_id,
            user_key_id: create.user_key_id,
        }
    }
}

impl ModelAudit {
    pub fn list(conn: &PgConnection, list: &AuditList) -> DriverResult<Vec<Audit>> {
        match list.query {
            AuditListQuery::CreatedLe(le, limit, offset_id) => Self::list_where_created_le(
                conn,
                le,
                *limit,
                offset_id,
                list.filter,
                list.service_id_mask,
            ),
            AuditListQuery::CreatedGe(ge, limit, offset_id) => Self::list_where_created_ge(
                conn,
                ge,
                *limit,
                offset_id,
                list.filter,
                list.service_id_mask,
            ),
            AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id) => {
                Self::list_where_created_le_and_ge(
                    conn,
                    le,
                    ge,
                    *limit,
                    offset_id,
                    list.filter,
                    list.service_id_mask,
                )
            }
        }
    }

    pub fn create(conn: &PgConnection, create: &AuditCreate) -> DriverResult<Audit> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelAuditInsert::from_create(&now, &id, create);
        diesel::insert_into(sso_audit::table)
            .values(&value)
            .get_result::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_opt(
        conn: &PgConnection,
        id: &Uuid,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Option<Audit>> {
        match service_id_mask {
            Some(service_id_mask) => sso_audit::table
                .filter(
                    sso_audit::dsl::service_id
                        .eq(service_id_mask)
                        .and(sso_audit::dsl::id.eq(id)),
                )
                .get_result::<ModelAudit>(conn),
            None => sso_audit::table
                .filter(sso_audit::dsl::id.eq(id))
                .get_result::<ModelAudit>(conn),
        }
        .optional()
        .map_err(Into::into)
        .map(|x| x.map(Into::into))
    }

    pub fn read_metrics(
        conn: &PgConnection,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        match service_id_mask {
            Some(service_id_mask) => sso_audit::table
                .select((sso_audit::dsl::type_, sql::<BigInt>("count(*)")))
                .filter(sso_audit::dsl::created_at.gt(from))
                .group_by(sso_audit::dsl::type_)
                .filter(sso_audit::dsl::service_id.eq(service_id_mask))
                .order(sso_audit::dsl::type_.asc())
                .load(conn),
            None => sso_audit::table
                .select((sso_audit::dsl::type_, sql::<BigInt>("count(*)")))
                .filter(sso_audit::dsl::created_at.gt(from))
                .group_by(sso_audit::dsl::type_)
                .order(sso_audit::dsl::type_.asc())
                .load(conn),
        }
        .map_err(Into::into)
    }

    pub fn delete(conn: &PgConnection, created_at: &DateTime<Utc>) -> DriverResult<usize> {
        diesel::delete(sso_audit::table.filter(sso_audit::dsl::created_at.le(created_at)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn list_where_created_le(
        conn: &PgConnection,
        le: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_inner(conn, le, limit, offset, filter, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, audit) in res.iter().enumerate() {
                        if &audit.id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_le_inner(
                                conn,
                                le,
                                limit,
                                offset,
                                filter,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    fn list_where_created_le_inner(
        conn: &PgConnection,
        created_le: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(sso_audit::dsl::created_at.le(created_le))
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.desc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn list_where_created_ge(
        conn: &PgConnection,
        ge: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_ge_inner(conn, ge, limit, offset, filter, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, audit) in res.iter().enumerate() {
                        if &audit.id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_ge_inner(
                                conn,
                                ge,
                                limit,
                                offset,
                                filter,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
    }

    fn list_where_created_ge_inner(
        conn: &PgConnection,
        created_ge: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(sso_audit::dsl::created_at.ge(created_ge))
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.asc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn list_where_created_le_and_ge(
        conn: &PgConnection,
        le: &DateTime<Utc>,
        ge: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_and_ge_inner(
            conn,
            le,
            ge,
            limit,
            offset,
            filter,
            service_id_mask,
        )
        .and_then(|res| {
            if let Some(offset_id) = offset_id {
                for (i, audit) in res.iter().enumerate() {
                    if &audit.id == offset_id {
                        let offset: i64 = (i + 1).try_into().unwrap();
                        return ModelAudit::list_where_created_le_and_ge_inner(
                            conn,
                            le,
                            ge,
                            limit,
                            offset,
                            filter,
                            service_id_mask,
                        );
                    }
                }
            }
            Ok(res)
        })
    }

    fn list_where_created_le_and_ge_inner(
        conn: &PgConnection,
        created_le: &DateTime<Utc>,
        created_ge: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(
                sso_audit::dsl::created_at
                    .ge(created_ge)
                    .and(sso_audit::dsl::created_at.le(created_le)),
            )
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.asc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn boxed_query_filter<'a>(
        mut query: sso_audit::BoxedQuery<'a, Pg>,
        filter: &'a AuditListFilter,
        service_id_mask: Option<&'a Uuid>,
    ) -> sso_audit::BoxedQuery<'a, Pg> {
        use diesel::dsl::any;

        if let Some(id) = &filter.id {
            let id: Vec<Uuid> = id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::id.eq(any(id)));
        }
        if let Some(type_) = &filter.type_ {
            let type_: Vec<String> = type_.to_vec();
            query = query.filter(sso_audit::dsl::type_.eq(any(type_)));
        }
        if let Some(service_id) = &filter.service_id {
            let service_id: Vec<Uuid> = service_id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::service_id.eq(any(service_id)));
        }
        if let Some(user_id) = &filter.user_id {
            let user_id: Vec<Uuid> = user_id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::user_id.eq(any(user_id)));
        }
        if let Some(service_id_mask) = service_id_mask {
            query = query.filter(sso_audit::dsl::service_id.eq(service_id_mask));
        }

        query
    }
}