//! Optional iCreate accounts via Supabase. Failures never block installs or launches.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use artcraft_launcher_core::account::{AccountSnapshot, BillingInterval, Plan, SubscriptionStatus};
use artcraft_launcher_core::protocol::Event;
use artcraft_launcher_core::time::parse_rfc3339;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::http::Http;
use crate::paths::Paths;
use crate::process;

const OAUTH_PORT: u16 = 48174;
const OAUTH_WAIT: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Debug)]
pub struct Config {
    pub url: String,
    pub anon_key: String,
    anon_bearer: String,
}

pub enum Work {
    Refresh,
    SendCode(String),
    VerifyCode { email: String, code: String },
    Google,
    SignOut,
    Checkout(BillingInterval),
    Backup(String),
    Restore,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Stored {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    user_id: String,
    email: String,
    plan: Plan,
    interval: Option<BillingInterval>,
    status: SubscriptionStatus,
    period_end: Option<i64>,
}

impl Stored {
    fn snapshot(&self, configured: bool, busy: bool, last_error: Option<String>) -> AccountSnapshot {
        AccountSnapshot {
            configured,
            busy,
            user_id: Some(self.user_id.clone()),
            email: Some(self.email.clone()),
            plan: self.plan,
            interval: self.interval,
            status: self.status,
            period_end: self.period_end,
            last_error,
        }
    }
}

pub fn config() -> Option<Config> {
    let url = pick("ICREATE_SUPABASE_URL", option_env!("ICREATE_SUPABASE_URL"))?;
    let anon_key = pick("ICREATE_SUPABASE_ANON_KEY", option_env!("ICREATE_SUPABASE_ANON_KEY"))?;
    if !url.starts_with("https://") {
        return None;
    }
    let url = url.trim_end_matches('/').to_string();
    let anon_bearer = format!("Bearer {anon_key}");
    Some(Config { url, anon_key, anon_bearer })
}

fn pick(runtime: &str, compiled: Option<&str>) -> Option<String> {
    std::env::var(runtime)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| compiled.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string))
}

pub fn local_snapshot(paths: &Paths) -> AccountSnapshot {
    let configured = config().is_some();
    match load(paths) {
        Some(stored) => stored.snapshot(configured, false, None),
        None => AccountSnapshot { configured, ..AccountSnapshot::default() },
    }
}

pub fn run(http: &Http, paths: &Paths, work: Work, emit: impl Fn(Event)) {
    match work {
        Work::SignOut => {
            let _ = std::fs::remove_file(paths.account_session());
            emit(Event::Account(AccountSnapshot { configured: config().is_some(), ..AccountSnapshot::default() }));
        }
        other => match run_inner(http, paths, other, &emit) {
            Ok(()) => {}
            Err(text) => {
                let mut snap = local_snapshot(paths);
                snap.busy = false;
                snap.last_error = Some(text.clone());
                emit(Event::Account(snap));
                emit(Event::AccountNotice { text, error: true });
            }
        },
    }
}

fn run_inner(http: &Http, paths: &Paths, work: Work, emit: &impl Fn(Event)) -> Result<(), String> {
    let Some(cfg) = config() else {
        return Err("This build has no account server configured.".into());
    };
    match work {
        Work::Refresh => refresh_now(http, &cfg, paths, emit),
        Work::SendCode(email) => send_code(http, &cfg, paths, email, emit),
        Work::VerifyCode { email, code } => verify_code(http, &cfg, paths, &email, &code, emit),
        Work::Google => google_sign_in(http, &cfg, paths, emit),
        Work::Checkout(interval) => checkout(http, &cfg, paths, interval, emit),
        Work::Backup(json) => backup(http, &cfg, paths, &json, emit),
        Work::Restore => restore_settings(http, &cfg, paths, emit),
        Work::SignOut => Ok(()),
    }
}

fn refresh_now(http: &Http, cfg: &Config, paths: &Paths, emit: &impl Fn(Event)) -> Result<(), String> {
    let stored = restore_session(http, cfg, paths)?;
    let stored = fetch_subscription(http, cfg, stored)?;
    save(paths, &stored)?;
    emit(Event::Account(stored.snapshot(true, false, None)));
    Ok(())
}

fn send_code(http: &Http, cfg: &Config, paths: &Paths, email: String, emit: &impl Fn(Event)) -> Result<(), String> {
    let email = email.trim().to_string();
    if !email_ok(&email) {
        return Err("Enter an email address.".into());
    }
    let body = serde_json::to_string(&json!({ "email": email, "create_user": true })).map_err(|e| e.to_string())?;
    let (status, text) = http.request_json("POST", &format!("{}/auth/v1/otp", cfg.url), &anon_headers(cfg), Some(&body))?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "Couldn't send a sign-in code."));
    }
    emit(Event::AccountNotice { text: format!("We sent a sign-in code to {email}"), error: false });
    emit(Event::Account(local_snapshot(paths)));
    Ok(())
}

fn verify_code(http: &Http, cfg: &Config, paths: &Paths, email: &str, code: &str, emit: &impl Fn(Event)) -> Result<(), String> {
    let body = serde_json::to_string(&json!({ "type": "email", "email": email.trim(), "token": code.trim() })).map_err(|e| e.to_string())?;
    let (status, text) = http.request_json("POST", &format!("{}/auth/v1/verify", cfg.url), &anon_headers(cfg), Some(&body))?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "That code didn't work."));
    }
    finish_session(http, cfg, paths, &text, emit)
}

fn finish_session(http: &Http, cfg: &Config, paths: &Paths, text: &str, emit: &impl Fn(Event)) -> Result<(), String> {
    let mut stored = session_from_auth(text)?;
    stored = fetch_subscription(http, cfg, stored)?;
    save(paths, &stored)?;
    emit(Event::Account(stored.snapshot(true, false, None)));
    emit(Event::AccountNotice { text: "Signed in. Apps still work if you sign out.".into(), error: false });
    Ok(())
}

fn google_sign_in(http: &Http, cfg: &Config, paths: &Paths, emit: &impl Fn(Event)) -> Result<(), String> {
    let verifier = pkce_verifier()?;
    let challenge = pkce_challenge(&verifier);
    let redirect = format!("http://127.0.0.1:{OAUTH_PORT}/callback");
    let auth = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}&code_challenge={}&code_challenge_method=S256&apikey={}",
        cfg.url,
        query_encode(&redirect),
        query_encode(&challenge),
        query_encode(&cfg.anon_key)
    );
    let listener = TcpListener::bind(("127.0.0.1", OAUTH_PORT)).map_err(|_| "Couldn't start Google sign-in (port 48174 in use).".to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    process::open(std::ffi::OsStr::new(&auth)).map_err(|e| format!("Couldn't open the browser ({e})"))?;
    emit(Event::AccountNotice { text: "A Google sign-in window opened in your browser.".into(), error: false });
    let mut stream = accept_oauth(&listener)?;
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let code = read_oauth_code(&mut stream)?;
    write_close_page(&mut stream);
    let body = serde_json::to_string(&json!({ "auth_code": code, "code_verifier": verifier })).map_err(|e| e.to_string())?;
    let (status, text) = http.request_json("POST", &format!("{}/auth/v1/token?grant_type=pkce", cfg.url), &anon_headers(cfg), Some(&body))?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "Google sign-in didn't finish."));
    }
    finish_session(http, cfg, paths, &text, emit)
}

fn checkout(http: &Http, cfg: &Config, paths: &Paths, interval: BillingInterval, emit: &impl Fn(Event)) -> Result<(), String> {
    let stored = restore_session(http, cfg, paths)?;
    let body = serde_json::to_string(&json!({ "interval": interval.label() })).map_err(|e| e.to_string())?;
    let bearer = format!("Bearer {}", stored.access_token);
    let headers = [("apikey", cfg.anon_key.as_str()), ("Authorization", bearer.as_str())];
    let (status, text) = http.request_json("POST", &format!("{}/functions/v1/payfast-checkout", cfg.url), &headers, Some(&body))?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "Couldn't start iCreate Pro checkout."));
    }
    let parsed: Value = serde_json::from_str(&text).map_err(|_| "Couldn't start iCreate Pro checkout.".to_string())?;
    let action = parsed.get("action").and_then(Value::as_str).ok_or("Couldn't start iCreate Pro checkout.")?;
    let fields = parsed.get("fields").and_then(Value::as_object).ok_or("Couldn't start iCreate Pro checkout.")?;
    let html = checkout_html(action, fields);
    let dest = paths.cache.join("payfast-checkout.html");
    crate::paths::write_atomic(&dest, html.as_bytes())?;
    process::open(dest.as_os_str()).map_err(|e| format!("Couldn't open PayFast ({e})"))?;
    emit(Event::AccountNotice { text: "PayFast opened in your browser. Come back here after you pay.".into(), error: false });
    emit(Event::Account(stored.snapshot(true, false, None)));
    Ok(())
}

fn backup(http: &Http, cfg: &Config, paths: &Paths, settings_json: &str, emit: &impl Fn(Event)) -> Result<(), String> {
    let stored = restore_session(http, cfg, paths)?;
    if !stored.snapshot(true, false, None).is_pro(crate::now()) {
        return Err("iCreate Pro is needed for cloud backup.".into());
    }
    let payload: Value = serde_json::from_str(settings_json).map_err(|_| "Couldn't read settings to back up.".to_string())?;
    let body = serde_json::to_string(&json!({ "user_id": stored.user_id, "payload": payload })).map_err(|e| e.to_string())?;
    let bearer = format!("Bearer {}", stored.access_token);
    let prefer = "return=minimal,resolution=merge-duplicates";
    let headers = [("apikey", cfg.anon_key.as_str()), ("Authorization", bearer.as_str()), ("Prefer", prefer)];
    let (status, text) = http.request_json("POST", &format!("{}/rest/v1/settings_backups", cfg.url), &headers, Some(&body))?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "Couldn't back up settings."));
    }
    emit(Event::AccountNotice { text: "Settings backed up to iCreate".into(), error: false });
    emit(Event::Account(stored.snapshot(true, false, None)));
    Ok(())
}

fn restore_settings(http: &Http, cfg: &Config, paths: &Paths, emit: &impl Fn(Event)) -> Result<(), String> {
    let stored = restore_session(http, cfg, paths)?;
    if !stored.snapshot(true, false, None).is_pro(crate::now()) {
        return Err("iCreate Pro is needed for cloud backup.".into());
    }
    let bearer = format!("Bearer {}", stored.access_token);
    let headers = [("apikey", cfg.anon_key.as_str()), ("Authorization", bearer.as_str()), ("Accept", "application/json")];
    let url = format!("{}/rest/v1/settings_backups?select=payload&user_id=eq.{}", cfg.url, stored.user_id);
    let (status, text) = http.request_json("GET", &url, &headers, None)?;
    if !(200..300).contains(&status) {
        return Err(api_error(&text, "Couldn't restore settings."));
    }
    let rows: Value = serde_json::from_str(&text).unwrap_or(Value::Array(vec![]));
    let payload = rows.as_array().and_then(|a| a.first()).and_then(|row| row.get("payload")).cloned();
    let Some(payload) = payload else {
        return Err("No cloud backup yet.".into());
    };
    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    emit(Event::AccountSettingsRestored { json });
    emit(Event::AccountNotice { text: "Restored settings from iCreate".into(), error: false });
    emit(Event::Account(stored.snapshot(true, false, None)));
    Ok(())
}

fn restore_session(http: &Http, cfg: &Config, paths: &Paths) -> Result<Stored, String> {
    let stored = load(paths).ok_or("Sign in first.")?;
    if stored.expires_at > crate::now() + 60 {
        return Ok(stored);
    }
    refresh(http, cfg, paths, stored)
}

fn refresh(http: &Http, cfg: &Config, paths: &Paths, stored: Stored) -> Result<Stored, String> {
    let body = serde_json::to_string(&json!({ "refresh_token": stored.refresh_token })).map_err(|e| e.to_string())?;
    let (status, text) =
        http.request_json("POST", &format!("{}/auth/v1/token?grant_type=refresh_token", cfg.url), &anon_headers(cfg), Some(&body))?;
    if !(200..300).contains(&status) {
        let _ = std::fs::remove_file(paths.account_session());
        return Err(api_error(&text, "Session expired. Sign in again. Apps still work."));
    }
    let mut next = session_from_auth(&text)?;
    next.plan = stored.plan;
    next.interval = stored.interval;
    next.status = stored.status;
    next.period_end = stored.period_end;
    save(paths, &next)?;
    Ok(next)
}

fn fetch_subscription(http: &Http, cfg: &Config, mut stored: Stored) -> Result<Stored, String> {
    let bearer = format!("Bearer {}", stored.access_token);
    let headers = [("apikey", cfg.anon_key.as_str()), ("Authorization", bearer.as_str()), ("Accept", "application/json")];
    let url = format!("{}/rest/v1/subscriptions?select=plan,billing_interval,status,current_period_end&user_id=eq.{}", cfg.url, stored.user_id);
    let (status, text) = match http.request_json("GET", &url, &headers, None) {
        Ok(pair) => pair,
        Err(e) => {
            // Offline: keep the cached plan so Pro extras still show from the last known state.
            log::info!("account: using saved subscription ({e})");
            return Ok(stored);
        }
    };
    if status == 401 {
        return Ok(stored);
    }
    if !(200..300).contains(&status) {
        return Ok(stored);
    }
    let rows: Value = serde_json::from_str(&text).unwrap_or(Value::Array(vec![]));
    let Some(row) = rows.as_array().and_then(|a| a.first()) else {
        stored.plan = Plan::Free;
        stored.status = SubscriptionStatus::Inactive;
        stored.interval = None;
        stored.period_end = None;
        return Ok(stored);
    };
    stored.plan = match row.get("plan").and_then(Value::as_str) {
        Some("pro") => Plan::Pro,
        _ => Plan::Free,
    };
    stored.interval = match row.get("billing_interval").and_then(Value::as_str) {
        Some("year") => Some(BillingInterval::Year),
        Some("month") => Some(BillingInterval::Month),
        _ => None,
    };
    stored.status = match row.get("status").and_then(Value::as_str) {
        Some("active") => SubscriptionStatus::Active,
        Some("cancelled") => SubscriptionStatus::Cancelled,
        Some("past_due") => SubscriptionStatus::PastDue,
        _ => SubscriptionStatus::Inactive,
    };
    stored.period_end = row.get("current_period_end").and_then(Value::as_str).and_then(parse_rfc3339);
    Ok(stored)
}

fn session_from_auth(text: &str) -> Result<Stored, String> {
    let v: Value = serde_json::from_str(text).map_err(|_| "Couldn't read the sign-in reply.".to_string())?;
    let access = v.get("access_token").and_then(Value::as_str).unwrap_or("").to_string();
    let refresh = v.get("refresh_token").and_then(Value::as_str).unwrap_or("").to_string();
    if access.is_empty() || refresh.is_empty() {
        return Err("Couldn't read the sign-in reply.".into());
    }
    let expires_in = v.get("expires_in").and_then(Value::as_i64).unwrap_or(3600);
    let user = v.get("user").cloned().unwrap_or(Value::Null);
    let user_id = user.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    let email = user.get("email").and_then(Value::as_str).unwrap_or("").to_string();
    if user_id.is_empty() {
        return Err("Couldn't read the sign-in reply.".into());
    }
    Ok(Stored {
        access_token: access,
        refresh_token: refresh,
        expires_at: crate::now().saturating_add(expires_in),
        user_id,
        email,
        plan: Plan::Free,
        interval: None,
        status: SubscriptionStatus::Inactive,
        period_end: None,
    })
}

fn load(paths: &Paths) -> Option<Stored> {
    let text = std::fs::read_to_string(paths.account_session()).ok()?;
    serde_json::from_str(&text).ok()
}

fn save(paths: &Paths, stored: &Stored) -> Result<(), String> {
    let text = serde_json::to_string(stored).map_err(|e| e.to_string())?;
    let path = paths.account_session();
    crate::paths::write_atomic(&path, text.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, perms);
        }
    }
    Ok(())
}

fn anon_headers(cfg: &Config) -> [(&str, &str); 2] {
    [("apikey", cfg.anon_key.as_str()), ("Authorization", cfg.anon_bearer.as_str())]
}

fn accept_oauth(listener: &TcpListener) -> Result<TcpStream, String> {
    let start = std::time::Instant::now();
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if start.elapsed() > OAUTH_WAIT {
                    return Err("Google sign-in timed out. Apps still work without an account.".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("Google sign-in didn't finish ({e})")),
        }
    }
}

fn api_error(body: &str, fallback: &str) -> String {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    v.get("error_description")
        .or_else(|| v.get("msg"))
        .or_else(|| v.get("message"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map_or_else(|| fallback.to_string(), str::to_string)
}

fn email_ok(email: &str) -> bool {
    let at = email.find('@').unwrap_or(0);
    at > 0 && email[at..].contains('.') && !email.contains(' ')
}

fn query_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn pkce_verifier() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| "Couldn't start Google sign-in.".to_string())?;
    Ok(b64url(&bytes))
}

fn pkce_challenge(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

fn b64url(bytes: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = (u32::from(bytes[i]) << 16) | (u32::from(bytes[i + 1]) << 8) | u32::from(bytes[i + 2]);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push(T[(n & 63) as usize] as char);
        i += 3;
    }
    let rest = bytes.len() - i;
    if rest == 1 {
        let n = u32::from(bytes[i]) << 16;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
    } else if rest == 2 {
        let n = (u32::from(bytes[i]) << 16) | (u32::from(bytes[i + 1]) << 8);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
    }
    out
}

fn read_oauth_code(stream: &mut TcpStream) -> Result<String, String> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).map_err(|e| format!("Google sign-in didn't finish ({e})"))?;
    let text = String::from_utf8_lossy(&buf[..n]);
    let line = text.lines().next().unwrap_or("");
    let path = line.split_whitespace().nth(1).unwrap_or("");
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    for pair in query.split('&') {
        if let Some(err) = pair.strip_prefix("error=") {
            return Err(format!("Google sign-in didn't finish ({err})."));
        }
        if let Some(code) = pair.strip_prefix("code=") {
            let code = code.split('#').next().unwrap_or(code);
            if !code.is_empty() {
                return Ok(code.to_string());
            }
        }
    }
    Err("Google sign-in didn't finish.".into())
}

fn write_close_page(stream: &mut TcpStream) {
    let body = b"<html><body><p>You can close this tab and return to iCreate.</p></body></html>";
    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn checkout_html(action: &str, fields: &serde_json::Map<String, Value>) -> String {
    let mut inputs = String::new();
    for (name, value) in fields {
        let Some(v) = value.as_str() else { continue };
        inputs.push_str(&format!("<input type=\"hidden\" name=\"{}\" value=\"{}\">\n", html_escape(name), html_escape(v)));
    }
    format!(
        "<!DOCTYPE html><html><body><p>Redirecting to PayFast…</p><form id=\"pay\" method=\"POST\" action=\"{}\">{inputs}</form><script>document.getElementById('pay').submit()</script></body></html>",
        html_escape(action)
    )
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_url_safe() {
        let challenge = pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk");
        assert!(challenge.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert!(!challenge.contains('='));
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn oauth_callback_reads_the_code() {
        let req = "GET /callback?code=abc-123&scope=email HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        let mut stream = mock_read(req);
        assert_eq!(read_oauth_code(&mut stream).unwrap(), "abc-123");
    }

    #[test]
    fn html_escape_quotes_hidden_fields() {
        assert_eq!(html_escape(r#"a&b<"c""#), "a&amp;b&lt;&quot;c&quot;");
    }

    fn mock_read(text: &str) -> TcpStream {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let payload = text.to_string();
        std::thread::spawn(move || {
            let mut client = std::net::TcpStream::connect(addr).unwrap();
            let _ = client.write_all(payload.as_bytes());
        });
        let (server, _) = listener.accept().unwrap();
        server
    }
}
