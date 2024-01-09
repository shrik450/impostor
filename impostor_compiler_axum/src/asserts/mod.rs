mod predicate;
mod query;

use std::fmt::Display;

use axum::extract::Request;
use serde_json::json;

use self::{predicate::Predicate, query::Query};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum AssertCompilationError {
    InvalidQueryType,
    InvalidPredicate,
    InvalidPredicateValue(String),
}

impl Display for AssertCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssertCompilationError::InvalidQueryType => write!(f, "invalid query type"),
            AssertCompilationError::InvalidPredicate => write!(f, "invalid predicate"),
            AssertCompilationError::InvalidPredicateValue(message) => {
                write!(f, "invalid predicate value - {}", message)
            }
        }
    }
}

impl std::error::Error for AssertCompilationError {}

#[derive(Debug)]
pub enum AssertionError {
    InvalidQueryValue(Box<dyn std::error::Error>),
}

impl Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssertionError::InvalidQueryValue(e) => write!(f, "invalid query value - {}", e),
        }
    }
}

impl std::error::Error for AssertionError {}

#[derive(Clone, Debug)]
pub(crate) struct Assert {
    query: Query,
    predicate: Predicate,
    not: bool,
}

impl TryFrom<impostor_core::ast::Assert> for Assert {
    type Error = AssertCompilationError;

    fn try_from(value: impostor_core::ast::Assert) -> Result<Self, Self::Error> {
        let query = value.query.try_into()?;
        let not = value.predicate.not;
        let predicate = value.predicate.try_into()?;

        Ok(Assert {
            query,
            predicate,
            not,
        })
    }
}

impl TryFrom<impostor_core::ast::Header> for Assert {
    type Error = AssertCompilationError;

    fn try_from(value: impostor_core::ast::Header) -> Result<Self, Self::Error> {
        let query = Query::Header(value.key.encoded());
        let not = false;
        let predicate = Predicate::Equal(json!(value.value.encoded()));

        Ok(Assert {
            query,
            predicate,
            not,
        })
    }
}

impl Assert {
    pub fn apply(&self, request: &Request) -> Result<bool, AssertionError> {
        let query_value = match self.query.apply(request) {
            Ok(value) => value,
            Err(e) => return Err(AssertionError::InvalidQueryValue(Box::new(e))),
        };
        let assertion_result = self.predicate.apply(&query_value);
        Ok(if self.not {
            !assertion_result
        } else {
            assertion_result
        })
    }
}

#[cfg(test)]
mod test {
    use axum::body::Body;

    use super::*;

    #[test]
    fn test_assert_header() {
        let assert = Assert {
            query: Query::Header("foo".to_string()),
            predicate: Predicate::Equal(json!("bar")),
            not: false,
        };

        let request = Request::builder()
            .header("foo", "bar")
            .body(Body::empty())
            .unwrap();
        assert!(assert.apply(&request).unwrap());

        let request = Request::builder()
            .header("foo", "baz")
            .body(Body::empty())
            .unwrap();
        assert!(!assert.apply(&request).unwrap());
    }

    #[test]
    fn test_assert_header_not() {
        let assert = Assert {
            query: Query::Header("foo".to_string()),
            predicate: Predicate::Equal(json!("bar")),
            not: true,
        };

        let request = Request::builder()
            .header("foo", "bar")
            .body(Body::empty())
            .unwrap();
        assert!(!assert.apply(&request).unwrap());

        let request = Request::builder()
            .header("foo", "baz")
            .body(Body::empty())
            .unwrap();
        assert!(assert.apply(&request).unwrap());
    }

    #[test]
    fn test_assert_query_param() {
        let assert = Assert {
            query: Query::QueryParam("foo".to_string()),
            predicate: Predicate::Equal(json!("bar")),
            not: false,
        };

        let request = Request::builder()
            .uri("http://localhost:3000/?foo=bar")
            .body(Body::empty())
            .unwrap();
        assert!(assert.apply(&request).unwrap());

        let request = Request::builder()
            .uri("http://localhost:3000/?foo=baz")
            .body(Body::empty())
            .unwrap();
        assert!(!assert.apply(&request).unwrap());
    }

    #[test]
    fn test_assert_header_value_invalid() {
        let assert = Assert {
            query: Query::Header("foo".to_string()),
            predicate: Predicate::Equal(json!("bar")),
            not: false,
        };

        let request = Request::builder()
            .header("foo", "世界")
            .body(Body::empty())
            .unwrap();
        assert!(assert.apply(&request).is_err());
    }
}
