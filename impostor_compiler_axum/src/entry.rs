use std::str::FromStr;

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue},
};
use impostor_core::ast::{Bytes as AstBytes, Entry as AstEntry};

use crate::{
    asserts::{Assert, AssertCompilationError},
    template::StringOrTemplate,
};

#[derive(Debug)]
pub enum EntryCompilationError {
    InvalidStatusCode(u16),
    InvalidMethod(String),
    InvalidHeaderName(Box<dyn std::error::Error + Send + Sync>),
    AssertCompilationError(AssertCompilationError),
    NotYetImplemented(String),
}

impl std::fmt::Display for EntryCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntryCompilationError::InvalidStatusCode(status_code) => {
                write!(f, "invalid status code: {}", status_code)
            }
            EntryCompilationError::InvalidMethod(method) => {
                write!(f, "invalid method: {}", method)
            }
            EntryCompilationError::NotYetImplemented(message) => {
                write!(f, "not yet implemented: {}", message)
            }
            EntryCompilationError::InvalidHeaderName(e) => {
                write!(f, "invalid header name: {}", e)
            }
            EntryCompilationError::AssertCompilationError(e) => write!(f, "invalid assert: {}", e),
        }
    }
}

/// An entry from the Impostor AST compiled for use as an axum handler.
#[derive(Clone, Debug)]
pub(crate) struct Entry {
    // Attributes required for routing
    pub path: String,
    pub method: axum::http::Method,

    // Attributes required for matching the request
    asserts: Vec<Assert>,

    // Attributes required for constructing the response
    status_code: axum::http::StatusCode,
    headers: Vec<(HeaderName, StringOrTemplate)>,
    body: Option<StringOrTemplate>,
}

impl TryFrom<AstEntry> for Entry {
    type Error = EntryCompilationError;

    fn try_from(entry: AstEntry) -> Result<Entry, EntryCompilationError> {
        let path = entry.request.path.to_string();
        let method = match axum::http::Method::from_str(&entry.request.method.0) {
            Ok(method) => method,
            Err(_) => return Err(EntryCompilationError::InvalidMethod(entry.request.method.0)),
        };

        let status_code = match axum::http::StatusCode::from_u16(entry.response.status.value) {
            Ok(status_code) => status_code,
            Err(_) => {
                return Err(EntryCompilationError::InvalidStatusCode(
                    entry.response.status.value,
                ))
            }
        };

        let mut headers = Vec::new();
        for header in entry.response.headers {
            let header_name = match HeaderName::from_str(&header.key.encoded()) {
                Ok(header_name) => header_name,
                Err(e) => return Err(EntryCompilationError::InvalidHeaderName(Box::new(e))),
            };
            headers.push((
                header_name,
                StringOrTemplate::from_ast_template(header.value),
            ));
        }
        let body = entry.response.body.map(|body| compile_body(body.value));
        let asserts: Result<Vec<Assert>, AssertCompilationError> = entry
            .request
            .asserts()
            .iter()
            .map(|assert| assert.clone().try_into())
            .chain(
                entry
                    .request
                    .headers
                    .iter()
                    .map(|header| header.clone().try_into()),
            )
            .collect();
        let asserts = asserts.map_err(EntryCompilationError::AssertCompilationError)?;

        Ok(Entry {
            path,
            method,
            status_code,
            asserts,
            headers,
            body,
        })
    }
}

struct InternalError {
    inner_error: Box<dyn std::fmt::Display>,
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "internal error: {}", self.inner_error)
    }
}

impl Entry {
    /// Handle an axum request with this entry.
    ///
    /// The returned tuple implements the axum `IntoResponse` trait, so it can
    /// be returned directly from an axum handler.
    pub fn handler(&self, _request: Request) -> (axum::http::StatusCode, HeaderMap, Vec<u8>) {
        let status_code = self.status_code;

        let headers: Result<HeaderMap, _> = self
            .headers
            .iter()
            .map(|(k, v)| {
                let header_name = k.to_owned();
                let header_value = v.execute();
                let header_value = match header_value {
                    Ok(header_value) => {
                        HeaderValue::from_str(&header_value).map_err(|e| InternalError {
                            inner_error: Box::new(e),
                        })
                    }
                    Err(e) => Err(InternalError {
                        inner_error: Box::new(e),
                    }),
                };
                match header_value {
                    Ok(header_value) => Ok((header_name, header_value)),
                    Err(e) => Err(e),
                }
            })
            .collect();
        let headers = match headers {
            Ok(headers) => headers,
            Err(e) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::new(),
                    format!("template error: {}", e).into(),
                )
            }
        };

        if let Some(body) = &self.body {
            match body.execute() {
                Ok(string_body) => (status_code, headers, string_body.into_bytes()),
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::new(),
                    format!("template error: {}", e).into(),
                ),
            }
        } else {
            (self.status_code, headers, vec![])
        }
    }

    /// Check if this entry matches a request.
    ///
    /// An entry matches a request if all of its asserts pass. Matching the
    /// request path isn't handled here, and should be handled by the axum
    /// router.
    pub fn matches(&self, request: &Request) -> bool {
        self.asserts
            .iter()
            // TODO: Log failures
            .all(|a| a.apply(request).is_ok_and(|r| r))
    }
}

fn compile_body(body: AstBytes) -> StringOrTemplate {
    match body {
        AstBytes::Json(val) => StringOrTemplate::String(val.encoded()),
        AstBytes::Xml(val) => StringOrTemplate::String(val),
        AstBytes::MultilineString(val) => val.value().into(),
        AstBytes::OnelineString(val) => val.into(),
        AstBytes::Base64(_) => todo!(),
        AstBytes::File(_) => todo!(),
        AstBytes::Hex(_) => todo!(),
    }
}
