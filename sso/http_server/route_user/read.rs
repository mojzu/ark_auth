use crate::http_server::internal::*;

#[api_v2_operation(summary = "Read users")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestUserRead>,
) -> HttpResult<Json<ResponseUserMany>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.user_read(&client, body).await;

        server.response_json(res)
    })
}
