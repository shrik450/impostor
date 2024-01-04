use std::collections::HashMap;

use impostor_core::ast::Template;

pub enum TemplateExecutionError {
    UnknownVariable(String),
}

impl std::fmt::Display for TemplateExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TemplateExecutionError::UnknownVariable(name) => {
                write!(f, "unknown variable: {}", name)
            }
        }
    }
}

/// Perf optimization to avoid having to execute a template if it's just a
/// static string.
#[derive(Clone)]
pub(crate) enum StringOrTemplate {
    String(String),
    Template(Template),
}

impl StringOrTemplate {
    pub fn execute(&self) -> Result<String, TemplateExecutionError> {
        match self {
            StringOrTemplate::String(s) => Ok(s.clone()),
            StringOrTemplate::Template(t) => execute(t, HashMap::new()),
        }
    }

    /// Converts an AST template into a `StringOrTemplate`.
    pub(crate) fn from_ast_template(template: Template) -> StringOrTemplate {
        if template
            .elements
            .iter()
            .any(|element| matches!(element, impostor_core::ast::TemplateElement::Expression(_)))
        {
            StringOrTemplate::Template(template)
        } else {
            StringOrTemplate::String(template.encoded())
        }
    }
}

pub fn execute(
    template: &Template,
    context: HashMap<String, String>,
) -> Result<String, TemplateExecutionError> {
    let mut result = String::new();

    for element in template.elements.iter() {
        match element {
            impostor_core::ast::TemplateElement::String { encoded, .. } => result.push_str(encoded),
            impostor_core::ast::TemplateElement::Expression(expression) => {
                if let Some(value) = context.get(expression.variable.name.as_str()) {
                    result.push_str(value);
                } else {
                    return Err(TemplateExecutionError::UnknownVariable(
                        expression.variable.name.clone(),
                    ));
                }
            }
        }
    }

    Ok(result)
}
