# https-inspector

A Cloudflare Worker that inspects incoming HTTP requests and returns detailed metadata about the request, including network fingerprints, TLS details, and geolocation data.

## What It Returns

When you make a request to the worker, it responds with a JSON object containing:

### Basic Request Info
- `method` — HTTP method (GET, POST, etc.)
- `url` — Full request URL
- `client_ip` — Client IP address (from `cf-connecting-ip`)
- `user_agent` — User agent string
- `headers` — All HTTP headers as key-value pairs

### Cloudflare `cf` Object
- **Bot Management**: `ja4`, `ja3_hash`, `score`, `verified_bot`, `static_resource`, `corporate_proxy`
- **TLS**: `tls_version`, `tls_cipher`, `tls_client_auth` (client certificate details when using Cloudflare Access/API Shield)
- **Geolocation**: `asn`, `as_organization`, `city`, `region`, `region_code`, `country`, `continent`, `coordinates`, `postal_code`, `metro_code`, `timezone`, `is_eu_country`
- **Network**: `colo`, `http_protocol`, `request_priority`

> **Note**: `ja4`/`ja3_hash` require Cloudflare Bot Management. `tls_client_auth` requires Cloudflare Access or API Shield.

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
