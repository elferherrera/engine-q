use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::ast::CellPath;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone)]
pub struct SubCommand;

struct Arguments {
    range: Value,
    column_paths: Vec<CellPath>,
}

#[derive(Clone)]
struct Substring(isize, isize);

impl From<(isize, isize)> for Substring {
    fn from(input: (isize, isize)) -> Substring {
        Substring(input.0, input.1)
    }
}

struct SubstringText(String, String);

impl Command for SubCommand {
    fn name(&self) -> &str {
        "str substring"
    }

    fn signature(&self) -> Signature {
        Signature::build("str substring")
            .required(
                "range",
                SyntaxShape::Any,
                "the indexes to substring [start end]",
            )
            .rest(
                "rest",
                SyntaxShape::CellPath,
                "optionally substring text by column paths",
            )
    }

    fn usage(&self) -> &str {
        "substrings text"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        operate(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Get a substring from the text",
                example: " 'good nushell' | str substring [5 12]",
                result: Some(Value::test_string("nushell")),
            },
            Example {
                description: "Alternatively, you can use the form",
                example: " 'good nushell' | str substring '5,12'",
                result: Some(Value::test_string("nushell")),
            },
            Example {
                description: "Drop the last `n` characters from the string",
                example: " 'good nushell' | str substring ',-5'",
                result: Some(Value::test_string("good nu")),
            },
            Example {
                description: "Get the remaining characters from a starting index",
                example: " 'good nushell' | str substring '5,'",
                result: Some(Value::test_string("nushell")),
            },
            Example {
                description: "Get the characters from the beginning until ending index",
                example: " 'good nushell' | str substring ',7'",
                result: Some(Value::test_string("good nu")),
            },
        ]
    }
}

fn operate(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let options = Arc::new(Arguments {
        range: call.req(engine_state, stack, 0)?,
        column_paths: call.rest(engine_state, stack, 1)?,
    });

    let head = call.head;
    let indexes: Arc<Substring> = Arc::new(process_arguments(&options, head)?.into());

    input.map(
        move |v| {
            if options.column_paths.is_empty() {
                action(&v, &indexes, head)
            } else {
                let mut ret = v;
                for path in &options.column_paths {
                    let indexes = indexes.clone();
                    let r = ret.update_cell_path(
                        &path.members,
                        Box::new(move |old| action(old, &indexes, head)),
                    );
                    if let Err(error) = r {
                        return Value::Error { error };
                    }
                }
                ret
            }
        },
        engine_state.ctrlc.clone(),
    )
}

fn action(input: &Value, options: &Substring, head: Span) -> Value {
    match input {
        Value::String { val: s, .. } => {
            let len: isize = s.len() as isize;

            let start: isize = if options.0 < 0 {
                options.0 + len
            } else {
                options.0
            };
            let end: isize = if options.1 < 0 {
                std::cmp::max(len + options.1, 0)
            } else {
                options.1
            };

            if start < len && end >= 0 {
                match start.cmp(&end) {
                    Ordering::Equal => Value::String {
                        val: "".to_string(),
                        span: head,
                    },
                    Ordering::Greater => Value::Error {
                        error: ShellError::UnsupportedInput(
                            "End must be greater than or equal to Start".to_string(),
                            head,
                        ),
                    },
                    Ordering::Less => Value::String {
                        val: {
                            if end == isize::max_value() {
                                s.chars().skip(start as usize).collect::<String>()
                            } else {
                                s.chars()
                                    .skip(start as usize)
                                    .take((end - start) as usize)
                                    .collect::<String>()
                            }
                        },
                        span: head,
                    },
                }
            } else {
                Value::String {
                    val: "".to_string(),
                    span: head,
                }
            }
        }
        other => Value::Error {
            error: ShellError::UnsupportedInput(
                format!(
                    "Input's type is {}. This command only works with strings.",
                    other.get_type()
                ),
                head,
            ),
        },
    }
}

fn process_arguments(options: &Arguments, head: Span) -> Result<(isize, isize), ShellError> {
    let search = match &options.range {
        Value::List { vals, .. } => {
            if vals.len() > 2 {
                Err(ShellError::UnsupportedInput(
                    "More than two indices given".to_string(),
                    head,
                ))
            } else {
                let idx: Vec<String> = vals
                    .iter()
                    .map(|v| {
                        match v {
                            Value::Int { val, .. } => Ok(val.to_string()),
                            Value::String { val, .. } => Ok(val.to_string()),
                            _ => Err(ShellError::UnsupportedInput(
                                "could not perform substring. Expecting a string or int"
                                    .to_string(),
                                head,
                            )),
                        }
                        .unwrap_or_else(|_| String::from(""))
                    })
                    .collect();

                let start = idx
                    .get(0)
                    .ok_or_else(|| {
                        ShellError::UnsupportedInput(
                            "could not perform substring".to_string(),
                            head,
                        )
                    })?
                    .to_string();
                let end = idx
                    .get(1)
                    .ok_or_else(|| {
                        ShellError::UnsupportedInput(
                            "could not perform substring".to_string(),
                            head,
                        )
                    })?
                    .to_string();
                Ok(SubstringText(start, end))
            }
        }
        Value::String { val, .. } => {
            let idx: Vec<&str> = val.split(',').collect();

            let start = idx
                .get(0)
                .ok_or_else(|| {
                    ShellError::UnsupportedInput("could not perform substring".to_string(), head)
                })?
                .to_string();
            let end = idx
                .get(1)
                .ok_or_else(|| {
                    ShellError::UnsupportedInput("could not perform substring".to_string(), head)
                })?
                .to_string();

            Ok(SubstringText(start, end))
        }
        _ => Err(ShellError::UnsupportedInput(
            "could not perform substring".to_string(),
            head,
        )),
    }?;
    let start = match &search {
        SubstringText(start, _) if start.is_empty() || start == "_" => 0,
        SubstringText(start, _) => start.trim().parse().map_err(|_| {
            ShellError::UnsupportedInput("could not perform substring".to_string(), head)
        })?,
    };

    let end = match &search {
        SubstringText(_, end) if end.is_empty() || end == "_" => isize::max_value(),
        SubstringText(_, end) => end.trim().parse().map_err(|_| {
            ShellError::UnsupportedInput("could not perform substring".to_string(), head)
        })?,
    };

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::{action, Span, SubCommand, Substring, Value};

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
    struct Expectation<'a> {
        options: (isize, isize),
        expected: &'a str,
    }

    impl Expectation<'_> {
        fn options(&self) -> Substring {
            Substring(self.options.0, self.options.1)
        }
    }

    fn expectation(word: &str, indexes: (isize, isize)) -> Expectation {
        Expectation {
            options: indexes,
            expected: word,
        }
    }

    #[test]
    fn substrings_indexes() {
        let word = Value::String {
            val: "andres".to_string(),
            span: Span::unknown(),
        };

        let cases = vec![
            expectation("a", (0, 1)),
            expectation("an", (0, 2)),
            expectation("and", (0, 3)),
            expectation("andr", (0, 4)),
            expectation("andre", (0, 5)),
            expectation("andres", (0, 6)),
            expectation("", (0, -6)),
            expectation("a", (0, -5)),
            expectation("an", (0, -4)),
            expectation("and", (0, -3)),
            expectation("andr", (0, -2)),
            expectation("andre", (0, -1)),
            // str substring [ -4 , _ ]
            // str substring   -4 ,
            expectation("dres", (-4, isize::max_value())),
            expectation("", (0, -110)),
            expectation("", (6, 0)),
            expectation("", (6, -1)),
            expectation("", (6, -2)),
            expectation("", (6, -3)),
            expectation("", (6, -4)),
            expectation("", (6, -5)),
            expectation("", (6, -6)),
        ];

        for expectation in &cases {
            let expected = expectation.expected;
            let actual = action(&word, &expectation.options(), Span::unknown());

            assert_eq!(
                actual,
                Value::String {
                    val: expected.to_string(),
                    span: Span::unknown()
                }
            );
        }
    }
}
