use inflector::cases::screamingsnakecase::to_screaming_snake_case;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value,
};

use crate::operate;
#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "str screaming-snake-case"
    }

    fn signature(&self) -> Signature {
        Signature::build("str screaming-snake-case")
            .rest(
                "rest",
                SyntaxShape::CellPath,
                "optionally convert text to SCREAMING_SNAKE_CASE by column paths",
            )
            .category(Category::Strings)
    }

    fn usage(&self) -> &str {
        "converts a string to SCREAMING_SNAKE_CASE"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        operate(engine_state, stack, call, input, &to_screaming_snake_case)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "convert a string to camelCase",
                example: r#" "NuShell" | str screaming-snake-case"#,
                result: Some(Value::String {
                    val: "NU_SHELL".to_string(),
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "convert a string to camelCase",
                example: r#" "this_is_the_second_case" | str screaming-snake-case"#,
                result: Some(Value::String {
                    val: "THIS_IS_THE_SECOND_CASE".to_string(),
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "convert a string to camelCase",
                example: r#""this-is-the-first-case" | str screaming-snake-case"#,
                result: Some(Value::String {
                    val: "THIS_IS_THE_FIRST_CASE".to_string(),
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "convert a column from a table to SCREAMING_SNAKE_CASE",
                example: r#"[[lang, gems]; [nu_test, 100]] | str screaming-snake-case lang"#,
                result: Some(Value::List {
                    vals: vec![Value::Record {
                        span: Span::unknown(),
                        cols: vec!["lang".to_string(), "gems".to_string()],
                        vals: vec![
                            Value::String {
                                val: "NU_TEST".to_string(),
                                span: Span::unknown(),
                            },
                            Value::test_int(100),
                        ],
                    }],
                    span: Span::unknown(),
                }),
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
