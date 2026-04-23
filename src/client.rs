use crate::config::ResolvedProfile;
use crate::error::{Error, Result};
use reqwest::blocking::{Client as ReqwestClient, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub enabled: bool,
    pub max_attempts: u32,
    pub initial_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_backoff: Duration::from_millis(500),
        }
    }
}

pub struct Client {
    pub(crate) http: ReqwestClient,
    pub(crate) base_url: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) retry: RetryPolicy,
}

pub struct ClientBuilder {
    timeout: Duration,
    user_agent: String,
    retry: RetryPolicy,
    proxy: Option<String>,
    no_proxy: Option<String>,
    insecure: bool,
    ca_cert: Option<String>,
    proxy_ca_cert: Option<String>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: format!("sn/{}", env!("CARGO_PKG_VERSION")),
            retry: RetryPolicy::default(),
            proxy: None,
            no_proxy: None,
            insecure: false,
            ca_cert: None,
            proxy_ca_cert: None,
            proxy_username: None,
            proxy_password: None,
        }
    }
}

impl ClientBuilder {
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    pub fn proxy(mut self, url: Option<String>) -> Self {
        self.proxy = url;
        self
    }

    pub fn no_proxy(mut self, hosts: Option<String>) -> Self {
        self.no_proxy = hosts;
        self
    }

    pub fn insecure(mut self, yes: bool) -> Self {
        self.insecure = yes;
        self
    }

    pub fn ca_cert(mut self, path: Option<String>) -> Self {
        self.ca_cert = path;
        self
    }

    pub fn proxy_ca_cert(mut self, path: Option<String>) -> Self {
        self.proxy_ca_cert = path;
        self
    }

    pub fn proxy_auth(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.proxy_username = username;
        self.proxy_password = password;
        self
    }

    pub fn build(self, profile: &ResolvedProfile) -> Result<Client> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent).unwrap());

        let mut builder = ReqwestClient::builder()
            .timeout(self.timeout)
            .default_headers(headers);

        if let Some(ref proxy_url) = self.proxy {
            let mut proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| Error::Config(format!("invalid proxy URL '{proxy_url}': {e}")))?;
            if let (Some(ref u), Some(ref p)) = (&self.proxy_username, &self.proxy_password) {
                proxy = proxy.basic_auth(u, p);
            }
            if let Some(ref hosts) = self.no_proxy {
                proxy = proxy.no_proxy(reqwest::NoProxy::from_string(hosts));
            }
            builder = builder.proxy(proxy);
        }

        if self.insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(ref path) = self.ca_cert {
            let pem = std::fs::read(path)
                .map_err(|e| Error::Config(format!("read CA cert '{}': {e}", path)))?;
            let cert = reqwest::Certificate::from_pem(&pem)
                .map_err(|e| Error::Config(format!("parse CA cert '{}': {e}", path)))?;
            builder = builder.add_root_certificate(cert);
        }

        if let Some(ref path) = self.proxy_ca_cert {
            let pem = std::fs::read(path)
                .map_err(|e| Error::Config(format!("read proxy CA cert '{}': {e}", path)))?;
            let cert = reqwest::Certificate::from_pem(&pem)
                .map_err(|e| Error::Config(format!("parse proxy CA cert '{}': {e}", path)))?;
            builder = builder.add_root_certificate(cert);
        }

        let http = builder
            .build()
            .map_err(|e| Error::Transport(format!("build client: {e}")))?;

        let base_url = normalize_base_url(&profile.instance);
        Ok(Client {
            http,
            base_url,
            username: profile.username.clone(),
            password: profile.password.clone(),
            retry: self.retry,
        })
    }
}

fn normalize_base_url(instance: &str) -> String {
    if instance.starts_with("http://") || instance.starts_with("https://") {
        instance.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", instance.trim_end_matches('/'))
    }
}

fn should_retry(status: StatusCode) -> bool {
    status.as_u16() == 429 || matches!(status.as_u16(), 502..=504)
}

fn jittered(d: Duration) -> Duration {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.subsec_nanos())
        .unwrap_or(0) as u64;
    let jitter_ms = nanos % 250;
    d + Duration::from_millis(jitter_ms)
}

fn execute_with_retry<F>(policy: RetryPolicy, mut send: F) -> Result<Value>
where
    F: FnMut() -> std::result::Result<Response, reqwest::Error>,
{
    let mut attempt: u32 = 0;
    let mut backoff = policy.initial_backoff;
    loop {
        attempt += 1;
        match send() {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp
                        .json::<Value>()
                        .map_err(|e| Error::Transport(format!("parse response: {e}")));
                }
                let retryable =
                    policy.enabled && should_retry(status) && attempt < policy.max_attempts;
                if !retryable {
                    let tx = transaction_id(&resp);
                    return Err(from_http(status, tx, resp));
                }
                std::thread::sleep(jittered(backoff));
                backoff = backoff.saturating_mul(2);
            }
            Err(e) => return Err(Error::Transport(format!("{e}"))),
        }
    }
}

fn execute_no_body_with_retry<F>(policy: RetryPolicy, mut send: F) -> Result<()>
where
    F: FnMut() -> std::result::Result<Response, reqwest::Error>,
{
    let mut attempt: u32 = 0;
    let mut backoff = policy.initial_backoff;
    loop {
        attempt += 1;
        match send() {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return Ok(());
                }
                let retryable =
                    policy.enabled && should_retry(status) && attempt < policy.max_attempts;
                if !retryable {
                    let tx = transaction_id(&resp);
                    return Err(from_http(status, tx, resp));
                }
                std::thread::sleep(jittered(backoff));
                backoff = backoff.saturating_mul(2);
            }
            Err(e) => return Err(Error::Transport(format!("{e}"))),
        }
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub fn get(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let query = query.to_vec();
        execute_with_retry(self.retry, move || {
            http.request(Method::GET, &url)
                .basic_auth(&user, Some(&pass))
                .query(&query)
                .send()
        })
    }

    pub fn post(&self, path: &str, query: &[(String, String)], body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let query = query.to_vec();
        let body = body.clone();
        execute_with_retry(self.retry, move || {
            http.request(Method::POST, &url)
                .basic_auth(&user, Some(&pass))
                .query(&query)
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
        })
    }

    pub fn put(&self, path: &str, query: &[(String, String)], body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let query = query.to_vec();
        let body = body.clone();
        execute_with_retry(self.retry, move || {
            http.request(Method::PUT, &url)
                .basic_auth(&user, Some(&pass))
                .query(&query)
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
        })
    }

    pub fn patch(&self, path: &str, query: &[(String, String)], body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let query = query.to_vec();
        let body = body.clone();
        execute_with_retry(self.retry, move || {
            http.request(Method::PATCH, &url)
                .basic_auth(&user, Some(&pass))
                .query(&query)
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
        })
    }

    pub fn delete(&self, path: &str, query: &[(String, String)]) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let query = query.to_vec();
        execute_no_body_with_retry(self.retry, move || {
            http.request(Method::DELETE, &url)
                .basic_auth(&user, Some(&pass))
                .query(&query)
                .send()
        })
    }
}

impl Client {
    /// Stream records from a paginated list endpoint, following Link: rel="next" headers.
    pub fn paginate(
        &self,
        initial_path: &str,
        initial_query: &[(String, String)],
        max_records: Option<u32>,
    ) -> Paginator<'_> {
        Paginator::new(
            self,
            initial_path.to_string(),
            initial_query.to_vec(),
            max_records,
        )
    }
}

pub struct Paginator<'a> {
    client: &'a Client,
    next_url: Option<String>,
    next_query: Vec<(String, String)>,
    buffer: std::collections::VecDeque<Value>,
    emitted: u32,
    cap: Option<u32>,
    finished: bool,
}

impl<'a> Paginator<'a> {
    fn new(
        client: &'a Client,
        path: String,
        query: Vec<(String, String)>,
        cap: Option<u32>,
    ) -> Self {
        Self {
            client,
            next_url: Some(format!("{}{path}", client.base_url)),
            next_query: query,
            buffer: std::collections::VecDeque::new(),
            emitted: 0,
            cap,
            finished: false,
        }
    }

    fn fetch_next_page(&mut self) -> Result<()> {
        let Some(url) = self.next_url.take() else {
            self.finished = true;
            return Ok(());
        };
        let req = self
            .client
            .http
            .request(Method::GET, &url)
            .basic_auth(&self.client.username, Some(&self.client.password))
            .query(&self.next_query);
        let resp =
            execute_request_with_retry(self.client.retry, || req.try_clone().unwrap().send())?;
        let status = resp.status();
        let tx = transaction_id(&resp);
        let link = resp
            .headers()
            .get("Link")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);
        if !status.is_success() {
            return Err(from_http(status, tx, resp));
        }
        let body: Value = resp
            .json()
            .map_err(|e| Error::Transport(format!("parse response: {e}")))?;
        if let Value::Array(records) = body.get("result").cloned().unwrap_or(Value::Array(vec![])) {
            for r in records {
                self.buffer.push_back(r);
            }
        }
        self.next_query.clear(); // next link carries all params
        self.next_url = link.and_then(parse_next_link);
        if self.next_url.is_none() {
            self.finished = true;
        }
        Ok(())
    }
}

fn parse_next_link(header: String) -> Option<String> {
    // ServiceNow Link: <...>;rel="next", <...>;rel="first", ...
    for part in header.split(',') {
        let part = part.trim();
        if let Some((url_part, rel_part)) = part.split_once(';') {
            let rel = rel_part.trim();
            if rel.contains("rel=\"next\"") {
                let u = url_part
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>');
                return Some(u.to_string());
            }
        }
    }
    None
}

impl<'a> Iterator for Paginator<'a> {
    type Item = Result<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cap) = self.cap {
            if cap != 0 && self.emitted >= cap {
                return None;
            }
        }
        if self.buffer.is_empty() && !self.finished {
            if let Err(e) = self.fetch_next_page() {
                self.finished = true;
                return Some(Err(e));
            }
        }
        match self.buffer.pop_front() {
            Some(v) => {
                self.emitted += 1;
                Some(Ok(v))
            }
            None => None,
        }
    }
}

fn execute_request_with_retry<F>(
    policy: RetryPolicy,
    mut send: F,
) -> std::result::Result<Response, Error>
where
    F: FnMut() -> std::result::Result<Response, reqwest::Error>,
{
    let mut attempt = 0u32;
    let mut backoff = policy.initial_backoff;
    loop {
        attempt += 1;
        match send() {
            Ok(resp) => {
                let status = resp.status();
                let retryable =
                    policy.enabled && should_retry(status) && attempt < policy.max_attempts;
                if !status.is_success() && retryable {
                    std::thread::sleep(jittered(backoff));
                    backoff = backoff.saturating_mul(2);
                    continue;
                }
                return Ok(resp);
            }
            Err(e) => return Err(Error::Transport(format!("{e}"))),
        }
    }
}

fn transaction_id(resp: &Response) -> Option<String> {
    resp.headers()
        .get("X-Transaction-ID")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
}

fn from_http(status: StatusCode, tx: Option<String>, resp: Response) -> Error {
    let body: Option<Value> = resp.json().ok();
    let (message, detail, sn_error) = body
        .as_ref()
        .and_then(|v| v.get("error"))
        .map(|err| {
            (
                err.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("ServiceNow error")
                    .to_string(),
                err.get("detail")
                    .and_then(|d| d.as_str())
                    .map(ToString::to_string),
                Some(err.clone()),
            )
        })
        .unwrap_or_else(|| (format!("HTTP {status}"), None, None));
    match status.as_u16() {
        401 | 403 => Error::Auth {
            status: status.as_u16(),
            message,
            transaction_id: tx,
        },
        s => Error::Api {
            status: s,
            message,
            detail,
            transaction_id: tx,
            sn_error,
        },
    }
}
