use std::collections::HashMap;

use axum::extract::Request;
use axum_extra::extract::CookieJar;
use impostor_core::ast::{Query as AstQuery, QueryValue as AstQueryValue};

use crate::possibly_trim_surrounding_quotes;

use super::AssertCompilationError;

#[derive(Clone, Debug)]
pub(super) enum Query {
    Path,
    Header(String),
    Cookie(String),
    Body,
    Jsonpath(String),
    QueryParam(String),
}

impl TryFrom<AstQuery> for Query {
    type Error = AssertCompilationError;

    fn try_from(value: AstQuery) -> Result<Self, Self::Error> {
        match value.value {
            AstQueryValue::Path => Ok(Query::Path),
            AstQueryValue::Header { name, .. } => Ok(Query::Header(
                possibly_trim_surrounding_quotes(name.encoded()),
            )),
            AstQueryValue::Cookie { expr, .. } => Ok(Query::Cookie(
                possibly_trim_surrounding_quotes(expr.name.encoded()),
            )),
            AstQueryValue::Body => Ok(Query::Body),
            AstQueryValue::Jsonpath { expr, .. } => Ok(Query::Jsonpath(
                possibly_trim_surrounding_quotes(expr.encoded()),
            )),
            AstQueryValue::QueryParam { name, .. } => Ok(Query::QueryParam(
                possibly_trim_surrounding_quotes(name.encoded()),
            )),
            _ => Err(AssertCompilationError::InvalidQueryType),
        }
    }
}

#[derive(Debug)]
pub(super) enum QueryApplicationError {
    InvalidHeaderValue(String, Box<dyn std::error::Error>),
    InvalidQueryParams(String),
}

impl std::fmt::Display for QueryApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QueryApplicationError::InvalidHeaderValue(name, message) => {
                write!(f, "invalid value in header {} - {}", name, message)
            }
            QueryApplicationError::InvalidQueryParams(message) => {
                write!(f, "invalid query params - {}", message)
            }
        }
    }
}

impl std::error::Error for QueryApplicationError {}

impl Query {
    pub(super) fn apply(
        &self,
        request: &Request,
    ) -> Result<serde_json::Value, QueryApplicationError> {
        let serialized = match self {
            Query::Path => serde_json::Value::String(request.uri().path().to_string()),
            Query::Header(name) => {
                if let Some(header_value) = request.headers().get(name) {
                    let header_as_str = header_value.to_str().map_err(|e| {
                        QueryApplicationError::InvalidHeaderValue(name.clone(), Box::new(e))
                    })?;
                    serde_json::Value::String(header_as_str.to_string())
                } else {
                    serde_json::Value::Null
                }
            }
            Query::Cookie(name) => {
                let cookies = CookieJar::from_headers(request.headers());
                if let Some(cookie_value) = cookies.get(name) {
                    serde_json::Value::String(cookie_value.value_trimmed().to_string())
                } else {
                    serde_json::Value::Null
                }
            }
            Query::Body => todo!(),
            Query::Jsonpath(_) => todo!(),
            Query::QueryParam(name) => {
                let query_string = match request.uri().query() {
                    Some(query_string) => query_string,
                    None => return Ok(serde_json::Value::Null),
                };

                let query_params =
                    serde_qs::from_str::<HashMap<String, serde_json::Value>>(query_string);
                let query_params = match query_params {
                    Ok(query_params) => query_params,
                    Err(e) => {
                        return Err(QueryApplicationError::InvalidQueryParams(format!(
                            "failed to parse query params: {}",
                            e
                        )))
                    }
                };

                let query_param = query_params.get(name);
                match query_param {
                    Some(query_param) => query_param.clone(),
                    None => serde_json::Value::Null,
                }
            }
        };

        Ok(serialized)
    }
}

#[cfg(test)]
mod test {
    use axum::body::Body;

    use super::*;

    fn create_test_request() -> Request<Body> {
        Request::builder()
            .uri("http://localhost:3000/?foo=bar&baz=qux")
            .header("x-foo", "bar")
            .header("x-baz", "qux")
            .header("cookie", "foo=bar; baz=qux")
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_query_application_for_query_param_foo() {
        let request = create_test_request();
        let query = Query::QueryParam("foo".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("bar".to_string()));
    }

    #[test]
    fn test_query_application_for_query_param_baz() {
        let request = create_test_request();
        let query = Query::QueryParam("baz".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("qux".to_string()));
    }

    #[test]
    fn test_query_application_for_nonexistent_query_param() {
        let request = create_test_request();
        let query = Query::QueryParam("quux".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_query_application_for_header_x_foo() {
        let request = create_test_request();
        let query = Query::Header("x-foo".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("bar".to_string()));
    }

    #[test]
    fn test_query_application_for_header_x_baz() {
        let request = create_test_request();
        let query = Query::Header("x-baz".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("qux".to_string()));
    }

    #[test]
    fn test_query_application_for_nonexistent_header() {
        let request = create_test_request();
        let query = Query::Header("x-quux".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_query_application_for_cookie_foo() {
        let request = create_test_request();
        let query = Query::Cookie("foo".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("bar".to_string()));
    }

    #[test]
    fn test_query_application_for_cookie_baz() {
        let request = create_test_request();
        let query = Query::Cookie("baz".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::String("qux".to_string()));
    }

    #[test]
    fn test_query_application_for_nonexistent_cookie() {
        let request = create_test_request();
        let query = Query::Cookie("quux".to_string());
        let result = query.apply(&request).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }
}
