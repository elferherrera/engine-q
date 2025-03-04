use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Config, Example, PipelineData, ShellError, Signature, Span, Value};

#[derive(Clone)]
pub struct FromUrl;

impl Command for FromUrl {
    fn name(&self) -> &str {
        "from url"
    }

    fn signature(&self) -> Signature {
        Signature::build("from url").category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Parse url-encoded string as a table."
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, ShellError> {
        let head = call.head;
        let config = stack.get_config().unwrap_or_default();
        from_url(input, head, &config)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "'bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter' | from url",
            description: "Convert url encoded string into a table",
            result: Some(Value::Record {
                cols: vec![
                    "bread".to_string(),
                    "cheese".to_string(),
                    "meat".to_string(),
                    "fat".to_string(),
                ],
                vals: vec![
                    Value::test_string("baguette"),
                    Value::test_string("comté"),
                    Value::test_string("ham"),
                    Value::test_string("butter"),
                ],
                span: Span::unknown(),
            }),
        }]
    }
}

fn from_url(input: PipelineData, head: Span, config: &Config) -> Result<PipelineData, ShellError> {
    let concat_string = input.collect_string("", config);

    let result = serde_urlencoded::from_str::<Vec<(String, String)>>(&concat_string);

    match result {
        Ok(result) => {
            let mut cols = vec![];
            let mut vals = vec![];
            for (k, v) in result {
                cols.push(k);
                vals.push(Value::String { val: v, span: head })
            }

            Ok(PipelineData::Value(
                Value::Record {
                    cols,
                    vals,
                    span: head,
                },
                None,
            ))
        }
        _ => Err(ShellError::UnsupportedInput(
            "String not compatible with url-encoding".to_string(),
            head,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(FromUrl {})
    }
}
