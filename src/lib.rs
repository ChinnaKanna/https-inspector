use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
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
    asn: Option<u32>,
    as_organization: Option<String>,
    city: Option<String>,
    colo: Option<String>,
    continent: Option<String>,
    country: Option<String>,
    http_protocol: Option<String>,
    coordinates: Option<Coordinates>,
    postal_code: Option<String>,
    metro_code: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    timezone: Option<String>,
    is_eu_country: Option<bool>,
    tls_version: Option<String>,
    tls_cipher: Option<String>,
    tls_client_auth: Option<TlsClientAuthDetails>,
    bot_management: Option<BotManagementDetails>,
    verified_bot_category: Option<String>,
    request_priority: Option<RequestPriorityDetails>,
}

#[derive(Serialize)]
struct Coordinates {
    latitude: f32,
    longitude: f32,
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
struct RequestPriorityDetails {
    weight: Option<usize>,
    exclusive: Option<bool>,
    group: Option<usize>,
    group_weight: Option<usize>,
}

/// Safely read a string property from a JsValue object
fn get_js_string(obj: &JsValue, key: &str) -> Option<String> {
    let val = js_sys::Reflect::get(obj, &key.into()).ok()?;
    if val.is_null() || val.is_undefined() {
        None
    } else {
        val.as_string()
    }
}

/// Safely read a number property from a JsValue object
fn get_js_u32(obj: &JsValue, key: &str) -> Option<u32> {
    let val = js_sys::Reflect::get(obj, &key.into()).ok()?;
    if val.is_null() || val.is_undefined() {
        None
    } else {
        val.as_f64().map(|v| v as u32)
    }
}

/// Safely read a boolean property from a JsValue object
fn get_js_bool(obj: &JsValue, key: &str) -> Option<bool> {
    let val = js_sys::Reflect::get(obj, &key.into()).ok()?;
    if val.is_null() || val.is_undefined() {
        None
    } else {
        val.as_bool()
    }
}

/// Safely read a sub-object from a JsValue
fn get_js_object(obj: &JsValue, key: &str) -> Option<JsValue> {
    let val = js_sys::Reflect::get(obj, &key.into()).ok()?;
    if val.is_null() || val.is_undefined() {
        None
    } else {
        Some(val)
    }
}

/// Get the cf properties from a web_sys::Request via wasm_bindgen
fn get_cf_from_request(req: &web_sys::Request) -> Option<JsValue> {
    // Use js_sys::Reflect to get the cf property from the request
    let cf = js_sys::Reflect::get(req, &"cf".into()).ok()?;
    if cf.is_null() || cf.is_undefined() {
        None
    } else {
        Some(cf)
    }
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?.to_string();
    let path = req.path();
    let method = req.method().to_string();

    // Parse query parameters
    let query_params: HashMap<String, String> = req
        .url()
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

    // Access the raw cf JS object from the underlying web_sys::Request
    let cf_details = get_cf_from_request(req.inner()).map(|cf_js| {
        // Read all properties safely via JS reflection
        let colo = get_js_string(&cf_js, "colo");
        let http_protocol = get_js_string(&cf_js, "httpProtocol");
        let tls_version = get_js_string(&cf_js, "tlsVersion");
        let tls_cipher = get_js_string(&cf_js, "tlsCipher");
        let city = get_js_string(&cf_js, "city");
        let country = get_js_string(&cf_js, "country");
        let continent = get_js_string(&cf_js, "continent");
        let postal_code = get_js_string(&cf_js, "postalCode");
        let metro_code = get_js_string(&cf_js, "metroCode");
        let region = get_js_string(&cf_js, "region");
        let region_code = get_js_string(&cf_js, "regionCode");
        let timezone = get_js_string(&cf_js, "timezone");
        let as_organization = get_js_string(&cf_js, "asOrganization");
        let asn = get_js_u32(&cf_js, "asn");
        let verified_bot_category = get_js_string(&cf_js, "verifiedBotCategory");

        // isEUCountry comes back as string "1" or null
        let is_eu_country = get_js_string(&cf_js, "isEUCountry").map(|v| v == "1");

        // Coordinates: latitude and longitude are separate string properties
        let coordinates = match (
            get_js_string(&cf_js, "latitude"),
            get_js_string(&cf_js, "longitude"),
        ) {
            (Some(lat_str), Some(lon_str)) => match (lat_str.parse(), lon_str.parse()) {
                (Ok(lat), Ok(lon)) => Some(Coordinates {
                    latitude: lat,
                    longitude: lon,
                }),
                _ => None,
            },
            _ => None,
        };

        // Bot Management
        let bot_management = get_js_object(&cf_js, "botManagement").map(|bm| {
            let score = get_js_u32(&bm, "score");
            let static_resource = get_js_bool(&bm, "staticResource");
            let verified_bot = get_js_bool(&bm, "verifiedBot");
            let corporate_proxy = get_js_bool(&bm, "corporateProxy");
            let ja4 = get_js_string(&bm, "ja4");
            let ja3_hash = get_js_string(&bm, "ja3Hash");
            let detection_ids = get_js_object(&bm, "detectionIds")
                .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_f64().map(|n| n as u32))
                        .collect()
                });

            let js_detection = get_js_object(&bm, "jsDetection").map(|js| JsDetectionDetails {
                passed: get_js_bool(&js, "passed"),
            });

            BotManagementDetails {
                score,
                static_resource,
                verified_bot,
                corporate_proxy,
                ja4,
                ja3_hash,
                js_detection,
                detection_ids,
            }
        });

        // TLS Client Auth
        let tls_client_auth = get_js_object(&cf_js, "tlsClientAuth").map(|auth| {
            TlsClientAuthDetails {
                cert_issuer_dn_legacy: get_js_string(&auth, "certIssuerDnLegacy"),
                cert_issuer_dn: get_js_string(&auth, "certIssuerDn"),
                cert_issuer_dn_rfc2253: get_js_string(&auth, "certIssuerDnRfc2253"),
                cert_subject_dn_legacy: get_js_string(&auth, "certSubjectDnLegacy"),
                cert_subject_dn: get_js_string(&auth, "certSubjectDn"),
                cert_subject_dn_rfc2253: get_js_string(&auth, "certSubjectDnRfc2253"),
                cert_verified: get_js_string(&auth, "certVerified"),
                cert_not_after: get_js_string(&auth, "certNotAfter"),
                cert_not_before: get_js_string(&auth, "certNotBefore"),
                cert_fingerprint_sha1: get_js_string(&auth, "certFingerprintSha1"),
                cert_fingerprint_sha256: get_js_string(&auth, "certFingerprintSha256"),
                cert_serial: get_js_string(&auth, "certSerial"),
                cert_presented: get_js_string(&auth, "certPresented"),
            }
        });

        // Request Priority - comes back as a string like "weight=256;exclusive=1"
        let request_priority = get_js_string(&cf_js, "requestPriority").map(|priority| {
            let mut weight = None;
            let mut exclusive = None;
            let mut group = None;
            let mut group_weight = None;

            for pair in priority.split(';') {
                let mut iter = pair.split('=');
                if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                    match key {
                        "weight" => weight = value.parse().ok(),
                        "exclusive" => exclusive = Some(value == "1"),
                        "group" => group = value.parse().ok(),
                        "group-weight" => group_weight = value.parse().ok(),
                        _ => {}
                    }
                }
            }

            RequestPriorityDetails {
                weight,
                exclusive,
                group,
                group_weight,
            }
        });

        CfDetails {
            asn,
            as_organization,
            city,
            colo,
            continent,
            country,
            http_protocol,
            coordinates,
            postal_code,
            metro_code,
            region,
            region_code,
            timezone,
            is_eu_country,
            tls_version,
            tls_cipher,
            tls_client_auth,
            bot_management,
            verified_bot_category,
            request_priority,
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
