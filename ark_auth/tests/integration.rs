mod support;

use ark_auth::client::{Error, RequestError};
use ark_auth::server::api::{
    AuditCreateBody, AuditListQuery, KeyListQuery, ServiceListQuery, UserListQuery,
};
use serde_json::Value;
use support::*;

const INVALID_UUID: &str = "5a044d9035334e95a60ac0338904d37c";
const INVALID_SERVICE_KEY: &str = "invalid-service-key";
const USER_NAME: &str = "user-name";
const USER_PASSWORD: &str = "user-name";
const KEY_NAME: &str = "key-name";

// TODO(test): Finish, improve tests.
// TODO(test): Password reset tests, SMTP testing using mailin_embedded?
// Service 2 cannot confirm reset password.
// Confirm reset password success.
// User password is updated.
// Cannot reuse token.

#[test]
#[ignore]
fn guide_api_key() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, None);
    let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    user_key_verify(&client, &user_key);
    client.auth_key_revoke(&user_key.key).unwrap();
    user_key_verify_bad_request(&client, &user_key.key);
}

#[test]
#[ignore]
fn guide_login() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    user_token_verify(&client, &user_token);
    let user_token = user_token_refresh(&client, &user_token);
    client.auth_token_revoke(&user_token.access_token).unwrap();

    let res = client
        .auth_token_verify(&user_token.refresh_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn guide_reset_password() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    client.auth_local_reset_password(&user_email).unwrap();
}

#[test]
#[ignore]
fn guide_oauth2_login() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    auth_microsoft_oauth2_request(&client);
}

#[test]
#[ignore]
fn api_ping_ok() {
    let client = client_create();
    let res = client.ping().unwrap();
    assert_eq!(res, Value::String("pong".to_owned()));
}

#[test]
#[ignore]
fn api_auth_local_login_forbidden() {
    let mut client = client_create();
    let user_email = email_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client
        .auth_local_login(&user_email, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_invalid_email() {
    let client = client_create();

    let res = client
        .auth_local_login("invalid-email", USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_invalid_password() {
    let client = client_create();
    let user_email = email_create();

    let res = client.auth_local_login(&user_email, "").unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_unknown_email() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let res = client
        .auth_local_login(&user_email, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_disabled_user() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let _user = user_create(&client, false, USER_NAME, &user_email, Some(USER_PASSWORD));

    let res = client
        .auth_local_login(&user_email, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_unknown_user_key() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let _user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));

    let res = client
        .auth_local_login(&user_email, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_incorrect_password() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    let res = client.auth_local_login(&user_email, "guests").unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

    client.options.set_authorisation(&service2_key.value);
    let res = client
        .auth_local_login(&user_email, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_login_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    let res = client.auth_local_login(&user_email, USER_PASSWORD).unwrap();
    assert_eq!(res.data.user_id, user.id);
}

#[test]
#[ignore]
fn api_auth_local_reset_password_forbidden() {
    let mut client = client_create();
    let user_email = email_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_local_reset_password(&user_email).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_local_reset_password_bad_request_invalid_email() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .auth_local_reset_password("invalid-email")
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_reset_password_ok_unknown_email() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    // Endpoint should not infer users existence.
    client.options.set_authorisation(&service_key.value);
    client.auth_local_reset_password(&user_email).unwrap();
}

#[test]
#[ignore]
fn api_auth_local_reset_password_ok_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

    // Endpoint should not infer users existence.
    client.options.set_authorisation(&service2_key.value);
    client.auth_local_reset_password(&user_email).unwrap();
}

#[test]
#[ignore]
fn api_auth_local_reset_password_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    client.auth_local_reset_password(&user_email).unwrap();
}

#[test]
#[ignore]
fn api_auth_local_reset_password_confirm_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client
        .auth_local_reset_password_confirm(INVALID_UUID, USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_local_reset_password_confirm_bad_request_invalid_token() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .auth_local_reset_password_confirm("", USER_PASSWORD)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_local_reset_password_confirm_bad_request_invalid_password() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .auth_local_reset_password_confirm(INVALID_UUID, "")
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_verify_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_key_verify(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_key_verify_bad_request_invalid_key() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_key_verify(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_verify_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

    client.options.set_authorisation(&service2_key.value);
    let res = client.auth_key_verify(&user_key.key).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_verify_bad_request_service_key() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_key_verify(&service_key.value).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_verify_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, None);
    let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    client.auth_key_verify(&user_key.key).unwrap();
}

#[test]
#[ignore]
fn api_auth_key_revoke_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_key_revoke(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_key_revoke_bad_request_invalid_key() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_key_revoke(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_revoke_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

    client.options.set_authorisation(&service2_key.value);
    let res = client.auth_key_revoke(&user_key.key).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_revoke_bad_request_service_key() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_key_revoke(&service_key.value).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_key_revoke_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, None);
    let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    client.auth_key_revoke(&user_key.key).unwrap();
    let res = client.auth_key_verify(&user_key.key).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_verify_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_token_verify(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_token_verify_bad_request_invalid_token() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_token_verify(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_verify_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    client.options.set_authorisation(&service2_key.value);
    let res = client
        .auth_token_verify(&user_token.access_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_verify_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    client.auth_token_verify(&user_token.access_token).unwrap();
}

#[test]
#[ignore]
fn api_auth_token_refresh_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_token_refresh(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_token_refresh_bad_request_invalid_token() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_token_refresh(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_refresh_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    client.options.set_authorisation(&service2_key.value);
    let res = client
        .auth_token_refresh(&user_token.refresh_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_refresh_bad_request_used_refresh_token() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    user_token_verify(&client, &user_token);
    let user_token2 = user_token_refresh(&client, &user_token);
    client.auth_token_verify(&user_token2.access_token).unwrap();

    let res = client
        .auth_token_refresh(&user_token.refresh_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_refresh_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    user_token_verify(&client, &user_token);
    let user_token = user_token_refresh(&client, &user_token);
    client.auth_token_verify(&user_token.access_token).unwrap();
}

#[test]
#[ignore]
fn api_auth_token_revoke_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.auth_token_revoke(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_auth_token_revoke_bad_request_invalid_token() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client.auth_token_revoke(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_revoke_bad_request_unknown_user_key_for_service() {
    let mut client = client_create();
    let (service1, service1_key) = service_key_create(&client);
    let (_service2, service2_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service1_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);
    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    client.options.set_authorisation(&service2_key.value);
    let res = client
        .auth_token_revoke(&user_token.refresh_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_auth_token_revoke_ok() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
    let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

    let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

    user_token_verify(&client, &user_token);
    client.auth_token_revoke(&user_token.access_token).unwrap();
    let res = client
        .auth_token_verify(&user_token.access_token)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_audit_id_list_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    client.options.set_authorisation(&service_key.value);

    client
        .audit_create(AuditCreateBody::new("/test/1", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/2", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/3", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/4", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/5", Value::Null, None, None))
        .unwrap();

    let res1 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: None,
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];

    let res2 = client
        .audit_list(AuditListQuery {
            gt: Some(r1_1.to_owned()),
            lt: None,
            created_gte: None,
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);

    let res3 = client
        .audit_list(AuditListQuery {
            gt: Some(r1_2.to_owned()),
            lt: None,
            created_gte: None,
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);

    let res4 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: Some(r3_5.to_owned()),
            created_gte: None,
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);

    let res5 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: Some(r4_4.to_owned()),
            created_gte: None,
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}

#[test]
#[ignore]
fn api_audit_created_list_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    client.options.set_authorisation(&service_key.value);

    let a1 = client
        .audit_create(AuditCreateBody::new("/test/1", Value::Null, None, None))
        .unwrap()
        .data;
    client
        .audit_create(AuditCreateBody::new("/test/2", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/3", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/4", Value::Null, None, None))
        .unwrap();
    client
        .audit_create(AuditCreateBody::new("/test/5", Value::Null, None, None))
        .unwrap();

    let res1 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: Some(a1.created_at.to_rfc3339()),
            created_lte: None,
            offset_id: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];
    assert_eq!(r1_1, &a1.id);
    let a1 = client.audit_read(&r1_1).unwrap().data;

    let res2 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: Some(a1.created_at.to_rfc3339()),
            created_lte: None,
            offset_id: Some(a1.id),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);
    let a2 = client.audit_read(&r2_2).unwrap().data;

    let res3 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: Some(a2.created_at.to_rfc3339()),
            created_lte: None,
            offset_id: Some(a2.id),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);
    let a5 = client.audit_read(&r3_5).unwrap().data;

    let res4 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: None,
            created_lte: Some(a5.created_at.to_rfc3339()),
            offset_id: Some(a5.id),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);
    let a4 = client.audit_read(&r4_4).unwrap().data;

    let res5 = client
        .audit_list(AuditListQuery {
            gt: None,
            lt: None,
            created_gte: None,
            created_lte: Some(a4.created_at.to_rfc3339()),
            offset_id: Some(a4.id),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}

#[test]
#[ignore]
fn api_key_list_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client
        .key_list(KeyListQuery {
            gt: None,
            lt: None,
            limit: None,
        })
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_key_list_bad_request_invalid_gt() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .key_list(KeyListQuery {
            gt: Some("".to_owned()),
            lt: None,
            limit: None,
        })
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_key_list_bad_request_invalid_lt() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .key_list(KeyListQuery {
            gt: None,
            lt: Some("".to_owned()),
            limit: None,
        })
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_key_list_bad_request_invalid_limit() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);

    client.options.set_authorisation(&service_key.value);
    let res = client
        .key_list(KeyListQuery {
            gt: None,
            lt: None,
            limit: Some("-1".to_owned()),
        })
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}

#[test]
#[ignore]
fn api_key_list_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, None);

    client
        .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
        .unwrap();
    client
        .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
        .unwrap();
    client
        .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
        .unwrap();
    client
        .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
        .unwrap();
    client
        .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
        .unwrap();

    let res1 = client
        .key_list(KeyListQuery {
            gt: None,
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];

    let res2 = client
        .key_list(KeyListQuery {
            gt: Some(r1_1.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);

    let res3 = client
        .key_list(KeyListQuery {
            gt: Some(r1_2.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);

    let res4 = client
        .key_list(KeyListQuery {
            gt: None,
            lt: Some(r3_5.to_owned()),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);

    let res5 = client
        .key_list(KeyListQuery {
            gt: None,
            lt: Some(r4_4.to_owned()),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}

#[test]
#[ignore]
fn api_key_create_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.key_create(true, KEY_NAME, None, None).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_key_read_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.key_read(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_service_list_ok() {
    let client = client_create();
    service_key_create(&client);
    service_key_create(&client);
    service_key_create(&client);
    service_key_create(&client);
    service_key_create(&client);

    let res1 = client
        .service_list(ServiceListQuery {
            gt: None,
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];

    let res2 = client
        .service_list(ServiceListQuery {
            gt: Some(r1_1.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);

    let res3 = client
        .service_list(ServiceListQuery {
            gt: Some(r1_2.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);

    let res4 = client
        .service_list(ServiceListQuery {
            gt: None,
            lt: Some(r3_5.to_owned()),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);

    let res5 = client
        .service_list(ServiceListQuery {
            gt: None,
            lt: Some(r4_4.to_owned()),
            limit: Some("3".to_owned()),
        })
        .unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}

#[test]
#[ignore]
fn api_service_read_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client.service_read(INVALID_UUID).unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_user_list_forbidden() {
    let mut client = client_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client
        .user_list(UserListQuery {
            gt: None,
            lt: None,
            limit: None,
            email_eq: None,
        })
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_user_list_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user1_email = email_create();
    let user2_email = email_create();
    let user3_email = email_create();
    let user4_email = email_create();
    let user5_email = email_create();

    client.options.set_authorisation(&service_key.value);
    user_create(&client, true, USER_NAME, &user1_email, None);
    user_create(&client, true, USER_NAME, &user2_email, None);
    user_create(&client, true, USER_NAME, &user3_email, None);
    user_create(&client, true, USER_NAME, &user4_email, None);
    user_create(&client, true, USER_NAME, &user5_email, None);

    let res1 = client
        .user_list(UserListQuery {
            gt: None,
            lt: None,
            limit: Some("3".to_owned()),
            email_eq: None,
        })
        .unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];

    let res2 = client
        .user_list(UserListQuery {
            gt: Some(r1_1.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
            email_eq: None,
        })
        .unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);

    let res3 = client
        .user_list(UserListQuery {
            gt: Some(r1_2.to_owned()),
            lt: None,
            limit: Some("3".to_owned()),
            email_eq: None,
        })
        .unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);

    let res4 = client
        .user_list(UserListQuery {
            gt: None,
            lt: Some(r3_5.to_owned()),
            limit: Some("3".to_owned()),
            email_eq: None,
        })
        .unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);

    let res5 = client
        .user_list(UserListQuery {
            gt: None,
            lt: Some(r4_4.to_owned()),
            limit: Some("3".to_owned()),
            email_eq: None,
        })
        .unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}

#[test]
#[ignore]
fn api_user_list_email_eq_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, USER_NAME, &user_email, None);

    let res = client
        .user_list(UserListQuery {
            gt: None,
            lt: None,
            limit: None,
            email_eq: Some(user.email),
        })
        .unwrap();
    assert_eq!(res.data.len(), 1);
    assert_eq!(res.data[0], user.id);
}

#[test]
#[ignore]
fn api_user_create_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    user_create(&client, true, USER_NAME, &user_email, None);
}

#[test]
#[ignore]
fn api_user_create_forbidden() {
    let mut client = client_create();
    let user_email = email_create();

    client.options.set_authorisation(INVALID_SERVICE_KEY);
    let res = client
        .user_create(true, USER_NAME, &user_email, None)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::Forbidden));
}

#[test]
#[ignore]
fn api_user_create_bad_request_duplicate_user_email() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    user_create(&client, true, USER_NAME, &user_email, None);

    let res = client
        .user_create(true, USER_NAME, &user_email, None)
        .unwrap_err();
    assert_eq!(res, Error::Request(RequestError::BadRequest));
}
