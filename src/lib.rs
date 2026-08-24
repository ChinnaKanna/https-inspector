use serde::Serialize;
use std::collections::HashMap;
use worker::*;

#[derive(Serialize)]
struct RequestDetails {
    method: String,
    url: String,
    client_ip: String,
    user_agent: String,
    headers: HashMap<String, String>,
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let headers = req
        .headers()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();

    let details = RequestDetails {
        method: req.method().to_string(),
        url: req.url()?.to_string(),
        client_ip: req.headers().get("cf-connecting-ip")?.unwrap_or_default(),
        user_agent: req.headers().get("user-agent")?.unwrap_or_default(),
        headers,
    };

    Response::from_json(&details)
}