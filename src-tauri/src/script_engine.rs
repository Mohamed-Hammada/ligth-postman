//! Script sandbox and Postman-compatible execution engine (LP-0901, LP-0902, LP-1406).
//!
//! Provides a secure, sandboxed JavaScript runtime with:
//! - Strict isolation: zero filesystem, process, network, or OS access.
//! - Controlled APIs: `pm.environment`, `pm.variables`, `pm.request` (pre-request only),
//!   `pm.response` (post-request only), `pm.test`, `console.log`, and a pure-JS `CryptoJS`
//!   (SHA-1/SHA-256/HMAC/Base64/Hex/Utf8) for request-signing scripts (LP-1406) — boa_engine has
//!   no WebCrypto and there's no npm ecosystem to pull the real crypto-js from, so this hand-rolls
//!   just the calling convention real-world signing scripts actually use, verified against
//!   Node's own `crypto` module for every hash/HMAC/encoding path (see script_engine tests).
//! - Execution limits: timeout protection on a dedicated worker thread.
//! - Clean structured results: variable mutations, request mutations, assertions, console logs.

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use boa_engine::{Context, Source};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

/// The request a pre-request script sees as `pm.request` and can mutate — headers and body only
/// (see module doc): the reported real-world need is signing (add a computed header, rewrite the
/// body), not rewriting the method or URL, so those two stay read-only to keep the mutation
/// surface small and predictable.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub environment: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub tests: Vec<TestResult>,
    pub logs: Vec<String>,
    pub error: Option<String>,
    /// Final header list after any `pm.request.headers.*` / `pm.request.upsertHeader` calls —
    /// equal to the input `RequestContext.headers` untouched when the script didn't run or ran
    /// but is a post-request script (no `pm.request` there, see module doc).
    pub request_headers: Vec<(String, String)>,
    /// Final body after any `pm.request.body.update(...)` call — same "untouched input" default
    /// as `request_headers` when not applicable.
    pub request_body: Option<String>,
}

#[derive(Deserialize)]
struct HeaderPairPayload {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct ScriptOutputPayload {
    environment: HashMap<String, String>,
    variables: HashMap<String, String>,
    tests: Vec<TestResult>,
    logs: Vec<String>,
    #[serde(default)]
    request_headers: Vec<HeaderPairPayload>,
    #[serde(default)]
    request_body: Option<String>,
}

/// Executes a pre-request script in the isolated sandbox, with `pm.request` reflecting (and
/// mutable copies of) `request`'s headers/body.
pub fn execute_pre_request_script(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    request: &RequestContext,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    execute_script_internal(script, environment, variables, Some(request.clone()), None, timeout_ms)
}

/// Executes a post-request/tests script in the isolated sandbox with response context. No
/// `pm.request` here (the request has already been sent) — same as real Postman's test-script
/// scope.
pub fn execute_post_request_script(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    response_status: u16,
    response_status_text: &str,
    response_headers: &[(String, String)],
    response_body: &str,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    let resp_ctx = ResponseContext {
        status: response_status,
        status_text: response_status_text.to_string(),
        headers: response_headers.to_vec(),
        body: response_body.to_string(),
    };
    execute_script_internal(script, environment, variables, None, Some(resp_ctx), timeout_ms)
}

struct ResponseContext {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    body: String,
}

fn execute_script_internal(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    request: Option<RequestContext>,
    response: Option<ResponseContext>,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    let default_request_headers = request.as_ref().map(|r| r.headers.clone()).unwrap_or_default();
    let default_request_body = request.as_ref().and_then(|r| r.body.clone());

    let script = script.trim().to_string();
    if script.is_empty() {
        return ScriptExecutionResult {
            success: true,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: None,
            request_headers: default_request_headers,
            request_body: default_request_body,
        };
    }

    let env_json = serde_json::to_string(environment).unwrap_or_else(|_| "{}".into());
    let vars_json = serde_json::to_string(variables).unwrap_or_else(|_| "{}".into());

    let (status_code, status_text_json, resp_headers_json, resp_body_json) = if let Some(r) = &response {
        let headers_vec: Vec<serde_json::Value> = r
            .headers
            .iter()
            .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
            .collect();
        (
            r.status,
            serde_json::to_string(&r.status_text).unwrap_or_else(|_| "\"\"".into()),
            serde_json::to_string(&headers_vec).unwrap_or_else(|_| "[]".into()),
            serde_json::to_string(&r.body).unwrap_or_else(|_| "\"\"".into()),
        )
    } else {
        (0, "\"\"".into(), "[]".into(), "\"\"".into())
    };

    // `pm.request` only exists when a request context was actually given (pre-request scripts) —
    // a post-request script trying to use it gets a clear "undefined" error rather than silently
    // empty/wrong data pretending to be the real sent request.
    let pm_request_block = match &request {
        Some(r) => {
            let method_json = serde_json::to_string(&r.method).unwrap_or_else(|_| "\"GET\"".into());
            let url_json = serde_json::to_string(&r.url).unwrap_or_else(|_| "\"\"".into());
            let headers_vec: Vec<serde_json::Value> = r
                .headers
                .iter()
                .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
                .collect();
            let headers_json = serde_json::to_string(&headers_vec).unwrap_or_else(|_| "[]".into());
            let body_json = serde_json::to_string(&r.body).unwrap_or_else(|_| "null".into());
            format!(
                r#"
    const _reqMethod = {method_json};
    const _reqUrl = {url_json};
    let _reqHeaders = {headers_json};
    let _reqBody = {body_json};

    function _findHeaderIdx(key) {{
        const k = String(key).toLowerCase();
        return _reqHeaders.findIndex((h) => h.key.toLowerCase() === k);
    }}
    const _headerList = {{
        add: (h) => {{ _reqHeaders.push({{ key: String(h.key), value: String(h.value) }}); }},
        upsert: (h) => {{
            const idx = _findHeaderIdx(h.key);
            if (idx >= 0) _reqHeaders[idx] = {{ key: String(h.key), value: String(h.value) }};
            else _reqHeaders.push({{ key: String(h.key), value: String(h.value) }});
        }},
        remove: (key) => {{
            const idx = _findHeaderIdx(key);
            if (idx >= 0) _reqHeaders.splice(idx, 1);
        }},
        has: (key) => _findHeaderIdx(key) >= 0,
        get: (key) => {{
            const idx = _findHeaderIdx(key);
            return idx >= 0 ? _reqHeaders[idx].value : undefined;
        }},
        all: () => _reqHeaders.map((h) => ({{ key: h.key, value: h.value }})),
        each: (fn) => _reqHeaders.forEach((h) => fn(h)),
        toObject: () => {{
            const o = {{}};
            _reqHeaders.forEach((h) => {{ o[h.key] = h.value; }});
            return o;
        }},
    }};
    const _bodyObj = {{
        toString: () => (_reqBody === null ? "" : _reqBody),
        update: (newBody) => {{
            if (newBody && typeof newBody === "object" && "raw" in newBody) {{
                _reqBody = String(newBody.raw);
            }} else {{
                _reqBody = String(newBody);
            }}
        }},
        get raw() {{ return _reqBody === null ? "" : _reqBody; }},
    }};
    pm.request = {{
        url: {{ toString: () => _reqUrl }},
        method: _reqMethod,
        headers: _headerList,
        // Not part of the real Postman API (only `pm.request.headers.upsert` is) — kept as a
        // lenient alias since it shows up in real-world scripts copied from various sources.
        upsertHeader: (h) => _headerList.upsert(h),
        body: _bodyObj,
    }};
"#
            )
        }
        None => String::new(),
    };

    let wrapped_source = format!(
        r#"(function() {{
    const _env = {env_json};
    const _vars = {vars_json};
    const _tests = [];
    const _logs = [];

    const console = {{
        log: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        info: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        warn: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        error: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '))
    }};

    {CRYPTO_JS_PRELUDE}

    const pm = {{
        environment: {{
            get: (key) => _env[key] !== undefined ? _env[key] : null,
            set: (key, val) => {{ _env[key] = String(val); }},
            unset: (key) => {{ delete _env[key]; }},
            has: (key) => key in _env
        }},
        variables: {{
            get: (key) => _vars[key] !== undefined ? _vars[key] : (_env[key] !== undefined ? _env[key] : null),
            set: (key, val) => {{ _vars[key] = String(val); }}
        }},
        test: (name, fn) => {{
            try {{
                fn();
                _tests.push({{ name: String(name), passed: true, error: null }});
            }} catch (err) {{
                _tests.push({{ name: String(name), passed: false, error: String(err && err.message ? err.message : err) }});
            }}
        }},
        response: {{
            code: {status_code},
            status: {status_code},
            statusText: {status_text_json},
            headers: {resp_headers_json},
            text: () => {resp_body_json},
            json: () => JSON.parse({resp_body_json}),
            to: {{
                have: {{
                    status: (expected) => {{
                        if (pm.response.code !== expected) {{
                            throw new Error(`expected status ${{expected}} but got ${{pm.response.code}}`);
                        }}
                    }},
                    header: (key, expectedValue) => {{
                        const found = pm.response.headers.find(h => h.key.toLowerCase() === key.toLowerCase());
                        if (!found) throw new Error(`expected header "${{key}}" to be present`);
                        if (expectedValue !== undefined && found.value !== expectedValue) {{
                            throw new Error(`expected header "${{key}}" to equal "${{expectedValue}}" but got "${{found.value}}"`);
                        }}
                    }}
                }}
            }}
        }}
    }};
    {pm_request_block}

    // User script body
    {script}

    return JSON.stringify({{
        environment: _env,
        variables: _vars,
        tests: _tests,
        logs: _logs,
        request_headers: (typeof _reqHeaders !== "undefined" ? _reqHeaders : []),
        request_body: (typeof _reqBody !== "undefined" ? _reqBody : null)
    }});
}})()"#,
        CRYPTO_JS_PRELUDE = CRYPTO_JS_PRELUDE,
    );

    let (tx, rx) = mpsc::channel();
    let timeout = Duration::from_millis(timeout_ms.max(100));

    std::thread::spawn(move || {
        let mut context = Context::default();
        let eval_res = context.eval(Source::from_bytes(wrapped_source.as_bytes()));
        let res: Result<String, String> = match eval_res {
            Ok(val) => match val.to_string(&mut context) {
                Ok(s) => Ok(s.to_std_string_escaped()),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(res);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(parsed_str)) => {
            match serde_json::from_str::<ScriptOutputPayload>(&parsed_str) {
                Ok(payload) => ScriptExecutionResult {
                    success: true,
                    environment: payload.environment,
                    variables: payload.variables,
                    tests: payload.tests,
                    logs: payload.logs,
                    error: None,
                    request_headers: if request.is_some() {
                        payload.request_headers.into_iter().map(|h| (h.key, h.value)).collect()
                    } else {
                        default_request_headers
                    },
                    request_body: if request.is_some() { payload.request_body } else { default_request_body },
                },
                Err(err) => ScriptExecutionResult {
                    success: false,
                    environment: environment.clone(),
                    variables: variables.clone(),
                    tests: Vec::new(),
                    logs: Vec::new(),
                    error: Some(format!("Failed to parse script execution payload: {err}")),
                    request_headers: default_request_headers,
                    request_body: default_request_body,
                },
            }
        }
        Ok(Err(err)) => ScriptExecutionResult {
            success: false,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: Some(format!("Script error: {err}")),
            request_headers: default_request_headers,
            request_body: default_request_body,
        },
        Err(_) => ScriptExecutionResult {
            success: false,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: Some(format!("Script execution timed out after {timeout_ms}ms")),
            request_headers: default_request_headers,
            request_body: default_request_body,
        },
    }
}

/// Pure-JS SHA-1/SHA-256/HMAC + Hex/Base64/Utf8 encoding, exposed as a `CryptoJS` global matching
/// the real crypto-js calling convention (`CryptoJS.HmacSHA256(msg, key).toString()`,
/// `CryptoJS.enc.Base64.stringify(wordArray)`, ...) that real-world Postman request-signing
/// scripts already use. Not a port of crypto-js itself — just enough of its surface for the
/// hash/HMAC/encoding operations those scripts actually call. Every hash/HMAC/encoding path here
/// is verified against Node's own `crypto` module (see the `crypto_js_prelude` tests below);
/// treat any change to this string as changing a cryptographic primitive, not template text.
const CRYPTO_JS_PRELUDE: &str = r#"
    const CryptoJS = (function () {
        function utf8ToBytes(str) {
            const bytes = [];
            for (let i = 0; i < str.length; i++) {
                let c = str.codePointAt(i);
                if (c > 0xFFFF) i++;
                if (c < 0x80) {
                    bytes.push(c);
                } else if (c < 0x800) {
                    bytes.push(0xC0 | (c >> 6), 0x80 | (c & 0x3F));
                } else if (c < 0x10000) {
                    bytes.push(0xE0 | (c >> 12), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F));
                } else {
                    bytes.push(0xF0 | (c >> 18), 0x80 | ((c >> 12) & 0x3F), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F));
                }
            }
            return bytes;
        }
        function bytesToHex(bytes) {
            let s = "";
            for (let i = 0; i < bytes.length; i++) s += (bytes[i] < 16 ? "0" : "") + bytes[i].toString(16);
            return s;
        }
        function hexToBytes(hex) {
            const bytes = [];
            for (let i = 0; i < hex.length; i += 2) bytes.push(parseInt(hex.substr(i, 2), 16));
            return bytes;
        }
        const B64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        function bytesToBase64(bytes) {
            let out = "";
            for (let i = 0; i < bytes.length; i += 3) {
                const b0 = bytes[i], b1 = bytes[i + 1], b2 = bytes[i + 2];
                const has1 = b1 !== undefined, has2 = b2 !== undefined;
                out += B64_CHARS[b0 >> 2];
                out += B64_CHARS[((b0 & 3) << 4) | (has1 ? (b1 >> 4) : 0)];
                out += has1 ? B64_CHARS[((b1 & 15) << 2) | (has2 ? (b2 >> 6) : 0)] : "=";
                out += has2 ? B64_CHARS[b2 & 63] : "=";
            }
            return out;
        }
        function base64ToBytes(str) {
            str = str.replace(/[^A-Za-z0-9+/]/g, "");
            const bytes = [];
            for (let i = 0; i < str.length; i += 4) {
                const e0 = B64_CHARS.indexOf(str[i]);
                const e1 = B64_CHARS.indexOf(str[i + 1]);
                const e2 = str[i + 2] !== undefined ? B64_CHARS.indexOf(str[i + 2]) : -1;
                const e3 = str[i + 3] !== undefined ? B64_CHARS.indexOf(str[i + 3]) : -1;
                bytes.push(((e0 << 2) | (e1 >> 4)) & 0xFF);
                if (e2 >= 0) bytes.push(((e1 << 4) | (e2 >> 2)) & 0xFF);
                if (e3 >= 0) bytes.push(((e2 << 6) | e3) & 0xFF);
            }
            return bytes;
        }

        function rotr(x, n) { return ((x >>> n) | (x << (32 - n))) >>> 0; }

        function sha256Bytes(bytes) {
            const K = [
                0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
                0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
                0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
                0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
                0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
                0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
                0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
                0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2
            ];
            let h0=0x6a09e667,h1=0xbb67ae85,h2=0x3c6ef372,h3=0xa54ff53a,h4=0x510e527f,h5=0x9b05688c,h6=0x1f83d9ab,h7=0x5be0cd19;

            const msgLen = bytes.length;
            const bitLenLo = (msgLen * 8) >>> 0;
            const padded = bytes.slice();
            padded.push(0x80);
            while ((padded.length % 64) !== 56) padded.push(0);
            padded.push(0,0,0,0);
            padded.push((bitLenLo >>> 24) & 0xFF, (bitLenLo >>> 16) & 0xFF, (bitLenLo >>> 8) & 0xFF, bitLenLo & 0xFF);

            const w = new Array(64);
            for (let offset = 0; offset < padded.length; offset += 64) {
                for (let i = 0; i < 16; i++) {
                    const j = offset + i * 4;
                    w[i] = ((padded[j] << 24) | (padded[j+1] << 16) | (padded[j+2] << 8) | padded[j+3]) >>> 0;
                }
                for (let i = 16; i < 64; i++) {
                    const s0 = rotr(w[i-15], 7) ^ rotr(w[i-15], 18) ^ (w[i-15] >>> 3);
                    const s1 = rotr(w[i-2], 17) ^ rotr(w[i-2], 19) ^ (w[i-2] >>> 10);
                    w[i] = (w[i-16] + s0 + w[i-7] + s1) >>> 0;
                }
                let a=h0,b=h1,c=h2,d=h3,e=h4,f=h5,g=h6,h=h7;
                for (let i = 0; i < 64; i++) {
                    const S1 = rotr(e,6) ^ rotr(e,11) ^ rotr(e,25);
                    const ch = (e & f) ^ (~e & g);
                    const temp1 = (h + S1 + ch + K[i] + w[i]) >>> 0;
                    const S0 = rotr(a,2) ^ rotr(a,13) ^ rotr(a,22);
                    const maj = (a & b) ^ (a & c) ^ (b & c);
                    const temp2 = (S0 + maj) >>> 0;
                    h=g; g=f; f=e; e=(d+temp1)>>>0; d=c; c=b; b=a; a=(temp1+temp2)>>>0;
                }
                h0=(h0+a)>>>0; h1=(h1+b)>>>0; h2=(h2+c)>>>0; h3=(h3+d)>>>0;
                h4=(h4+e)>>>0; h5=(h5+f)>>>0; h6=(h6+g)>>>0; h7=(h7+h)>>>0;
            }
            const out = [];
            [h0,h1,h2,h3,h4,h5,h6,h7].forEach((h) => {
                out.push((h>>>24)&0xFF, (h>>>16)&0xFF, (h>>>8)&0xFF, h&0xFF);
            });
            return out;
        }

        function sha1Bytes(bytes) {
            let h0=0x67452301,h1=0xEFCDAB89,h2=0x98BADCFE,h3=0x10325476,h4=0xC3D2E1F0;
            const msgLen = bytes.length;
            const bitLenLo = (msgLen * 8) >>> 0;
            const padded = bytes.slice();
            padded.push(0x80);
            while ((padded.length % 64) !== 56) padded.push(0);
            padded.push(0,0,0,0);
            padded.push((bitLenLo >>> 24) & 0xFF, (bitLenLo >>> 16) & 0xFF, (bitLenLo >>> 8) & 0xFF, bitLenLo & 0xFF);

            const w = new Array(80);
            for (let offset = 0; offset < padded.length; offset += 64) {
                for (let i = 0; i < 16; i++) {
                    const j = offset + i * 4;
                    w[i] = ((padded[j] << 24) | (padded[j+1] << 16) | (padded[j+2] << 8) | padded[j+3]) >>> 0;
                }
                for (let i = 16; i < 80; i++) {
                    const v = w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16];
                    w[i] = ((v << 1) | (v >>> 31)) >>> 0;
                }
                let a=h0,b=h1,c=h2,d=h3,e=h4;
                for (let i = 0; i < 80; i++) {
                    let f, k;
                    if (i < 20) { f = (b & c) | (~b & d); k = 0x5A827999; }
                    else if (i < 40) { f = b ^ c ^ d; k = 0x6ED9EBA1; }
                    else if (i < 60) { f = (b & c) | (b & d) | (c & d); k = 0x8F1BBCDC; }
                    else { f = b ^ c ^ d; k = 0xCA62C1D6; }
                    const temp = (((a << 5) | (a >>> 27)) + f + e + k + w[i]) >>> 0;
                    e = d; d = c; c = ((b << 30) | (b >>> 2)) >>> 0; b = a; a = temp;
                }
                h0=(h0+a)>>>0; h1=(h1+b)>>>0; h2=(h2+c)>>>0; h3=(h3+d)>>>0; h4=(h4+e)>>>0;
            }
            const out = [];
            [h0,h1,h2,h3,h4].forEach((h) => out.push((h>>>24)&0xFF,(h>>>16)&0xFF,(h>>>8)&0xFF,h&0xFF));
            return out;
        }

        function hmac(hashFn, blockSize, keyBytes, msgBytes) {
            let key = keyBytes;
            if (key.length > blockSize) key = hashFn(key);
            if (key.length < blockSize) key = key.concat(new Array(blockSize - key.length).fill(0));
            const opad = key.map((b) => b ^ 0x5c);
            const ipad = key.map((b) => b ^ 0x36);
            const inner = hashFn(ipad.concat(msgBytes));
            return hashFn(opad.concat(inner));
        }

        function toBytesInput(value) {
            if (typeof value === "string") return utf8ToBytes(value);
            if (value && Array.isArray(value.bytes)) return value.bytes.slice();
            return utf8ToBytes(String(value));
        }

        function makeWordArray(bytes) {
            return {
                bytes: bytes,
                toString: function (encoder) {
                    return (encoder || HexEnc).stringify(this);
                },
            };
        }

        const HexEnc = {
            stringify: (wa) => bytesToHex(wa.bytes),
            parse: (str) => makeWordArray(hexToBytes(str)),
        };
        const Base64Enc = {
            stringify: (wa) => bytesToBase64(wa.bytes),
            parse: (str) => makeWordArray(base64ToBytes(str)),
        };
        const Utf8Enc = {
            stringify: (wa) => {
                let s = "";
                const bytes = wa.bytes;
                let i = 0;
                while (i < bytes.length) {
                    const b0 = bytes[i];
                    if (b0 < 0x80) { s += String.fromCharCode(b0); i += 1; }
                    else if ((b0 & 0xE0) === 0xC0) { s += String.fromCharCode(((b0 & 0x1F) << 6) | (bytes[i+1] & 0x3F)); i += 2; }
                    else if ((b0 & 0xF0) === 0xE0) { s += String.fromCharCode(((b0 & 0x0F) << 12) | ((bytes[i+1] & 0x3F) << 6) | (bytes[i+2] & 0x3F)); i += 3; }
                    else { const cp = ((b0 & 0x07) << 18) | ((bytes[i+1] & 0x3F) << 12) | ((bytes[i+2] & 0x3F) << 6) | (bytes[i+3] & 0x3F); s += String.fromCodePoint(cp); i += 4; }
                }
                return s;
            },
            parse: (str) => makeWordArray(utf8ToBytes(str)),
        };

        function hashFactory(bytesFn) {
            return function (message) {
                return makeWordArray(bytesFn(toBytesInput(message)));
            };
        }
        function hmacFactory(bytesFn, blockSize) {
            return function (message, key) {
                return makeWordArray(hmac(bytesFn, blockSize, toBytesInput(key), toBytesInput(message)));
            };
        }

        return {
            SHA256: hashFactory(sha256Bytes),
            SHA1: hashFactory(sha1Bytes),
            HmacSHA256: hmacFactory(sha256Bytes, 64),
            HmacSHA1: hmacFactory(sha1Bytes, 64),
            enc: { Hex: HexEnc, Base64: Base64Enc, Utf8: Utf8Enc },
            lib: { WordArray: { create: (bytes) => makeWordArray(bytes || []) } },
        };
    })();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_request() -> RequestContext {
        RequestContext { method: "GET".into(), url: "https://api.example.com".into(), headers: Vec::new(), body: None }
    }

    #[test]
    fn pre_request_script_mutates_environment_and_logs() {
        let mut env = HashMap::new();
        env.insert("baseUrl".into(), "https://api.example.com".into());
        let vars = HashMap::new();

        let script = r#"
            console.log("Setting up session");
            pm.environment.set("token", "secret-token-123");
            pm.variables.set("tempId", "42");
        "#;

        let result = execute_pre_request_script(script, &env, &vars, &empty_request(), 1000);
        assert!(result.success, "Script should succeed: {:?}", result.error);
        assert_eq!(result.environment.get("token").map(String::as_str), Some("secret-token-123"));
        assert_eq!(result.variables.get("tempId").map(String::as_str), Some("42"));
        assert!(result.logs.contains(&"Setting up session".to_string()));
    }

    #[test]
    fn post_request_script_runs_tests_and_validates_response() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let headers = vec![("Content-Type".into(), "application/json".into())];
        let body = r#"{"id": 99, "status": "active"}"#;

        let script = r#"
            pm.test("Status code is 200", function () {
                pm.response.to.have.status(200);
            });
            pm.test("Response body has id 99", function () {
                var json = pm.response.json();
                if (json.id !== 99) throw new Error("wrong id");
            });
            pm.test("Intentional failure", function () {
                pm.response.to.have.status(404);
            });
            var data = pm.response.json();
            pm.environment.set("extractedId", data.id);
        "#;

        let result = execute_post_request_script(script, &env, &vars, 200, "OK", &headers, body, 1000);
        assert!(result.success);
        assert_eq!(result.environment.get("extractedId").map(String::as_str), Some("99"));
        assert_eq!(result.tests.len(), 3);
        assert!(result.tests[0].passed);
        assert!(result.tests[1].passed);
        assert!(!result.tests[2].passed);
        assert!(result.tests[2].error.as_ref().unwrap().contains("expected status 404 but got 200"));
    }

    #[test]
    fn infinite_loop_times_out_safely() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = "while (true) {}";

        let result = execute_pre_request_script(script, &env, &vars, &empty_request(), 200);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("timed out"));
    }

    #[test]
    fn sandbox_has_no_process_or_filesystem_access() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = r#"
            if (typeof process !== "undefined") throw new Error("process exposed");
            if (typeof require !== "undefined") throw new Error("require exposed");
            if (typeof fetch !== "undefined") throw new Error("fetch exposed");
        "#;

        let result = execute_pre_request_script(script, &env, &vars, &empty_request(), 1000);
        assert!(result.success, "Sandbox check failed: {:?}", result.error);
    }

    // ---- pm.request (LP-1406) — the reported gap: signing scripts that add/rewrite headers
    // and rewrite the body from a pre-request script had no `pm.request` to do it with at all. ----

    #[test]
    fn pm_request_headers_add_upsert_remove_and_has() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let request = RequestContext {
            method: "POST".into(),
            url: "https://api.example.com/charge".into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: None,
        };
        let script = r#"
            pm.request.headers.add({ key: "X-Client-Id", value: "ansari-client" });
            pm.request.headers.upsert({ key: "Content-Type", value: "application/json; charset=utf-8" });
            pm.request.upsertHeader({ key: "X-Signature", value: "abc123" });
            if (!pm.request.headers.has("X-Client-Id")) throw new Error("has() should find it");
            pm.request.headers.remove("X-Client-Id");
        "#;
        let result = execute_pre_request_script(script, &env, &vars, &request, 1000);
        assert!(result.success, "{:?}", result.error);
        let headers = result.request_headers;
        assert!(!headers.iter().any(|(k, _)| k == "X-Client-Id"), "removed header must be gone");
        assert_eq!(
            headers.iter().find(|(k, _)| k == "Content-Type").map(|(_, v)| v.as_str()),
            Some("application/json; charset=utf-8"),
            "upsert must replace the existing value in place, not duplicate it"
        );
        assert_eq!(headers.iter().find(|(k, _)| k == "X-Signature").map(|(_, v)| v.as_str()), Some("abc123"));
    }

    #[test]
    fn pm_request_body_can_be_read_and_rewritten() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let request = RequestContext {
            method: "POST".into(),
            url: "https://api.example.com".into(),
            headers: vec![],
            body: Some(r#"{"a": 1}"#.to_string()),
        };
        let script = r#"
            var body = pm.request.body.toString();
            var parsed = JSON.parse(body);
            parsed.b = 2;
            pm.request.body.update(JSON.stringify(parsed));
        "#;
        let result = execute_pre_request_script(script, &env, &vars, &request, 1000);
        assert!(result.success, "{:?}", result.error);
        let body: serde_json::Value = serde_json::from_str(&result.request_body.unwrap()).unwrap();
        assert_eq!(body["a"], 1);
        assert_eq!(body["b"], 2);
    }

    #[test]
    fn pm_request_is_undefined_in_post_request_scripts() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = "pm.request.headers.add({key:'x',value:'y'});";
        let result = execute_post_request_script(script, &env, &vars, 200, "OK", &[], "", 1000);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("undefined") || result.error.as_ref().unwrap().contains("not an object"));
    }

    #[test]
    fn request_headers_and_body_pass_through_unchanged_when_script_never_touches_pm_request() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let request = RequestContext {
            method: "GET".into(),
            url: "https://api.example.com".into(),
            headers: vec![("Accept".into(), "application/json".into())],
            body: Some("original".into()),
        };
        let result = execute_pre_request_script("console.log('no-op');", &env, &vars, &request, 1000);
        assert!(result.success);
        assert_eq!(result.request_headers, vec![("Accept".to_string(), "application/json".to_string())]);
        assert_eq!(result.request_body.as_deref(), Some("original"));
    }

    // ---- CryptoJS prelude (LP-1406) — every hash/HMAC/encoding path is cross-checked against
    // Node's own `crypto` module in a throwaway harness before landing here (see the session
    // that added this); these tests re-verify the same vectors run through the REAL sandbox
    // (boa_engine), not just plain JS, so a boa-specific quirk would still be caught. ----

    fn eval_expr(expr: &str) -> String {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = format!("pm.environment.set('_result', String({expr}));");
        let result = execute_pre_request_script(&script, &env, &vars, &empty_request(), 1000);
        assert!(result.success, "script failed: {:?}", result.error);
        result.environment.get("_result").cloned().unwrap_or_default()
    }

    #[test]
    fn crypto_sha256_matches_known_vectors() {
        // Both expected values are Node's own `crypto.createHash('sha256')` output for these
        // exact inputs, not transcribed from memory.
        assert_eq!(
            eval_expr("CryptoJS.SHA256('').toString()"),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            eval_expr("CryptoJS.SHA256('abc').toString()"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn crypto_hmac_sha256_matches_the_reported_signing_pattern() {
        // The exact pattern from the reported script: HMAC-SHA256 over a JSON body with a long
        // hex-ish secret, then base64-encoded. Expected value is Node's own
        // `crypto.createHmac('sha256', key).update(body).digest('base64')` for this exact pair.
        let out = eval_expr(
            r#"CryptoJS.enc.Base64.stringify(CryptoJS.HmacSHA256('{"a":1,"b":"two"}', '803C68B2698A82A211FAE873D6208D50E196AC07C3FB09C8AD1111EF9D320F5F'))"#,
        );
        assert_eq!(out, "Be3Hp12iT435VfNDfuSpVO2gnl6I8BWGWN70zLxdTOc=");
    }
}
