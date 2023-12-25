mod predicate;
mod query;

use std::fmt::Display;

use axum::extract::Request;

use self::{predicate::Predicate, query::Query};

#[allow(clippy::enum_variant_names)]
pub(crate) enum AssertCompilationError {
    InvalidQueryType,
    InvalidPredicate,
    InvalidPredicateValue(String),
}

pub(crate) enum AssertionError {
    InvalidQueryValue(Box<dyn Display>),
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

pub(crate) struct Assert {
    query: Query,
    predicate: Predicate,
    not: bool,
}

impl TryFrom<inko_core::ast::Assert> for Assert {
    type Error = AssertCompilationError;

    fn try_from(value: inko_core::ast::Assert) -> Result<Self, Self::Error> {
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

impl Assert {
    fn apply(&self, request: &Request) -> Result<bool, AssertionError> {
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
