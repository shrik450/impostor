mod asserts;
mod entry;
mod error;
pub(crate) mod template;

pub fn compile(contents: &str) -> error::Result<axum::Router> {
    let parse_result = inko_core::parser::parse_inko_file(contents);
    let ast = match parse_result {
        Ok(ast) => ast,
        Err(e) => return Err(error::Error::ParseError(e)),
    };

    todo!("compile");
}

#[cfg(test)]
mod tests {
    use super::*;
}
