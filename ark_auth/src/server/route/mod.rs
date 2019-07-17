pub mod audit;
pub mod auth;
pub mod key;
pub mod service;
pub mod user;

use crate::core::AuditMeta;
use crate::server::api::path;
use crate::server::Error;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use futures::{future, Future};
use serde::Serialize;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::V1)
        .service(web::resource(path::PING).route(web::get().to(ping_handler)))
        .service(audit::route_v1_scope())
        .service(auth::route_v1_scope())
        .service(key::route_v1_scope())
        .service(service::route_v1_scope())
        .service(user::route_v1_scope())
}

pub fn route_service(configuration: &mut web::ServiceConfig) {
    configuration.service(route_v1_scope());
}

fn ping_handler() -> actix_web::Result<HttpResponse> {
    let body = r#"pong"#;
    Ok(HttpResponse::Ok().json(body))
}

/// Build audit meta from HTTP request.
pub fn request_audit_meta(req: &HttpRequest) -> future::FutureResult<AuditMeta, Error> {
    let connection_info = req.connection_info();
    let remote = connection_info.remote().ok_or_else(|| Error::BadRequest);

    let user_agent = req
        .headers()
        .get(http::header::USER_AGENT)
        .ok_or_else(|| Error::BadRequest)
        .and_then(|x| x.to_str().map_err(|_err| Error::BadRequest));

    let forwarded = req.headers().get(http::header::FORWARDED);
    let forwarded = if let Some(forwarded) = forwarded {
        forwarded
            .to_str()
            .map_err(|_err| Error::BadRequest)
            .map(Some)
    } else {
        Ok(None)
    };

    future::result(remote.and_then(|remote| {
        let user_agent = user_agent?;
        let forwarded = forwarded?;
        Ok(AuditMeta::new(
            user_agent,
            remote,
            forwarded.map(|x| x.to_owned()),
        ))
    }))
}

/// Route response empty handler.
pub fn route_response_empty<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(_res) => future::ok(HttpResponse::Ok().finish()),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response handler.
pub fn route_response_json<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().json(res)),
        Err(err) => future::ok(err.error_response()),
    }
}
