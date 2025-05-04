use crate::Activity;

use base64ct::{Base64, Encoding};
use http::{HeaderMap, HeaderValue, StatusCode, header};
use spacebadgers::{BadgeBuilder, icons::EVA_ICONS_FILL};
use worker::{Env, Error, Method, Request, Response, ResponseBuilder, Result, State};
use worker_macros::durable_object;

#[durable_object]
pub struct User {
    state: State,
}

#[durable_object]
impl DurableObject for User {
    fn new(state: State, _: Env) -> Self {
        Self { state }
    }

    async fn fetch(&mut self, request: Request) -> Result<Response> {
        match request.method() {
            Method::Get => self.get_activity_badge().await,
            Method::Put => self.update_activity(request).await,
            Method::Delete => self.delete_activity().await,
            _ => Err(Error::RouteNoDataError),
        }
    }
}

impl User {
    async fn get_activity_badge(&self) -> Result<Response> {
        let activity = self
            .state
            .storage()
            .get::<Activity>("activity")
            .await
            .map(Some)
            .unwrap_or_default();

        let badge_builder = match activity {
            None => BadgeBuilder::new()
                .icon(format!(
                    "data:image/svg+xml;base64,{}",
                    Base64::encode_string(EVA_ICONS_FILL.icons["eva-moon"].as_bytes())
                ))
                .label("Sleeping".to_owned())
                .status("Now"),
            Some(activity) => BadgeBuilder::new()
                .label(format!("Using {}", activity.name))
                .optional_icon(activity.icon)
                .status("Now"),
        };

        Ok(Response::builder()
            .with_headers({
                let mut header_map = HeaderMap::new();
                header_map.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("image/svg+xml"),
                );

                header_map.into()
            })
            .fixed(badge_builder.build().svg().into_bytes()))
    }

    async fn update_activity(&self, mut request: Request) -> Result<Response> {
        let activity = request.json::<Activity>().await?;
        self.state.storage().put("activity", activity).await?;

        Ok(ResponseBuilder::new()
            .with_status(StatusCode::NO_CONTENT.as_u16())
            .empty())
    }

    async fn delete_activity(&self) -> Result<Response> {
        self.state.storage().delete("activity").await?;

        Ok(ResponseBuilder::new()
            .with_status(StatusCode::NO_CONTENT.as_u16())
            .empty())
    }
}
