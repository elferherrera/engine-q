use indexmap::map::IndexMap;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Config, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Value,
};

#[derive(Clone)]
pub struct FromIni;

impl Command for FromIni {
    fn name(&self) -> &str {
        "from ini"
    }

    fn signature(&self) -> Signature {
        Signature::build("from ini").category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Parse text as .ini and create table"
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "'[foo]
a=1
b=2' | from ini",
            description: "Converts ini formatted string to table",
            result: Some(Value::Record {
                cols: vec!["foo".to_string()],
                vals: vec![Value::Record {
                    cols: vec!["a".to_string(), "b".to_string()],
                    vals: vec![
                        Value::String {
                            val: "1".to_string(),
                            span: Span::unknown(),
                        },
                        Value::String {
                            val: "2".to_string(),
                            span: Span::unknown(),
                        },
                    ],
                    span: Span::unknown(),
                }],
                span: Span::unknown(),
            }),
        }]
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
        from_ini(input, head, &config)
    }
}

pub fn from_ini_string_to_value(s: String, span: Span) -> Result<Value, ShellError> {
    let v: Result<IndexMap<String, IndexMap<String, String>>, serde_ini::de::Error> =
        serde_ini::from_str(&s);
    match v {
        Ok(index_map) => {
            let (cols, vals) = index_map
                .into_iter()
                .fold((vec![], vec![]), |mut acc, (k, v)| {
                    let (cols, vals) = v.into_iter().fold((vec![], vec![]), |mut acc, (k, v)| {
                        acc.0.push(k);
                        acc.1.push(Value::String { val: v, span });
                        acc
                    });
                    acc.0.push(k);
                    acc.1.push(Value::Record { cols, vals, span });
                    acc
                });
            Ok(Value::Record { cols, vals, span })
        }
        Err(err) => Err(ShellError::UnsupportedInput(
            format!("Could not load ini: {}", err),
            span,
        )),
    }
}

fn from_ini(input: PipelineData, head: Span, config: &Config) -> Result<PipelineData, ShellError> {
    let concat_string = input.collect_string("", config);

    match from_ini_string_to_value(concat_string, head) {
        Ok(x) => Ok(x.into_pipeline_data()),
        Err(other) => Err(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(FromIni {})
    }
}
