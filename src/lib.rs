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
    cf: CfDetails,
}

#[derive(Serialize)]
struct CfDetails {
    asn: Option<u32>,
    as_organization: Option<String>,
    city: Option<String>,
    colo: String,
    continent: Option<String>,
    country: Option<String>,
    http_protocol: String,
    coordinates: Option<Coordinates>,
    postal_code: Option<String>,
    metro_code: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    timezone: String,
    is_eu_country: bool,
    tls_version: String,
    tls_cipher: String,
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
    cert_issuer_dn_legacy: String,
    cert_issuer_dn: String,
    cert_issuer_dn_rfc2253: String,
    cert_subject_dn_legacy: String,
    cert_subject_dn: String,
    cert_subject_dn_rfc2253: String,
    cert_verified: String,
    cert_not_after: String,
    cert_not_before: String,
    cert_fingerprint_sha1: String,
    cert_fingerprint_sha256: String,
    cert_serial: String,
    cert_presented: String,
}

#[derive(Serialize)]
struct BotManagementDetails {
    score: u32,
    static_resource: bool,
    verified_bot: bool,
    corporate_proxy: bool,
    ja4: Option<String>,
    ja3_hash: Option<String>,
}

#[derive(Serialize)]
struct RequestPriorityDetails {
    weight: usize,
    exclusive: bool,
    group: usize,
    group_weight: usize,
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let headers = req
        .headers()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();

    let cf = req.cf().expect("cf object should be available");

    let tls_client_auth = cf.tls_client_auth().map(|auth| TlsClientAuthDetails {
        cert_issuer_dn_legacy: auth.cert_issuer_dn_legacy(),
        cert_issuer_dn: auth.cert_issuer_dn(),
        cert_issuer_dn_rfc2253: auth.cert_issuer_dn_rfc2253(),
        cert_subject_dn_legacy: auth.cert_subject_dn_legacy(),
        cert_subject_dn: auth.cert_subject_dn(),
        cert_subject_dn_rfc2253: auth.cert_subject_dn_rfc2253(),
        cert_verified: auth.cert_verified(),
        cert_not_after: auth.cert_not_after(),
        cert_not_before: auth.cert_not_before(),
        cert_fingerprint_sha1: auth.cert_fingerprint_sha1(),
        cert_fingerprint_sha256: auth.cert_fingerprint_sha256(),
        cert_serial: auth.cert_serial(),
        cert_presented: auth.cert_presented(),
    });

    let bot_management = cf.bot_management().map(|bm| BotManagementDetails {
        score: bm.score(),
        static_resource: bm.static_resource(),
        verified_bot: bm.verified_bot(),
        corporate_proxy: bm.corporate_proxy(),
        ja4: bm.ja4(),
        ja3_hash: bm.ja3_hash(),
    });

    let coordinates = cf.coordinates().map(|(lat, lon)| Coordinates {
        latitude: lat,
        longitude: lon,
    });

    let request_priority = cf.request_priority().map(|rp| RequestPriorityDetails {
        weight: rp.weight,
        exclusive: rp.exclusive,
        group: rp.group,
        group_weight: rp.group_weight,
    });

    let cf_details = CfDetails {
        asn: cf.asn(),
        as_organization: cf.as_organization(),
        city: cf.city(),
        colo: cf.colo(),
        continent: cf.continent(),
        country: cf.country(),
        http_protocol: cf.http_protocol(),
        coordinates,
        postal_code: cf.postal_code(),
        metro_code: cf.metro_code(),
        region: cf.region(),
        region_code: cf.region_code(),
        timezone: cf.timezone_name(),
        is_eu_country: cf.is_eu_country(),
        tls_version: cf.tls_version(),
        tls_cipher: cf.tls_cipher(),
        tls_client_auth,
        bot_management,
        verified_bot_category: cf.verified_bot_category(),
        request_priority,
    };

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
