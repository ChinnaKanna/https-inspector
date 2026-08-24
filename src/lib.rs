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
    cf: Option<CfDetails>,
}

#[derive(Serialize)]
struct CfDetails {
    // Safe properties (return Option, no internal .unwrap())
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

    // Bot Management (all safe)
    bot_management: Option<BotManagementDetails>,

    // TLS Client Authentication (safe)
    tls_client_auth: Option<TlsClientAuthDetails>,

    // Request priority (safe)
    request_priority: Option<RequestPriorityDetails>,

    // Host metadata (safe)
    host_metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct Coordinates {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize)]
struct BotManagementDetails {
    score: Option<u32>,
    static_resource: Option<bool>,
    verified_bot: Option<bool>,
    corporate_proxy: Option<bool>,
    ja4: Option<String>,
    ja3_hash: Option<String>,
    js_detection: Option<JsDetectionDetails>,
    detection_ids: Option<Vec<u32>>,
}

#[derive(Serialize)]
struct JsDetectionDetails {
    passed: Option<bool>,
}

#[derive(Serialize)]
struct TlsClientAuthDetails {
    cert_issuer_dn_legacy: Option<String>,
    cert_issuer_dn: Option<String>,
    cert_issuer_dn_rfc2253: Option<String>,
    cert_subject_dn_legacy: Option<String>,
    cert_subject_dn: Option<String>,
    cert_subject_dn_rfc2253: Option<String>,
    cert_verified: Option<String>,
    cert_not_after: Option<String>,
    cert_not_before: Option<String>,
    cert_fingerprint_sha1: Option<String>,
    cert_fingerprint_sha256: Option<String>,
    cert_serial: Option<String>,
    cert_presented: Option<String>,
}

#[derive(Serialize)]
struct RequestPriorityDetails {
    weight: Option<usize>,
    exclusive: Option<bool>,
    group: Option<usize>,
    group_weight: Option<usize>,
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?.to_string();
    let path = req.path();
    let method = req.method().to_string();

    // Collect headers and compute sizes
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

    // Extract Cloudflare cf properties
    // Only access properties that return Option<T> (no internal .unwrap())
    let cf_details = req.cf().map(|cf| {
        let tls_client_auth = cf.tls_client_auth().map(|auth| TlsClientAuthDetails {
            cert_issuer_dn_legacy: Some(auth.cert_issuer_dn_legacy()),
            cert_issuer_dn: Some(auth.cert_issuer_dn()),
            cert_issuer_dn_rfc2253: Some(auth.cert_issuer_dn_rfc2253()),
            cert_subject_dn_legacy: Some(auth.cert_subject_dn_legacy()),
            cert_subject_dn: Some(auth.cert_subject_dn()),
            cert_subject_dn_rfc2253: Some(auth.cert_subject_dn_rfc2253()),
            cert_verified: Some(auth.cert_verified()),
            cert_not_after: Some(auth.cert_not_after()),
            cert_not_before: Some(auth.cert_not_before()),
            cert_fingerprint_sha1: Some(auth.cert_fingerprint_sha1()),
            cert_fingerprint_sha256: Some(auth.cert_fingerprint_sha256()),
            cert_serial: Some(auth.cert_serial()),
            cert_presented: Some(auth.cert_presented()),
        });

        let bot_management = cf.bot_management().map(|bm| {
            let js_detection = bm.js_detection().map(|js| JsDetectionDetails {
                passed: Some(js.passed()),
            });

            BotManagementDetails {
                score: Some(bm.score()),
                static_resource: Some(bm.static_resource()),
                verified_bot: Some(bm.verified_bot()),
                corporate_proxy: Some(bm.corporate_proxy()),
                ja4: bm.ja4(),
                ja3_hash: bm.ja3_hash(),
                js_detection,
                detection_ids: Some(bm.detection_ids()),
            }
        });

        let coordinates = cf.coordinates().map(|(lat, lon)| Coordinates {
            latitude: lat,
            longitude: lon,
        });

        let request_priority = cf.request_priority().map(|rp| RequestPriorityDetails {
            weight: Some(rp.weight),
            exclusive: Some(rp.exclusive),
            group: Some(rp.group),
            group_weight: Some(rp.group_weight),
        });

        let host_metadata = cf.host_metadata::<serde_json::Value>().ok().flatten();

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
            tls_client_auth,
            request_priority,
            host_metadata,
        }
    });

    let details = RequestDetails {
        method,
        url,
        path,
        client_ip,
        user_agent,
        headers,
        header_count,
        total_header_bytes,
        cf: cf_details,
    };

    Response::from_json(&details)
}
