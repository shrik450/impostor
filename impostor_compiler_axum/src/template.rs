use std::collections::HashMap;

use impostor_core::ast::Template;

use crate::possibly_trim_surrounding_quotes;

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
#[derive(Clone, Debug)]
pub(crate) enum StringOrTemplate {
    String(String),
    Template(Template),
}

impl StringOrTemplate {
    /// Executes the template, substituting variables with values from the
    /// context. If the template is just a static string, returns a copy of that
    /// string. This means that this method will always allocate.
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
            StringOrTemplate::String(possibly_trim_surrounding_quotes(template.encoded()))
        }
    }
}

impl From<Template> for StringOrTemplate {
    fn from(template: Template) -> Self {
        Self::from_ast_template(template)
    }
}

fn execute(
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
