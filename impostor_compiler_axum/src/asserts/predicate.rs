use impostor_core::ast::{
    Predicate as AstPredicate, PredicateFuncValue as AstPredicateFuncValue,
    PredicateValue as AstPredicateValue,
};
use regex::Regex;

use crate::possibly_trim_surrounding_quotes;

use super::AssertCompilationError;

#[derive(Clone, Debug)]
pub(super) enum Number {
    Integer(i64),
    Float(f64),
    BigInteger(String),
}

#[derive(Clone, Debug)]
pub(super) enum Predicate {
    Equal(serde_json::Value),
    NotEqual(serde_json::Value),
    GreaterThan(Number),
    GreaterThanOrEqual(Number),
    LessThan(Number),
    LessThanOrEqual(Number),
    StartWith(String),
    EndWith(String),
    Contain(String),
    Include(serde_json::Value),
    Match(Regex),
    IsInteger,
    IsFloat,
    IsBoolean,
    IsString,
    IsCollection,
    Exist,
    IsEmpty,
}

impl TryFrom<AstPredicate> for Predicate {
    type Error = AssertCompilationError;

    fn try_from(value: AstPredicate) -> Result<Self, Self::Error> {
        Ok(match value.predicate_func.value {
            AstPredicateFuncValue::Equal { value, .. } => {
                Predicate::Equal(try_into_serde_value(value)?)
            }
            AstPredicateFuncValue::NotEqual { value, .. } => {
                Predicate::NotEqual(try_into_serde_value(value)?)
            }
            AstPredicateFuncValue::GreaterThan { value, .. } => {
                Predicate::GreaterThan(try_into_number(value)?)
            }
            AstPredicateFuncValue::GreaterThanOrEqual { value, .. } => {
                Predicate::GreaterThanOrEqual(try_into_number(value)?)
            }
            AstPredicateFuncValue::LessThan { value, .. } => {
                Predicate::LessThan(try_into_number(value)?)
            }
            AstPredicateFuncValue::LessThanOrEqual { value, .. } => {
                Predicate::LessThanOrEqual(try_into_number(value)?)
            }
            AstPredicateFuncValue::StartWith { value, .. } => {
                Predicate::StartWith(try_into_string(value)?)
            }
            AstPredicateFuncValue::EndWith { value, .. } => {
                Predicate::EndWith(try_into_string(value)?)
            }
            AstPredicateFuncValue::Contain { value, .. } => {
                Predicate::Contain(try_into_string(value)?)
            }
            AstPredicateFuncValue::Include { value, .. } => {
                Predicate::Include(try_into_serde_value(value)?)
            }
            AstPredicateFuncValue::Match { value, .. } => Predicate::Match(try_into_regex(value)?),
            AstPredicateFuncValue::IsInteger => Predicate::IsInteger,
            AstPredicateFuncValue::IsFloat => Predicate::IsFloat,
            AstPredicateFuncValue::IsBoolean => Predicate::IsBoolean,
            AstPredicateFuncValue::IsString => Predicate::IsString,
            AstPredicateFuncValue::IsCollection => Predicate::IsCollection,
            AstPredicateFuncValue::Exist => Predicate::Exist,
            AstPredicateFuncValue::IsEmpty => Predicate::IsEmpty,
            _ => return Err(AssertCompilationError::InvalidPredicate),
        })
    }
}

impl Predicate {
    pub(super) fn apply(&self, against: &serde_json::Value) -> bool {
        match self {
            Predicate::Equal(value) => compare_eq(value, against),
            Predicate::NotEqual(value) => compare_ne(value, against),
            // The next four are the other way around from the name!
            Predicate::GreaterThan(value) => compare_lt(value, against),
            Predicate::GreaterThanOrEqual(value) => compare_lteq(value, against),
            Predicate::LessThan(value) => compare_gt(value, against),
            Predicate::LessThanOrEqual(value) => compare_gteq(value, against),
            Predicate::StartWith(value) => compare_start_with(value, against),
            Predicate::EndWith(value) => compare_end_with(value, against),
            Predicate::Contain(value) => compare_contain(value, against),
            Predicate::Include(value) => compare_include(value, against),
            Predicate::Match(value) => compare_match(value, against),
            Predicate::IsInteger => compare_is_integer(against),
            Predicate::IsFloat => compare_is_float(against),
            Predicate::IsBoolean => compare_is_boolean(against),
            Predicate::IsString => compare_is_string(against),
            Predicate::IsCollection => compare_is_collection(against),
            Predicate::Exist => compare_exist(against),
            Predicate::IsEmpty => compare_is_empty(against),
        }
    }
}

fn compare_eq(first: &serde_json::Value, second: &serde_json::Value) -> bool {
    first == second
}

fn compare_ne(first: &serde_json::Value, second: &serde_json::Value) -> bool {
    first != second
}

fn compare_gt(first: &Number, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(second) => match first {
            Number::Integer(first) => {
                if second.is_i64() {
                    first > &second.as_i64().unwrap()
                } else {
                    first > &(second.as_f64().unwrap().floor() as i64)
                }
            }
            Number::Float(first) => {
                if second.is_i64() {
                    first.ceil() as i64 > second.as_i64().unwrap()
                } else {
                    first > &second.as_f64().unwrap()
                }
            }
            Number::BigInteger(_) => todo!(),
        },
        _ => false,
    }
}

fn compare_gteq(first: &Number, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(second) => match first {
            Number::Integer(first) => {
                if second.is_i64() {
                    first >= &second.as_i64().unwrap()
                } else {
                    first >= &(second.as_f64().unwrap().floor() as i64)
                }
            }
            Number::Float(first) => {
                if second.is_i64() {
                    first.ceil() as i64 >= second.as_i64().unwrap()
                } else {
                    first >= &second.as_f64().unwrap()
                }
            }
            Number::BigInteger(_) => todo!(),
        },
        _ => false,
    }
}

fn compare_lt(first: &Number, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(second) => match first {
            Number::Integer(first) => {
                if second.is_i64() {
                    first < &second.as_i64().unwrap()
                } else {
                    first < &(second.as_f64().unwrap().ceil() as i64)
                }
            }
            Number::Float(first) => {
                if second.is_i64() {
                    (first.floor() as i64) < second.as_i64().unwrap()
                } else {
                    first < &second.as_f64().unwrap()
                }
            }
            Number::BigInteger(_) => todo!(),
        },
        _ => false,
    }
}

fn compare_lteq(first: &Number, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(second) => match first {
            Number::Integer(first) => {
                if second.is_i64() {
                    first <= &second.as_i64().unwrap()
                } else {
                    first <= &(second.as_f64().unwrap().ceil() as i64)
                }
            }
            Number::Float(first) => {
                if second.is_i64() {
                    (first.floor() as i64) <= second.as_i64().unwrap()
                } else {
                    first <= &second.as_f64().unwrap()
                }
            }
            Number::BigInteger(_) => todo!(),
        },
        _ => false,
    }
}

fn compare_start_with(first: &String, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::String(second) => second.starts_with(first),
        _ => false,
    }
}

fn compare_end_with(first: &String, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::String(second) => second.ends_with(first),
        _ => false,
    }
}

fn compare_contain(first: &String, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::String(second) => second.contains(first),
        _ => false,
    }
}

fn compare_include(first: &serde_json::Value, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Array(second) => second.contains(first),
        serde_json::Value::Object(second) => second.contains_key(first.as_str().unwrap()),
        _ => false,
    }
}

fn compare_match(first: &Regex, second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::String(second) => first.is_match(second),
        _ => false,
    }
}

fn compare_is_integer(second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(num) => num.is_i64(),
        _ => false,
    }
}

fn compare_is_float(second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Number(num) => num.is_f64(),
        _ => false,
    }
}

fn compare_is_boolean(second: &serde_json::Value) -> bool {
    matches!(second, serde_json::Value::Bool(_))
}

fn compare_is_string(second: &serde_json::Value) -> bool {
    matches!(second, serde_json::Value::String(_))
}

fn compare_is_collection(second: &serde_json::Value) -> bool {
    matches!(
        second,
        serde_json::Value::Array(_) | serde_json::Value::Object(_)
    )
}

fn compare_exist(second: &serde_json::Value) -> bool {
    !second.is_null()
}

fn compare_is_empty(second: &serde_json::Value) -> bool {
    match second {
        serde_json::Value::Array(arr) => arr.is_empty(),
        serde_json::Value::Object(obj) => obj.is_empty(),
        _ => false,
    }
}

fn try_into_serde_value(
    value: AstPredicateValue,
) -> Result<serde_json::Value, AssertCompilationError> {
    Ok(match value {
        AstPredicateValue::String(value) => {
            serde_json::Value::String(possibly_trim_surrounding_quotes(value.encoded()))
        }
        AstPredicateValue::Number(value) => match value {
            impostor_core::ast::Number::Float(f) => {
                serde_json::to_value(f.value).expect("cannot fail")
            }
            impostor_core::ast::Number::Integer(i) => serde_json::to_value(i).expect("cannot fail"),
            impostor_core::ast::Number::BigInteger(_) => {
                todo!("support compiling big integer assert")
            }
        },
        AstPredicateValue::Bool(value) => serde_json::Value::Bool(value),
        AstPredicateValue::Null => serde_json::Value::Null,
        _ => {
            return Err(AssertCompilationError::InvalidPredicateValue(
                "unsupported type".into(),
            ))
        }
    })
}

fn try_into_number(value: AstPredicateValue) -> Result<Number, AssertCompilationError> {
    Ok(match value {
        AstPredicateValue::Number(value) => match value {
            impostor_core::ast::Number::Float(f) => Number::Float(f.value),
            impostor_core::ast::Number::Integer(i) => Number::Integer(i),
            impostor_core::ast::Number::BigInteger(i) => Number::BigInteger(i),
        },
        _ => {
            return Err(AssertCompilationError::InvalidPredicateValue(
                "expected number".into(),
            ))
        }
    })
}

fn try_into_string(value: AstPredicateValue) -> Result<String, AssertCompilationError> {
    Ok(match value {
        AstPredicateValue::String(value) => value.encoded(),
        _ => {
            return Err(AssertCompilationError::InvalidPredicateValue(
                "expected string".into(),
            ))
        }
    })
}

fn try_into_regex(value: AstPredicateValue) -> Result<Regex, AssertCompilationError> {
    Ok(match value {
        AstPredicateValue::Regex(value) => value.inner,
        _ => {
            return Err(AssertCompilationError::InvalidPredicateValue(
                "expected regex".into(),
            ))
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_apply_equal() {
        let predicate = Predicate::Equal(json!("test"));
        let against = &json!("test");

        assert!(predicate.apply(against));

        let predicate = Predicate::Equal(json!(42));
        assert!(!predicate.apply(against));

        let predicate = Predicate::Equal(json!(true));
        assert!(!predicate.apply(against));

        let predicate = Predicate::Equal(json!(null));
        assert!(!predicate.apply(against));

        let predicate = Predicate::Equal(json!("not_test"));
        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_not_equal() {
        let predicate = Predicate::NotEqual(json!("test"));
        let against = &json!("other");

        assert!(predicate.apply(against));

        let predicate = Predicate::NotEqual(json!(42));
        assert!(predicate.apply(against));

        let predicate = Predicate::NotEqual(json!(true));
        assert!(predicate.apply(against));

        let predicate = Predicate::NotEqual(json!(null));
        assert!(predicate.apply(against));

        let predicate = Predicate::NotEqual(json!("not_test"));
        assert!(predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than() {
        let predicate = Predicate::GreaterThan(Number::Integer(10));
        let against = &json!(15);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThan(Number::Integer(10));
        let against = &json!(5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than_or_equal() {
        let predicate = Predicate::GreaterThanOrEqual(Number::Integer(10));
        let against = &json!(10);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThanOrEqual(Number::Integer(10));
        let against = &json!(5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than() {
        let predicate = Predicate::LessThan(Number::Integer(10));
        let against = &json!(5);

        assert!(predicate.apply(against));

        let predicate = Predicate::LessThan(Number::Integer(10));
        let against = &json!(15);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than_or_equal() {
        let predicate = Predicate::LessThanOrEqual(Number::Integer(10));
        let against = &json!(10);

        assert!(predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than_float_vs_integer() {
        let predicate = Predicate::GreaterThan(Number::Float(10.5));
        let against = &json!(15);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThan(Number::Float(10.5));
        let against = &json!(5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than_or_equal_float_vs_integer() {
        let predicate = Predicate::GreaterThanOrEqual(Number::Float(10.5));
        let against = &json!(10);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThanOrEqual(Number::Float(10.5));
        let against = &json!(5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than_float_vs_integer() {
        let predicate = Predicate::LessThan(Number::Float(10.5));
        let against = &json!(5);

        assert!(predicate.apply(against));

        let predicate = Predicate::LessThan(Number::Float(10.5));
        let against = &json!(15);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than_or_equal_float_vs_integer() {
        let predicate = Predicate::LessThanOrEqual(Number::Float(10.5));
        let against = &json!(10);

        assert!(predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than_integer_vs_float() {
        let predicate = Predicate::GreaterThan(Number::Integer(10));
        let against = &json!(10.5);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThan(Number::Integer(10));
        let against = &json!(5.5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_greater_than_or_equal_integer_vs_float() {
        let predicate = Predicate::GreaterThanOrEqual(Number::Integer(10));
        let against = &json!(10.5);

        assert!(predicate.apply(against));

        let predicate = Predicate::GreaterThanOrEqual(Number::Integer(10));
        let against = &json!(5.5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than_integer_vs_float() {
        let predicate = Predicate::LessThan(Number::Integer(10));
        let against = &json!(5.5);

        assert!(predicate.apply(against));

        let predicate = Predicate::LessThan(Number::Integer(10));
        let against = &json!(15.5);

        assert!(!predicate.apply(against));
    }

    #[test]
    fn test_apply_less_than_or_equal_integer_vs_float() {
        let predicate = Predicate::LessThanOrEqual(Number::Integer(10));
        let against = &json!(10.5);

        assert!(predicate.apply(against));
    }

    #[test]
    fn test_apply_start_with() {
        let predicate = Predicate::StartWith("test".to_string());
        let against = &json!("test value");

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!("value test")));
    }

    #[test]
    fn test_apply_end_with() {
        let predicate = Predicate::EndWith("test".to_string());
        let against = &json!("value test");

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!("test value")));
    }

    #[test]
    fn test_apply_contain() {
        let predicate = Predicate::Contain("test".to_string());
        let against = &json!("value with test");

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!("value without")));
    }

    #[test]
    fn test_apply_include() {
        let predicate = Predicate::Include(json!("test"));
        let against = &json!(["test", "other"]);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(["other"])));
    }

    #[test]
    fn test_apply_match() {
        let predicate = Predicate::Match(Regex::new("^test$").unwrap());
        let against = &json!("test");

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!("other")));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_apply_is_integer() {
        let predicate = Predicate::IsInteger;
        let against = &json!(42);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(3.14)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_apply_is_float() {
        let predicate = Predicate::IsFloat;
        let against = &json!(3.14);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(42)));
    }

    #[test]
    fn test_apply_is_boolean() {
        let predicate = Predicate::IsBoolean;
        let against = &json!(true);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(42)));
    }

    #[test]
    fn test_apply_is_string() {
        let predicate = Predicate::IsString;
        let against = &json!("test");

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(42)));
    }

    #[test]
    #[deny(clippy::approx_constant)]
    fn test_apply_is_collection() {
        let predicate = Predicate::IsCollection;
        let against = &json!(["test", "other"]);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(42)));
    }

    #[test]
    fn test_apply_exist() {
        let predicate = Predicate::Exist;
        let against = &json!(42);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(null)));
    }

    #[test]
    fn test_apply_is_empty() {
        let predicate = Predicate::IsEmpty;
        let against = &json!([]);

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!([1, 2, 3])));

        assert!(predicate.apply(against));
        assert!(!predicate.apply(&json!(null)));
    }
}
