/*
 * Copyright (C) 2023 The Impostor Contributors
 * Copyright (C) 2023 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use crate::ast::*;
use crate::parser::combinators::*;
use crate::parser::error::*;
use crate::parser::filter::filters;
use crate::parser::predicate::predicate;
use crate::parser::primitives::*;
use crate::parser::query::query;
use crate::parser::reader::Reader;
use crate::parser::{key_string, ParseResult};

pub fn request_sections(reader: &mut Reader) -> ParseResult<Vec<RequestSection>> {
    let sections = zero_or_more(request_section, reader)?;
    Ok(sections)
}

fn request_section(reader: &mut Reader) -> ParseResult<RequestSection> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let start = reader.state.pos;
    let name = section_name(reader)?;
    let source_info = SourceInfo {
        start,
        end: reader.state.pos,
    };
    let line_terminator0 = line_terminator(reader)?;
    let value = match name.as_str() {
        "Captures" => section_value_captures(reader)?,
        "Asserts" => section_value_asserts(reader)?,
        _ => {
            let inner = ParseError::ResponseSectionName { name: name.clone() };
            let pos = Pos::new(start.line, start.column + 1);
            return Err(Error::new(pos, false, inner));
        }
    };

    Ok(RequestSection {
        line_terminators,
        space0,
        line_terminator0,
        value,
        source_info,
    })
}

fn section_name(reader: &mut Reader) -> ParseResult<String> {
    let pos = reader.state.pos;
    try_literal("[", reader)?;
    let name = reader.read_while(|c| c.is_alphanumeric());
    if name.is_empty() {
        // Could be the empty json array for the body
        let inner = ParseError::Expecting {
            value: "a valid section name".to_string(),
        };
        return Err(Error::new(pos, true, inner));
    }
    try_literal("]", reader)?;
    Ok(name)
}

fn section_value_captures(reader: &mut Reader) -> ParseResult<RequestSectionValue> {
    let items = zero_or_more(capture, reader)?;
    Ok(RequestSectionValue::Captures(items))
}

fn section_value_asserts(reader: &mut Reader) -> ParseResult<RequestSectionValue> {
    let asserts = zero_or_more(assert, reader)?;
    Ok(RequestSectionValue::Asserts(asserts))
}

fn capture(reader: &mut Reader) -> ParseResult<Capture> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let name = recover(key_string::parse, reader)?;
    let space1 = zero_or_more_spaces(reader)?;
    recover(|p1| literal(":", p1), reader)?;
    let space2 = zero_or_more_spaces(reader)?;
    let q = query(reader)?;
    let filters = filters(reader)?;
    let line_terminator0 = line_terminator(reader)?;
    Ok(Capture {
        line_terminators,
        space0,
        name,
        space1,
        space2,
        query: q,
        filters,
        line_terminator0,
    })
}

fn assert(reader: &mut Reader) -> ParseResult<Assert> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let query0 = query(reader)?;
    let filters = filters(reader)?;
    let space1 = one_or_more_spaces(reader)?;
    let predicate0 = predicate(reader)?;

    let line_terminator0 = line_terminator(reader)?;
    Ok(Assert {
        line_terminators,
        space0,
        query: query0,
        filters,
        space1,
        predicate: predicate0,
        line_terminator0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Pos;

    #[test]
    fn test_section_name() {
        let mut reader = Reader::new("[SectionA]");
        assert_eq!(section_name(&mut reader).unwrap(), String::from("SectionA"));

        let mut reader = Reader::new("[]");
        assert!(section_name(&mut reader).err().unwrap().recoverable);
    }

    #[test]
    fn test_asserts_section() {
        let mut reader = Reader::new("[Asserts]\nheader \"Location\" == \"https://google.fr\"\n");

        assert_eq!(
            request_section(&mut reader).unwrap(),
            RequestSection {
                line_terminators: vec![],
                space0: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                },
                line_terminator0: LineTerminator {
                    space0: Whitespace {
                        value: String::new(),
                        source_info: SourceInfo::new(Pos::new(1, 10), Pos::new(1, 10)),
                    },
                    comment: None,
                    newline: Whitespace {
                        value: String::from("\n"),
                        source_info: SourceInfo::new(Pos::new(1, 10), Pos::new(2, 1)),
                    },
                },
                value: RequestSectionValue::Asserts(vec![Assert {
                    line_terminators: vec![],
                    space0: Whitespace {
                        value: String::new(),
                        source_info: SourceInfo::new(Pos::new(2, 1), Pos::new(2, 1)),
                    },
                    query: Query {
                        source_info: SourceInfo::new(Pos::new(2, 1), Pos::new(2, 18)),
                        value: QueryValue::Header {
                            space0: Whitespace {
                                value: String::from(" "),
                                source_info: SourceInfo::new(Pos::new(2, 7), Pos::new(2, 8)),
                            },
                            name: Template {
                                delimiter: Some('"'),
                                elements: vec![TemplateElement::String {
                                    value: "Location".to_string(),
                                    encoded: "Location".to_string(),
                                }],
                                source_info: SourceInfo::new(Pos::new(2, 8), Pos::new(2, 18)),
                            },
                        },
                    },
                    filters: vec![],
                    space1: Whitespace {
                        value: String::from(" "),
                        source_info: SourceInfo::new(Pos::new(2, 18), Pos::new(2, 19)),
                    },
                    predicate: Predicate {
                        not: false,
                        space0: Whitespace {
                            value: String::new(),
                            source_info: SourceInfo::new(Pos::new(2, 19), Pos::new(2, 19)),
                        },
                        predicate_func: PredicateFunc {
                            source_info: SourceInfo::new(Pos::new(2, 19), Pos::new(2, 41)),
                            value: PredicateFuncValue::Equal {
                                space0: Whitespace {
                                    value: String::from(" "),
                                    source_info: SourceInfo::new(Pos::new(2, 21), Pos::new(2, 22)),
                                },
                                value: PredicateValue::String(Template {
                                    delimiter: Some('"'),
                                    elements: vec![TemplateElement::String {
                                        value: "https://google.fr".to_string(),
                                        encoded: "https://google.fr".to_string(),
                                    }],
                                    source_info: SourceInfo::new(Pos::new(2, 22), Pos::new(2, 41)),
                                }),
                                operator: true,
                            },
                        },
                    },
                    line_terminator0: LineTerminator {
                        space0: Whitespace {
                            value: String::new(),
                            source_info: SourceInfo::new(Pos::new(2, 41), Pos::new(2, 41)),
                        },
                        comment: None,
                        newline: Whitespace {
                            value: String::from("\n"),
                            source_info: SourceInfo::new(Pos::new(2, 41), Pos::new(3, 1)),
                        },
                    },
                }]),
                source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 10)),
            }
        );
    }

    #[test]
    fn test_asserts_section_error() {
        let mut reader = Reader::new("x[Assertsx]\nheader Location == \"https://google.fr\"\n");
        let error = request_section(&mut reader).err().unwrap();
        assert_eq!(error.pos, Pos { line: 1, column: 1 });
        assert_eq!(
            error.inner,
            ParseError::Expecting {
                value: String::from("[")
            }
        );
        assert!(error.recoverable);

        let mut reader = Reader::new("[Assertsx]\nheader Location == \"https://google.fr\"\n");
        let error = request_section(&mut reader).err().unwrap();
        assert_eq!(error.pos, Pos { line: 1, column: 2 });
        assert_eq!(
            error.inner,
            ParseError::ResponseSectionName {
                name: String::from("Assertsx")
            }
        );
        assert!(!error.recoverable);
    }

    #[test]
    fn test_capture() {
        let mut reader = Reader::new("url: header \"Location\"");
        let capture0 = capture(&mut reader).unwrap();

        assert_eq!(
            capture0.name,
            Template {
                delimiter: None,
                elements: vec![TemplateElement::String {
                    value: "url".to_string(),
                    encoded: "url".to_string(),
                }],
                source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 4)),
            },
        );
        assert_eq!(
            capture0.query,
            Query {
                source_info: SourceInfo::new(Pos::new(1, 6), Pos::new(1, 23)),
                value: QueryValue::Header {
                    space0: Whitespace {
                        value: String::from(" "),
                        source_info: SourceInfo::new(Pos::new(1, 12), Pos::new(1, 13)),
                    },
                    name: Template {
                        delimiter: Some('"'),
                        elements: vec![TemplateElement::String {
                            value: "Location".to_string(),
                            encoded: "Location".to_string(),
                        }],
                        source_info: SourceInfo::new(Pos::new(1, 13), Pos::new(1, 23)),
                    },
                },
            }
        );
    }

    #[test]
    fn test_capture_with_filter() {
        let mut reader = Reader::new("token: header \"Location\" regex \"token=(.*)\"");
        let capture0 = capture(&mut reader).unwrap();

        assert_eq!(
            capture0.query,
            Query {
                source_info: SourceInfo::new(Pos::new(1, 8), Pos::new(1, 25)),
                value: QueryValue::Header {
                    space0: Whitespace {
                        value: String::from(" "),
                        source_info: SourceInfo::new(Pos::new(1, 14), Pos::new(1, 15)),
                    },
                    name: Template {
                        delimiter: Some('"'),
                        elements: vec![TemplateElement::String {
                            value: "Location".to_string(),
                            encoded: "Location".to_string(),
                        }],
                        source_info: SourceInfo::new(Pos::new(1, 15), Pos::new(1, 25)),
                    },
                },
            }
        );
        assert_eq!(reader.state.cursor, 43);
    }

    #[test]
    fn test_capture_with_filter_error() {
        let mut reader = Reader::new("token: header \"Location\" regex ");
        let error = capture(&mut reader).err().unwrap();
        assert_eq!(
            error.pos,
            Pos {
                line: 1,
                column: 32,
            }
        );
        assert_eq!(
            error.inner,
            ParseError::Expecting {
                value: "\" or /".to_string()
            }
        );
        assert!(!error.recoverable);

        let mut reader = Reader::new("token: header \"Location\" xxx");
        let error = capture(&mut reader).err().unwrap();
        assert_eq!(
            error.pos,
            Pos {
                line: 1,
                column: 26,
            }
        );
        assert_eq!(
            error.inner,
            ParseError::Expecting {
                value: "line_terminator".to_string()
            }
        );
        assert!(!error.recoverable);
    }

    #[test]
    fn test_assert() {
        let mut reader = Reader::new("header \"Location\" == \"https://google.fr\"");
        let assert0 = assert(&mut reader).unwrap();

        assert_eq!(
            assert0.query,
            Query {
                source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 18)),
                value: QueryValue::Header {
                    space0: Whitespace {
                        value: String::from(" "),
                        source_info: SourceInfo::new(Pos::new(1, 7), Pos::new(1, 8)),
                    },
                    name: Template {
                        delimiter: Some('"'),
                        elements: vec![TemplateElement::String {
                            value: "Location".to_string(),
                            encoded: "Location".to_string(),
                        }],
                        source_info: SourceInfo::new(Pos::new(1, 8), Pos::new(1, 18)),
                    },
                },
            }
        );
    }

    #[test]
    fn test_assert_jsonpath() {
        let mut reader = Reader::new("jsonpath \"$.errors\" == 5");

        assert_eq!(
            assert(&mut reader).unwrap().predicate,
            Predicate {
                not: false,
                space0: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(1, 21), Pos::new(1, 21)),
                },
                predicate_func: PredicateFunc {
                    source_info: SourceInfo::new(Pos::new(1, 21), Pos::new(1, 25)),
                    value: PredicateFuncValue::Equal {
                        space0: Whitespace {
                            value: String::from(" "),
                            source_info: SourceInfo::new(Pos::new(1, 23), Pos::new(1, 24)),
                        },
                        value: PredicateValue::Number(Number::Integer(5)),
                        operator: true,
                    },
                },
            }
        );
    }
}
