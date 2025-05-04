mod objects;
mod routes;

use crate::routes::new_router;

use serde::{Deserialize, Serialize};
use worker::{Context, Env, Request, Response, Result};
use worker_macros::event;

#[derive(Debug, Serialize, Deserialize)]
struct Activity {
    icon: Option<String>,
    name: String,
}

#[event(fetch)]
async fn fetch(request: Request, env: Env, _: Context) -> Result<Response> {
    new_router().run(request, env).await
}
