use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Value,
};

#[derive(Clone)]
pub struct FromToml;

impl Command for FromToml {
    fn name(&self) -> &str {
        "from toml"
    }

    fn signature(&self) -> Signature {
        Signature::build("from toml").category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Parse text as .toml and create table."
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'a = 1' | from toml",
                description: "Converts toml formatted string to table",
                result: Some(Value::Record {
                    cols: vec!["a".to_string()],
                    vals: vec![Value::Int {
                        val: 1,
                        span: Span::unknown(),
                    }],
                    span: Span::unknown(),
                }),
            },
            Example {
                example: "'a = 1
b = [1, 2]' | from toml",
                description: "Converts toml formatted string to table",
                result: Some(Value::Record {
                    cols: vec!["a".to_string(), "b".to_string()],
                    vals: vec![
                        Value::Int {
                            val: 1,
                            span: Span::unknown(),
                        },
                        Value::List {
                            vals: vec![
                                Value::Int {
                                    val: 1,
                                    span: Span::unknown(),
                                },
                                Value::Int {
                                    val: 2,
                                    span: Span::unknown(),
                                },
                            ],
                            span: Span::unknown(),
                        },
                    ],
                    span: Span::unknown(),
                }),
            },
        ]
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, ShellError> {
        let span = call.head;
        let config = stack.get_config().unwrap_or_default();
        let mut string_input = input.collect_string("", &config);
        string_input.push('\n');
        Ok(convert_string_to_value(string_input, span)?.into_pipeline_data())
    }
}

fn convert_toml_to_value(value: &toml::Value, span: Span) -> Value {
    match value {
        toml::Value::Array(array) => {
            let v: Vec<Value> = array
                .iter()
                .map(|x| convert_toml_to_value(x, span))
                .collect();

            Value::List { vals: v, span }
        }
        toml::Value::Boolean(b) => Value::Bool { val: *b, span },
        toml::Value::Float(f) => Value::Float { val: *f, span },
        toml::Value::Integer(i) => Value::Int { val: *i, span },
        toml::Value::Table(k) => {
            let mut cols = vec![];
            let mut vals = vec![];

            for item in k {
                cols.push(item.0.clone());
                vals.push(convert_toml_to_value(item.1, span));
            }

            Value::Record { cols, vals, span }
        }
        toml::Value::String(s) => Value::String {
            val: s.clone(),
            span,
        },
        toml::Value::Datetime(d) => Value::String {
            val: d.to_string(),
            span,
        },
    }
}

pub fn convert_string_to_value(string_input: String, span: Span) -> Result<Value, ShellError> {
    let result: Result<toml::Value, toml::de::Error> = toml::from_str(&string_input);
    match result {
        Ok(value) => Ok(convert_toml_to_value(&value, span)),

        Err(_x) => Err(ShellError::CantConvert(
            "structured data from toml".into(),
            "string".into(),
            span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(FromToml {})
    }
}
