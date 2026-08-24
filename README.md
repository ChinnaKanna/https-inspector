# https-inspector

A Cloudflare Worker that inspects incoming HTTP requests and returns **all available metadata** about the request, including network fingerprints (JA4/JA3), TLS details, bot management signals, geolocation, and request headers.

## What It Returns

### Basic Request Info
```json
{
  "method": "GET",
  "url": "https://https-inspector.chinnabanglore.workers.dev/",
  "path": "/",
  "client_ip": "207.60.82.113",
  "user_agent": "Mozilla/5.0 ...",
  "headers": { ... },
  "header_count": 18,
  "total_header_bytes": 1234,
  "cf": { ... }
}
```

### Cloudflare `cf` Object — Full Reference

#### Network / Origin
| Field | Type | Description |
|-------|------|-------------|
| `asn` | `number` | Autonomous System Number |
| `as_organization` | `string` | ASN organization name |
| `colo` | `string` | Cloudflare data center code (e.g. "ATX", "LUX") |
| `http_protocol` | `string` | HTTP protocol (e.g. "HTTP/2", "HTTP/3") |
| `tls_version` | `string` | TLS version (e.g. "TLSv1.3") |
| `tls_cipher` | `string` | TLS cipher suite (e.g. "AEAD-AES128-GCM-SHA256") |

#### Geolocation
| Field | Type | Description |
|-------|------|-------------|
| `city` | `string` | City name |
| `region` | `string` | Region/State name |
| `region_code` | `string` | Region code (e.g. "TX") |
| `country` | `string` | Two-letter country code |
| `continent` | `string` | Continent code (e.g. "NA") |
| `coordinates` | `object` | `{ latitude, longitude }` |
| `postal_code` | `string` | Postal/ZIP code |
| `metro_code` | `string` | DMA metro code |
| `timezone` | `string` | Timezone name |
| `is_eu_country` | `boolean` | Whether country is in EU |

#### Bot Management (requires Cloudflare Bot Management)
| Field | Type | Description |
|-------|------|-------------|
| `bot_management.score` | `number` | Bot score (0-100) |
| `bot_management.verified_bot` | `boolean` | Known verified bot |
| `bot_management.static_resource` | `boolean` | Static resource request |
| `bot_management.corporate_proxy` | `boolean` | Corporate proxy detected |
| `bot_management.ja4` | `string` | **JA4 network fingerprint** |
| `bot_management.ja3_hash` | `string` | **JA3 TLS fingerprint hash** |
| `bot_management.js_detection.passed` | `boolean` | JS challenge passed |
| `bot_management.detection_ids` | `number[]` | Detection IDs |
| `verified_bot_category` | `string` | Bot category (if verified) |

#### TLS Client Authentication (requires Cloudflare Access / API Shield)
| Field | Type | Description |
|-------|------|-------------|
| `tls_client_auth.cert_issuer_dn` | `string` | Certificate issuer DN |
| `tls_client_auth.cert_subject_dn` | `string` | Certificate subject DN |
| `tls_client_auth.cert_verified` | `string` | Verification status |
| `tls_client_auth.cert_fingerprint_sha256` | `string` | SHA-256 fingerprint |
| `tls_client_auth.cert_fingerprint_sha1` | `string` | SHA-1 fingerprint |
| `tls_client_auth.cert_serial` | `string` | Serial number |
| `tls_client_auth.cert_not_before` | `string` | Valid from |
| `tls_client_auth.cert_not_after` | `string` | Valid until |
| `tls_client_auth.cert_presented` | `string` | Certificate presented |

#### Browser Request Priority
| Field | Type | Description |
|-------|------|-------------|
| `request_priority.weight` | `number` | HTTP/2 weight |
| `request_priority.exclusive` | `boolean` | HTTP/2 exclusive flag |
| `request_priority.group` | `number` | HTTP/2 stream group |
| `request_priority.group_weight` | `number` | HTTP/2 group weight |

#### Host Metadata (Cloudflare for SaaS)
| Field | Type | Description |
|-------|------|-------------|
| `host_metadata` | `object` | Custom host metadata |

---

## What Cloudflare Workers CANNOT Provide

Cloudflare Workers runs in a **sandboxed V8 isolate** — it has no access to raw network layers. The following are **impossible** to get from a Worker:

| Detail | Available? | Why |
|--------|-----------|-----|
| TCP handshake (SYN, SYN-ACK, ACK) | ❌ No | Below Workers runtime |
| TCP packet lengths / window size | ❌ No | No raw socket access |
| HTTP/2 frame sizes | ❌ No | Abstracted by runtime |
| Raw TLS Client Hello / Server Hello bytes | ❌ No | TLS terminated at edge, only metadata exposed |
| TLS key exchange details | ❌ No | Not exposed to V8 |
| IP TTL, TCP options | ❌ No | Network layer invisible |
| **JA4 fingerprint** | ✅ Yes | Via `cf.bot_management.ja4` |
| **JA3 hash** | ✅ Yes | Via `cf.bot_management.ja3_hash` |
| **TLS version/cipher** | ✅ Yes | Via `cf.tls_version`, `cf.tls_cipher` |
| **Client certificate** | ✅ Yes | Via `cf.tls_client_auth` |
| **Bot score** | ✅ Yes | Via `cf.bot_management.score` |
| **Geo/ASN/Colo** | ✅ Yes | Full `cf` object |
| **HTTP protocol** | ✅ Yes | Via `cf.http_protocol` |
| **Request priority** | ✅ Yes | Via `cf.request_priority` |

> To capture raw TCP/TLS data you'd need **eBPF/XDP on bare metal**, **Wireshark/tcpdump**, or a **custom proxy with raw socket access** — not a serverless platform.

---

## Local Setup (Mac)

### Prerequisites

1. **Rust** — Install via rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **wasm32-unknown-unknown target** — Required to compile Rust to WebAssembly:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **worker-build** — Cloudflare's build tool for Rust workers:
   ```bash
   cargo install worker-build
   ```

4. **Wrangler CLI** — Cloudflare's deployment tool:
   ```bash
   npm install -g wrangler
   ```

5. **Node.js** — Required for Wrangler (v18+ recommended):
   ```bash
   brew install node
   ```

### Clone the Repository

```bash
git clone https://github.com/ChinnaKanna/https-inspector.git
cd https-inspector
```

### Build

```bash
worker-build --release
```

This compiles the Rust code to WebAssembly and outputs the build artifacts to the `build/` directory:
- `build/index.js` — JavaScript shim
- `build/index_bg.wasm` — Compiled WebAssembly module

### Verify Locally

Start the local development server:

```bash
wrangler dev
```

Then test it:

```bash
curl http://localhost:8787
```

You should see a JSON response with your request details.

### Deploy to Cloudflare

```bash
wrangler deploy
```

Or push to GitHub — the repository includes a GitHub Actions workflow (`.github/workflows/deploy.yml`) that automatically deploys on every push to `main`.

### Authenticate with Cloudflare (first time only)

```bash
wrangler login
```

This opens a browser window to authorize Wrangler with your Cloudflare account.

---

## Project Structure

```
.
├── src/
│   └── lib.rs          # Worker source code (Rust)
├── build/              # Compiled output (committed for direct deploy)
│   ├── index.js
│   ├── index_bg.wasm
│   ├── package.json
│   └── worker/
├── Cargo.toml          # Rust dependencies
├── wrangler.toml       # Cloudflare Worker configuration
└── .github/workflows/
    └── deploy.yml      # CI/CD pipeline
```

## Tech Stack

- **Rust** — Worker logic compiled to WebAssembly
- **worker crate** — Cloudflare Workers SDK for Rust
- **serde** — Serialization for JSON responses
- **Cloudflare Workers** — Serverless execution environment
