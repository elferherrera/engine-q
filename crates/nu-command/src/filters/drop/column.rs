use nu_engine::CallExt;
use nu_protocol::ast::{Call, CellPath};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, FromValue, IntoInterruptiblePipelineData, IntoPipelineData, PipelineData, ShellError,
    Signature, Span, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct DropColumn;

impl Command for DropColumn {
    fn name(&self) -> &str {
        "drop column"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "columns",
                SyntaxShape::Int,
                "starting from the end, the number of columns to remove",
            )
            .category(Category::Filters)
    }

    fn usage(&self) -> &str {
        "Remove the last number of columns. If you want to remove columns by name, try 'reject'."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        // the number of columns to drop
        let columns: Option<i64> = call.opt(engine_state, stack, 0)?;
        let span = call.head;

        let columns_to_drop = if let Some(quantity) = columns {
            quantity
        } else {
            1
        };

        dropcol(engine_state, span, input, columns_to_drop)
    }
}

fn dropcol(
    engine_state: &EngineState,
    span: Span,
    input: PipelineData,
    columns: i64, // the number of columns to drop
) -> Result<PipelineData, ShellError> {
    let mut keep_columns = vec![];

    match input {
        PipelineData::Value(
            Value::List {
                vals: input_vals,
                span,
            },
            ..,
        ) => {
            let mut output = vec![];
            let input_cols = get_input_cols(input_vals.clone());
            let kc = get_keep_columns(input_cols, columns);
            keep_columns = get_cellpath_columns(kc);

            for input_val in input_vals {
                let mut cols = vec![];
                let mut vals = vec![];

                for path in &keep_columns {
                    let fetcher = input_val.clone().follow_cell_path(&path.members)?;
                    cols.push(path.into_string());
                    vals.push(fetcher);
                }
                output.push(Value::Record { cols, vals, span })
            }

            Ok(output
                .into_iter()
                .into_pipeline_data(engine_state.ctrlc.clone()))
        }
        PipelineData::Stream(stream, ..) => {
            let mut output = vec![];

            let v: Vec<_> = stream.into_iter().collect();
            let input_cols = get_input_cols(v.clone());
            let kc = get_keep_columns(input_cols, columns);
            keep_columns = get_cellpath_columns(kc);

            for input_val in v {
                let mut cols = vec![];
                let mut vals = vec![];

                for path in &keep_columns {
                    let fetcher = input_val.clone().follow_cell_path(&path.members)?;
                    cols.push(path.into_string());
                    vals.push(fetcher);
                }
                output.push(Value::Record { cols, vals, span })
            }

            Ok(output
                .into_iter()
                .into_pipeline_data(engine_state.ctrlc.clone()))
        }
        PipelineData::Value(v, ..) => {
            let mut cols = vec![];
            let mut vals = vec![];

            for cell_path in &keep_columns {
                let result = v.clone().follow_cell_path(&cell_path.members)?;

                cols.push(cell_path.into_string());
                vals.push(result);
            }

            Ok(Value::Record { cols, vals, span }.into_pipeline_data())
        }
    }
}

fn get_input_cols(input: Vec<Value>) -> Vec<String> {
    let rec = input.first();
    match rec {
        Some(Value::Record { cols, vals: _, .. }) => cols.to_vec(),
        _ => vec!["".to_string()],
    }
}

fn get_cellpath_columns(keep_cols: Vec<String>) -> Vec<CellPath> {
    let mut output = vec![];
    for keep_col in keep_cols {
        let span = Span::unknown();
        let val = Value::String {
            val: keep_col,
            span,
        };
        let cell_path = match CellPath::from_value(&val) {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        output.push(cell_path);
    }
    output
}

fn get_keep_columns(input: Vec<String>, mut num_of_columns_to_drop: i64) -> Vec<String> {
    let vlen: i64 = input.len() as i64;

    if num_of_columns_to_drop > vlen {
        num_of_columns_to_drop = vlen;
    }

    let num_of_columns_to_keep = (vlen - num_of_columns_to_drop) as usize;
    input[0..num_of_columns_to_keep].to_vec()
}
