use headers::{Authorization, Header, authorization::Bearer};
use http::{StatusCode, header};
use worker::{Error, Fetch, Headers, Request, RequestInit, Response, Result, RouteContext, Router};

pub fn new_router() -> Router<'static, ()> {
    Router::new()
        .get_async("/:user_id.svg", get_user_activity_badge)
        .put_async("/", update_user_activity)
        .delete_async("/", delete_user_activity)
}

async fn get_user_activity_badge(
    request: Request,
    route_context: RouteContext<()>,
) -> Result<Response> {
    let user_id = route_context
        .param("user_id.svg")
        .ok_or("invalid user id")?
        .trim_end_matches(".svg");

    let user_stub = route_context
        .durable_object("USER")
        .map_err(|error| format!("invalid user durable object: {error}"))?
        .id_from_name(user_id)
        .map_err(|error| format!("failed to load durable object by user id: {error}"))?
        .get_stub()
        .map_err(|error| format!("failed to get durable object stub: {error}"))?;

    user_stub.fetch_with_request(request).await
}

async fn update_user_activity(
    request: Request,
    route_context: RouteContext<()>,
) -> Result<Response> {
    let user_id = authorization(&request, &route_context).await?;

    let user_stub = route_context
        .durable_object("USER")?
        .id_from_name(&user_id)?
        .get_stub()?;

    user_stub.fetch_with_request(request).await
}

async fn delete_user_activity(
    request: Request,
    route_context: RouteContext<()>,
) -> Result<Response> {
    let user_id = authorization(&request, &route_context).await?;

    let user_stub = route_context
        .durable_object("USER")?
        .id_from_name(&user_id)?
        .get_stub()?;

    user_stub.fetch_with_request(request).await
}

async fn authorization(request: &Request, route_context: &RouteContext<()>) -> Result<String> {
    let authorization = {
        let header_value = request
            .headers()
            .get(header::AUTHORIZATION.as_str())?
            .ok_or("invalid authorization header")?;

        Authorization::<Bearer>::decode(&mut [header_value.try_into()?].iter())
            .map_err(|error| format!("invalid authorization header: {}", error))
    }?;

    let cache = route_context.kv("CACHE")?;

    let token_key = format!("tokens/{}", authorization.token());

    match cache.get(&token_key).text().await? {
        None => {
            let mut request_init = RequestInit::default();
            request_init.with_headers({
                let mut headers = Headers::default();
                headers.set(header::USER_AGENT.as_str(), "MakaTime")?;

                let mut authorization_header_values = vec![];
                authorization.encode(&mut authorization_header_values);

                for authorization_header_value in authorization_header_values {
                    headers.append(
                        header::AUTHORIZATION.as_str(),
                        authorization_header_value.to_str().unwrap(),
                    )?;
                }

                headers
            });

            let request = Request::new_with_init("https://api.github.com/user", &request_init)?;

            let mut response = Fetch::Request(request.clone().unwrap()).send().await?;
            if response.status_code() != StatusCode::OK {
                return Err(Error::from("invalid authorization token"));
            }

            let response = response.json::<serde_json::Value>().await?;
            let user_id = response.get("login").unwrap().as_str().unwrap();

            cache
                .put(&token_key, user_id)?
                .expiration_ttl(60)
                .execute()
                .await?;

            Ok(user_id.to_owned())
        }
        Some(user_id) => Ok(user_id.to_owned()),
    }
}
