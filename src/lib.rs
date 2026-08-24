use serde::Serialize;
use std::collections::HashMap;
use worker::*;

#[derive(Serialize)]
struct RequestDetails {
    method: String,
    url: String,
    path: String,
    client_ip: String,
    user_agent: String,
    headers: HashMap<String, String>,
    header_count: usize,
    total_header_bytes: usize,
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?.to_string();
    let path = req.path();
    let method = req.method().to_string();

    let mut headers = HashMap::new();
    let mut total_header_bytes: usize = 0;
    for (key, value) in req.headers().into_iter() {
        total_header_bytes += key.len() + value.len();
        headers.insert(key, value);
    }
    let header_count = headers.len();

    let client_ip = req
        .headers()
        .get("cf-connecting-ip")?
        .unwrap_or_default();
    let user_agent = req.headers().get("user-agent")?.unwrap_or_default();

    let details = RequestDetails {
        method,
        url,
        path,
        client_ip,
        user_agent,
        headers,
        header_count,
        total_header_bytes,
    };

    Response::from_json(&details)
}
