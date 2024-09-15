use std::{collections::HashMap, fmt::Display};

use actix_http::{
    header::{HeaderMap, HeaderValue, TryIntoHeaderPair},
    Method, StatusCode, Uri,
};
use actix_web::http::header;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tracing::info;
use url::{form_urlencoded, Url};

pub trait Body {
    fn get_body(&self) -> Bytes;
}

pub struct JsonBody<'a, T: Serialize>(&'a T);

pub struct FormDataBody<'a>(HashMap<&'a str, &'a str>);

impl<'a> Body for FormDataBody<'a> {
    fn get_body(&self) -> Bytes {
        let str = self.0.iter().fold(String::new(), |mut accum, kv| {
            if !accum.is_empty() {
                accum.push('&');
            }

            let mut accum =
                form_urlencoded::byte_serialize(kv.0.as_bytes()).fold(accum, |mut str, c| {
                    str.push_str(c);
                    str
                });
            accum.push('=');
            let accum =
                form_urlencoded::byte_serialize(kv.1.as_bytes()).fold(accum, |mut str, c| {
                    str.push_str(c);
                    str
                });
            accum
        });
        Bytes::from(str)
    }
}

impl<'a, T: Serialize> Body for JsonBody<'a, T> {
    fn get_body(&self) -> Bytes {
        let bytes = serde_json::to_string(&self.0).expect("Failed to serialize test data to json");
        bytes.into()
    }
}

impl Body for () {
    fn get_body(&self) -> Bytes {
        Bytes::from("")
    }
}

#[derive(Clone)]
pub struct TestHttpClient {
    client: awc::Client,
    base_uri: Url,
}

impl TestHttpClient {
    pub fn new(port: u16) -> TestHttpClient {
        TestHttpClient {
            client: Default::default(),
            base_uri: format!("http://127.0.0.1:{}/", port).parse().unwrap(),
        }
    }

    pub fn request(&self, method: Method, uri: &str) -> TestHttpRequest {
        TestHttpRequest::new(self).method(method).uri(uri)
    }
    pub fn get(&self, uri: &str) -> TestHttpRequest {
        self.request(Method::GET, uri)
    }
    pub fn post(&self, uri: &str) -> TestHttpRequest {
        self.request(Method::POST, uri)
    }
    pub fn put(&self, uri: &str) -> TestHttpRequest {
        self.request(Method::PUT, uri)
    }
    pub fn patch(&self, uri: &str) -> TestHttpRequest {
        self.request(Method::PATCH, uri)
    }
}

pub struct TestHttpRequest<B: Body = ()> {
    uri: String,
    method: Method,
    client: TestHttpClient,
    body: B,
    headers: HeaderMap,
}

impl TestHttpRequest<()> {
    pub fn new(client: &TestHttpClient) -> Self {
        Self {
            uri: "/".to_string(),
            method: Method::GET,
            client: client.clone(),
            body: (),
            headers: Default::default(),
        }
    }
}

impl<B: Body> TestHttpRequest<B> {
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn post(self) -> Self {
        self.method(Method::POST)
    }
    pub fn get(self) -> Self {
        self.method(Method::GET)
    }
    pub fn put(self) -> Self {
        self.method(Method::PUT)
    }
    pub fn patch(self) -> Self {
        self.method(Method::PATCH)
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Вставить заголовок с перезаписью при совпадении
    pub fn insert_header(mut self, header: impl TryIntoHeaderPair) -> Self {
        match header.try_into_pair() {
            Ok((key, value)) => {
                self.headers.insert(key, value);
            }
            Err(err) => {
                panic!("Error inserting test header: {}.", err.into());
            }
        }

        self
    }

    /// Вставить заголовок с перезаписью при совпадении
    pub fn access_token(self, token: &str) -> Self {
        self.insert_header((
            actix_http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
    }

    pub fn json<T: Serialize>(self, data: &T) -> TestHttpRequest<JsonBody<T>> {
        self.insert_header((
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        ))
        .body(JsonBody(data))
    }

    pub fn form_data<'a>(self, data: HashMap<&'a str, &'a str>) -> TestHttpRequest<FormDataBody> {
        self.insert_header((
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        ))
        .body(FormDataBody(data))
    }

    pub fn body<T: Body>(self, data: T) -> TestHttpRequest<T> {
        TestHttpRequest {
            body: data,
            uri: self.uri,
            method: self.method,
            client: self.client,
            headers: self.headers,
        }
    }

    pub async fn send(self) -> TestHttpResponse {
        let url: Uri = self
            .client
            .base_uri
            .join(&self.uri)
            .unwrap()
            .to_string()
            .parse()
            .unwrap();
        let body = self.body.get_body();

        let url_str = url.to_string();

        let mut req = self
            .client
            .client
            .request(self.method.clone(), url)
            .timeout(std::time::Duration::from_secs(10));
        for h in self.headers {
            req = req.insert_header(h);
        }

        let req_headers: LogHeaders = req.headers().clone().into();
        info!(
            method = self.method.to_string(),
            url = url_str,
            body = String::from_utf8_lossy(&body).to_string(),
            headers = req_headers.to_string(),
            "test request"
        );

        let mut response = req.send_body(body).await.unwrap();

        let body = response.body().await.unwrap();

        let response_headers: LogHeaders = response.headers().clone().into();
        info!(
            status = response.status().as_u16(),
            body = String::from_utf8_lossy(&body).to_string(),
            headers = response_headers.to_string(),
            "test response"
        );

        TestHttpResponse {
            status: response.status(),
            headers: response.headers().to_owned(),
            body,
        }
    }
}

struct LogHeaders(HeaderMap);

impl From<HeaderMap> for LogHeaders {
    fn from(value: HeaderMap) -> Self {
        Self(value)
    }
}

impl Display for LogHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (name, value)) in self.0.clone().into_iter().enumerate() {
            if index > 0 {
                write!(f, "\r\n")?;
            }
            write!(f, "{}: {}", name, value.to_str().unwrap())?;
        }
        Ok(())
    }
}

pub struct TestHttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl TestHttpResponse {
    /// Returns
    ///
    /// Ok, if 2xx
    ///
    /// Err, if another
    pub fn result<T: DeserializeOwned, E: DeserializeOwned>(&self) -> Result<T, E> {
        if self.status.is_success() {
            let response: T = serde_json::from_slice(&self.body).unwrap_or_else(|e| {
                panic!(
                    "unable to parse data {}: {}\n{}",
                    std::any::type_name::<T>(),
                    e,
                    String::from_utf8_lossy(&self.body)
                );
            });
            Ok(response)
        } else {
            let err: E = serde_json::from_slice(&self.body).unwrap_or_else(|e| {
                panic!(
                    "unable to parse error {}: {}\n{}",
                    std::any::type_name::<E>(),
                    e,
                    String::from_utf8_lossy(&self.body)
                );
            });
            Err(err)
        }
    }
}
