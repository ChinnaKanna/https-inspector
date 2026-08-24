use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
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
    cf: Option<HashMap<String, serde_json::Value>>,
}

/// Get the raw cf JS object from the underlying web_sys::Request
fn get_cf_js(req: &web_sys::Request) -> Option<JsValue> {
    let cf = js_sys::Reflect::get(req, &"cf".into()).ok()?;
    if cf.is_null() || cf.is_undefined() {
        None
    } else {
        Some(cf)
    }
}

/// Recursively convert any JsValue to a serde_json::Value
fn js_to_json(value: &JsValue) -> Option<serde_json::Value> {
    if value.is_null() || value.is_undefined() {
        Some(serde_json::Value::Null)
    } else if let Some(s) = value.as_string() {
        Some(serde_json::Value::String(s))
    } else if let Some(b) = value.as_bool() {
        Some(serde_json::Value::Bool(b))
    } else if let Some(n) = value.as_f64() {
        if n == n.floor() && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            serde_json::Value::Number(serde_json::Number::from(n as i64)).into()
        } else {
            serde_json::Number::from_f64(n).map(serde_json::Value::Number)
        }
    } else if value.is_array() {
        let arr = js_sys::Array::from(value);
        let mut result = Vec::new();
        for item in arr.iter() {
            if let Some(json_val) = js_to_json(&item) {
                result.push(json_val);
            }
        }
        Some(serde_json::Value::Array(result))
    } else if value.is_object() {
        let mut map = serde_json::Map::new();
        let obj = js_sys::Object::from(value.clone());
        let keys = js_sys::Object::keys(&obj);
        for key in keys.iter() {
            if let Some(key_str) = key.as_string() {
                if let Ok(val) = js_sys::Reflect::get(value, &key) {
                    if let Some(json_val) = js_to_json(&val) {
                        map.insert(key_str, json_val);
                    }
                }
            }
        }
        Some(serde_json::Value::Object(map))
    } else {
        value.as_string().map(serde_json::Value::String)
    }
}

/// Read all properties from the cf JS object dynamically
fn read_all_cf_properties(cf_js: &JsValue) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    
    // Get all own property keys of the cf object
    let obj = js_sys::Object::from(cf_js.clone());
    let keys = js_sys::Object::keys(&obj);
    
    for key in keys.iter() {
        if let Some(key_str) = key.as_string() {
            // Safely get each property
            if let Ok(val) = js_sys::Reflect::get(cf_js, &key) {
                if let Some(json_val) = js_to_json(&val) {
                    result.insert(key_str, json_val);
                }
            }
        }
    }
    
    result
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

    // Read ALL cf properties dynamically
    let cf_details = get_cf_js(req.inner()).map(|cf_js| {
        read_all_cf_properties(&cf_js)
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
