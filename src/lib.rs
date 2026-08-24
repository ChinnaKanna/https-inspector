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
    let headers = req
        .headers()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();

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

        let bot_management = cf.bot_management().map(|bm| BotManagementDetails {
            score: Some(bm.score()),
            static_resource: Some(bm.static_resource()),
            verified_bot: Some(bm.verified_bot()),
            corporate_proxy: Some(bm.corporate_proxy()),
            ja4: bm.ja4(),
            ja3_hash: bm.ja3_hash(),
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

        CfDetails {
            asn: cf.asn(),
            as_organization: cf.as_organization(),
            city: cf.city(),
            colo: Some(cf.colo()),
            continent: cf.continent(),
            country: cf.country(),
            http_protocol: Some(cf.http_protocol()),
            coordinates,
            postal_code: cf.postal_code(),
            metro_code: cf.metro_code(),
            region: cf.region(),
            region_code: cf.region_code(),
            timezone: Some(cf.timezone_name()),
            is_eu_country: Some(cf.is_eu_country()),
            tls_version: Some(cf.tls_version()),
            tls_cipher: Some(cf.tls_cipher()),
            tls_client_auth,
            bot_management,
            verified_bot_category: cf.verified_bot_category(),
            request_priority,
        }
    });

    let details = RequestDetails {
        method: req.method().to_string(),
        url: req.url()?.to_string(),
        client_ip: req
            .headers()
            .get("cf-connecting-ip")?
            .unwrap_or_default(),
        user_agent: req.headers().get("user-agent")?.unwrap_or_default(),
        headers,
        cf: cf_details,
    };

    Response::from_json(&details)
}
