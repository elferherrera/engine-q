use nu_engine::CallExt;
use nu_protocol::{
    ast::Call, ast::CellPath, engine::Command, engine::EngineState, engine::Stack, Category,
    Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value,
};
use strip_ansi_escapes::strip;

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "ansi strip"
    }

    fn signature(&self) -> Signature {
        Signature::build("ansi strip")
            .rest(
                "column path",
                SyntaxShape::CellPath,
                "optionally, remove ansi sequences by column paths",
            )
            .category(Category::Platform)
    }

    fn usage(&self) -> &str {
        "strip ansi escape sequences from string"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, ShellError> {
        operate(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "strip ansi escape sequences from string",
            example: r#"echo [ (ansi green) (ansi cursor_on) "hello" ] | str collect | ansi strip"#,
            result: Some(Value::test_string("hello")),
        }]
    }
}

fn operate(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let column_paths: Vec<CellPath> = call.rest(engine_state, stack, 0)?;
    let head = call.head;
    input.map(
        move |v| {
            if column_paths.is_empty() {
                action(&v, &head)
            } else {
                let mut ret = v;

                for path in &column_paths {
                    let r = ret
                        .update_cell_path(&path.members, Box::new(move |old| action(old, &head)));
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

fn action(input: &Value, command_span: &Span) -> Value {
    match input {
        Value::String { val, span } => {
            let stripped_string = {
                if let Ok(bytes) = strip(&val) {
                    String::from_utf8_lossy(&bytes).to_string()
                } else {
                    val.to_string()
                }
            };

            Value::string(stripped_string, *span)
        }
        other => {
            let got = format!("value is {}, not string", other.get_type().to_string());

            Value::Error {
                error: ShellError::TypeMismatch(got, other.span().unwrap_or(*command_span)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{action, SubCommand};
    use nu_protocol::{Span, Value};

    #[test]
    fn examples_work_as_expected() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }

    #[test]
    fn test_stripping() {
        let input_string =
            Value::test_string("\u{1b}[3;93;41mHello\u{1b}[0m \u{1b}[1;32mNu \u{1b}[1;35mWorld");
        let expected = Value::test_string("Hello Nu World");

        let actual = action(&input_string, &Span::unknown());
        assert_eq!(actual, expected);
    }
}
