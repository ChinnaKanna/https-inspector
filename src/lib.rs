use serde::Serialize;
use std::collections::HashMap;
use worker::*;

#[derive(Serialize)]
struct RequestDetails {
    method: String,
    url: String,
    path: String,
    query_params: HashMap<String, String>,
    client_ip: String,
    user_agent: String,
    headers: HashMap<String, String>,
    header_count: usize,
    total_header_bytes: usize,
    cf: Option<CfDetails>,
}

#[derive(Serialize)]
struct CfDetails {
    // Only properties that return Option<T> (no internal .unwrap())
    asn: Option<u32>,
    as_organization: Option<String>,
    city: Option<String>,
    continent: Option<String>,
    country: Option<String>,
    coordinates: Option<Coordinates>,
    postal_code: Option<String>,
    metro_code: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    verified_bot_category: Option<String>,
    bot_management: Option<BotManagementDetails>,
}

#[derive(Serialize)]
struct Coordinates {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize)]
struct BotManagementDetails {
    score: u32,
    static_resource: bool,
    verified_bot: bool,
    corporate_proxy: bool,
    ja4: Option<String>,
    ja3_hash: Option<String>,
    js_detection: Option<JsDetectionDetails>,
    detection_ids: Vec<u32>,
}

#[derive(Serialize)]
struct JsDetectionDetails {
    passed: bool,
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?.to_string();
    let path = req.path();
    let method = req.method().to_string();

    // Parse query parameters
    let query_params: HashMap<String, String> = req.url()
        .ok()
        .and_then(|u| u.query_pairs().into_owned().collect::<HashMap<_, _>>().into())
        .unwrap_or_default();

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

    // Only access Cf properties that return Option<T> (no internal .unwrap())
    let cf_details = req.cf().map(|cf| {
        let bot_management = cf.bot_management().map(|bm| {
            let js_detection = bm.js_detection().map(|js| JsDetectionDetails {
                passed: js.passed(),
            });

            BotManagementDetails {
                score: bm.score(),
                static_resource: bm.static_resource(),
                verified_bot: bm.verified_bot(),
                corporate_proxy: bm.corporate_proxy(),
                ja4: bm.ja4(),
                ja3_hash: bm.ja3_hash(),
                js_detection,
                detection_ids: bm.detection_ids(),
            }
        });

        let coordinates = cf.coordinates().map(|(lat, lon)| Coordinates {
            latitude: lat,
            longitude: lon,
        });

        CfDetails {
            asn: cf.asn(),
            as_organization: cf.as_organization(),
            city: cf.city(),
            continent: cf.continent(),
            country: cf.country(),
            coordinates,
            postal_code: cf.postal_code(),
            metro_code: cf.metro_code(),
            region: cf.region(),
            region_code: cf.region_code(),
            verified_bot_category: cf.verified_bot_category(),
            bot_management,
        }
    });

    let details = RequestDetails {
        method,
        url,
        path,
        query_params,
        client_ip,
        user_agent,
        headers,
        header_count,
        total_header_bytes,
        cf: cf_details,
    };

    Response::from_json(&details)
}
