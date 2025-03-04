use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, SyntaxShape,
    Value,
};

#[derive(Clone)]
pub struct StrCollect;

impl Command for StrCollect {
    fn name(&self) -> &str {
        "str collect"
    }

    fn signature(&self) -> Signature {
        Signature::build("str collect")
            .optional(
                "separator",
                SyntaxShape::String,
                "optional separator to use when creating string",
            )
            .category(Category::Strings)
    }

    fn usage(&self) -> &str {
        "creates a string from the input, optionally using a separator"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let separator: Option<String> = call.opt(engine_state, stack, 0)?;

        let config = stack.get_config().unwrap_or_default();

        // Hmm, not sure what we actually want. If you don't use debug_string, Date comes out as human readable
        // which feels funny
        #[allow(clippy::needless_collect)]
        let strings: Vec<String> = input
            .into_iter()
            .map(|value| value.debug_string("\n", &config))
            .collect();

        let output = if let Some(separator) = separator {
            strings.join(&separator)
        } else {
            strings.join("")
        };

        Ok(Value::String {
            val: output,
            span: call.head,
        }
        .into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Create a string from input",
                example: "['nu', 'shell'] | str collect",
                result: Some(Value::String {
                    val: "nushell".to_string(),
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "Create a string from input with a separator",
                example: "['nu', 'shell'] | str collect '-'",
                result: Some(Value::String {
                    val: "nu-shell".to_string(),
                    span: Span::unknown(),
                }),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(StrCollect {})
    }
}
