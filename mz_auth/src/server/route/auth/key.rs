use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{path, AuthKeyBody, AuthKeyResponse},
    AuditData, AuditMeta, Auth, Key, ServerResult, ServerValidateFromValue,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::KEY)
        .service(web::resource(path::VERIFY).route(web::post().to_async(verify_handler)))
        .service(web::resource(path::REVOKE).route(web::post().to_async(revoke_handler)))
}

fn verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthKeyBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || {
                verify_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.key,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    key: String,
    audit_data: Option<AuditData>,
) -> ServerResult<AuthKeyResponse> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::key_verify(
                data.driver(),
                &service,
                &mut audit,
                key,
                audit_data.as_ref(),
            )
        })
        .map_err(Into::into)
        .map(|user_key| AuthKeyResponse { data: user_key })
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthKeyBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || {
                revoke_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.key,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    key: String,
    audit_data: Option<AuditData>,
) -> ServerResult<usize> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::key_revoke(
                data.driver(),
                &service,
                &mut audit,
                key,
                audit_data.as_ref(),
            )
        })
        .map_err(Into::into)
}
