use std::str::FromStr;

use axum::{extract::Request, http::Response};
use inko_core::ast::{Bytes, Entry};

use crate::template::StringOrTemplate;

#[derive(Debug)]
pub enum EntryCompilationError {
    InvalidStatusCode(u16),
    InvalidMethod(String),
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
        }
    }
}

/// An entry from the Inko AST compiled for use as an axum handler.
struct CompiledEntry {
    // Attributes required for routing
    path: String,
    method: axum::http::Method,

    // Attributes required for matching the request

    // Attributes required for constructing the response
    status_code: axum::http::StatusCode,
    headers: Vec<(String, StringOrTemplate)>,
    body: Option<StringOrTemplate>,
}

impl TryFrom<Entry> for CompiledEntry {
    type Error = EntryCompilationError;

    fn try_from(entry: Entry) -> Result<CompiledEntry, EntryCompilationError> {
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
            headers.push((
                header.key.to_string(),
                StringOrTemplate::from_ast_template(header.value),
            ));
        }
        let body = entry.response.body.map(|body| compile_body(body.value));

        Ok(CompiledEntry {
            path,
            method,
            status_code,
            headers,
            body,
        })
    }
}

impl CompiledEntry {
    async fn handler(&self, _request: Request) -> axum::http::Result<Response<Vec<u8>>> {
        let mut response = Response::builder();
        response = response.status(self.status_code);

        if let Some(body) = &self.body {
            match body.execute() {
                Ok(string_body) => response.body(string_body.into_bytes()),
                Err(e) => Response::builder()
                    .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("template error: {}", e).into()),
            }
        } else {
            response.body(Vec::new())
        }
    }
}

fn compile_body(body: Bytes) -> StringOrTemplate {
    match body {
        Bytes::Json(val) => StringOrTemplate::String(val.encoded()),
        Bytes::Xml(val) => StringOrTemplate::String(val),
        Bytes::MultilineString(val) => StringOrTemplate::from_ast_template(val.value()),
        Bytes::OnelineString(val) => StringOrTemplate::from_ast_template(val),
        Bytes::Base64(_) => todo!(),
        Bytes::File(_) => todo!(),
        Bytes::Hex(_) => todo!(),
    }
}
