//! A compiler from Impostor AST to an axum router.

use std::collections::HashMap;

use axum::{
    extract::Request,
    http::{HeaderMap, Method},
    routing::{MethodFilter, MethodRouter},
};
use impostor_core::ast::ImpostorFile;

use crate::entry::Entry;

mod asserts;
mod entry;
mod error;
pub(crate) mod template;

/// Compile the AST for a complete Impostor File into an axum router.
pub fn compile_ast(ast: ImpostorFile) -> error::Result<axum::Router> {
    let entries: Vec<Entry> = ast
        .entries
        .into_iter()
        .map(|entry| {
            entry
                .try_into()
                .map_err(error::Error::EntryCompilationError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut routes_to_entries: HashMap<(String, Method), Vec<Entry>> = HashMap::new();

    for entry in entries {
        let route = entry.path.clone();
        let method = entry.method.clone();
        let entries_for_this_route = routes_to_entries
            .entry((route, method))
            .or_insert(Vec::new());
        entries_for_this_route.push(entry);
    }

    let mut router = axum::Router::new();

    for ((route, method), entries) in routes_to_entries {
        let method_filter: MethodFilter = match method.try_into() {
            Ok(method_filter) => method_filter,
            Err(e) => return Err(error::Error::InvalidMethod(Box::new(e))),
        };

        router = router.route(
            &route,
            MethodRouter::<()>::new().on(method_filter, |request: Request| async {
                for entry in entries {
                    let asserts_passed = entry.matches(&request);
                    if asserts_passed {
                        return entry.handler(request);
                    }
                }

                (axum::http::StatusCode::NOT_FOUND, HeaderMap::new(), vec![])
            }),
        );
    }

    Ok(router)
}

/// Compile an Impostor file into an axum router.
pub fn compile(contents: &str) -> error::Result<axum::Router> {
    let parse_result = impostor_core::parser::parse_impostor_file(contents);
    let ast = match parse_result {
        Ok(ast) => ast,
        Err(e) => return Err(error::Error::ParseError(e)),
    };

    compile_ast(ast)
}

/// Trim the first and last characters from a string if they match any of ', "
/// or `.
///
/// This is useful because the parsed AST will have quotes around strings for
/// asserts like `header "foo" == "bar"` will have the value `"bar"` with quotes
/// around it. This trims single quotes, double quotes, and backticks.
pub(crate) fn possibly_trim_surrounding_quotes(s: String) -> String {
    let mut chars = s.chars();
    let first_char = chars.next();
    let last_char = chars.next_back();

    match (first_char, last_char) {
        (Some('\''), Some('\'')) => s[1..s.len() - 1].to_string(),
        (Some('"'), Some('"')) => s[1..s.len() - 1].to_string(),
        (Some('`'), Some('`')) => s[1..s.len() - 1].to_string(),
        _ => s,
    }
}

#[cfg(test)]
mod test {
    use axum::body::Body;

    use super::*;

    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_compile() {
        let contents = r#"
            GET /hello

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);
    }

    #[tokio::test]
    async fn test_compile_two_routes() {
        let contents = r#"
            GET /hello-json

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}

            GET /hello-text

            HTTP 400
            Content-Type: text/plain; charset=utf-8

            # The backticks are necessary for the parse to take this as a string
            `hello world`
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello-json")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);

        let request = axum::http::Request::builder()
            .uri("/hello-text")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), b"hello world");
    }

    #[tokio::test]
    async fn test_compile_route_with_header_assert() {
        let contents = r#"
            GET /hello
            [Asserts]
            header "x-foo" == "bar"

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "bar")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "baz")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

        let request = axum::http::Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_compile_route_with_query_param_assert() {
        let contents = r#"
            GET /hello
            [Asserts]
            queryparam "foo" == "bar"

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello?foo=bar")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);

        let request = axum::http::Request::builder()
            .uri("/hello?foo=baz")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

        let request = axum::http::Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_compile_route_with_implicit_header_assert() {
        let contents = r#"
            GET /hello
            x-foo: bar

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "bar")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);

        let request = axum::http::Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_compile_multiple_entries_for_route() {
        let contents = r#"
            GET /hello
            x-foo: bar

            HTTP 200
            Content-Type: application/json

            {"hello": "world"}

            GET /hello
            [Asserts]
            header "x-foo" == "baz"

            HTTP 201
            Content-Type: text/plain; charset=utf-8

            `hello world`

            GET /hello

            HTTP 400
            Content-Type: text/plain; charset=utf-8

            `Expected x-foo header to be bar or baz`
        "#;

        let router = compile(contents).unwrap();

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "bar")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), br#"{"hello": "world"}"#);

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "baz")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::CREATED);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_as_bytes.as_ref(), b"hello world");

        let request = axum::http::Request::builder()
            .uri("/hello")
            .header("x-foo", "qux")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            body_as_bytes.as_ref(),
            b"Expected x-foo header to be bar or baz"
        );

        let request = axum::http::Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        let body_as_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            body_as_bytes.as_ref(),
            b"Expected x-foo header to be bar or baz"
        );
    }

    #[test]
    fn test_trim_quotes_for_single_quotes() {
        let result = possibly_trim_surrounding_quotes("'foo'".to_string());
        assert_eq!(result, "foo".to_string());
    }

    #[test]
    fn test_trim_quotes_for_double_quotes() {
        let result = possibly_trim_surrounding_quotes("\"foo\"".to_string());
        assert_eq!(result, "foo".to_string());
    }

    #[test]
    fn test_trim_quotes_for_backticks() {
        let result = possibly_trim_surrounding_quotes("`foo`".to_string());
        assert_eq!(result, "foo".to_string());
    }

    #[test]
    fn test_trim_quotes_for_no_quotes() {
        let result = possibly_trim_surrounding_quotes("foo".to_string());
        assert_eq!(result, "foo".to_string());
    }
}
