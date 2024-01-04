use std::collections::HashMap;

use axum::{
    extract::Request,
    http::{HeaderMap, Method},
    routing::{MethodFilter, MethodRouter},
};

use crate::entry::Entry;

mod asserts;
mod entry;
mod error;
pub(crate) mod template;

pub fn compile(contents: &str) -> error::Result<axum::Router> {
    let parse_result = impostor_core::parser::parse_impostor_file(contents);
    let ast = match parse_result {
        Ok(ast) => ast,
        Err(e) => return Err(error::Error::ParseError(e)),
    };

    let entries: Vec<Entry> = ast
        .entries
        .into_iter()
        .map(|entry| {
            entry
                .try_into()
                .map_err(error::Error::EntryCompilationError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut routes_methods_entries: HashMap<(String, Method), Vec<Entry>> = HashMap::new();

    for entry in entries {
        let route = entry.path.clone();
        let method = entry.method.clone();
        let entries_for_this_route = routes_methods_entries
            .entry((route, method))
            .or_insert(Vec::new());
        entries_for_this_route.push(entry);
    }

    let mut router = axum::Router::new();

    for ((route, method), entries) in routes_methods_entries {
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
