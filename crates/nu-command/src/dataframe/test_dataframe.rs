use nu_engine::eval_block;
use nu_parser::parse;
use nu_protocol::{
    engine::{Command, EngineState, Stack, StateWorkingSet},
    PipelineData, Span, Value, CONFIG_VARIABLE_ID,
};

use super::ToDataFrame;
use crate::Let;

pub fn test_dataframe(cmd: impl Command + 'static) {
    let examples = cmd.examples();
    let mut engine_state = Box::new(EngineState::new());

    let delta = {
        // Base functions that are needed for testing
        // Try to keep this working set small to keep tests running as fast as possible
        let mut working_set = StateWorkingSet::new(&*engine_state);
        working_set.add_decl(Box::new(Let));
        working_set.add_decl(Box::new(ToDataFrame));

        // Adding the command that is being tested to the working set
        working_set.add_decl(Box::new(cmd));

        working_set.render()
    };

    let _ = engine_state.merge_delta(delta);

    for example in examples {
        // Skip tests that don't have results to compare to
        if example.result.is_none() {
            continue;
        }
        let start = std::time::Instant::now();

        let (block, delta) = {
            let mut working_set = StateWorkingSet::new(&*engine_state);
            let (output, err) = parse(&mut working_set, None, example.example.as_bytes(), false);

            if let Some(err) = err {
                panic!("test parse error in `{}`: {:?}", example.example, err)
            }

            (output, working_set.render())
        };

        let _ = engine_state.merge_delta(delta);

        let mut stack = Stack::new();

        // Set up our initial config to start from
        stack.vars.insert(
            CONFIG_VARIABLE_ID,
            Value::Record {
                cols: vec![],
                vals: vec![],
                span: Span::unknown(),
            },
        );

        match eval_block(
            &engine_state,
            &mut stack,
            &block,
            PipelineData::new(Span::unknown()),
        ) {
            Err(err) => panic!("test eval error in `{}`: {:?}", example.example, err),
            Ok(result) => {
                let result = result.into_value(Span::unknown());
                println!("input: {}", example.example);
                println!("result: {:?}", result);
                println!("done: {:?}", start.elapsed());

                // Note. Value implements PartialEq for Bool, Int, Float, String and Block
                // If the command you are testing requires to compare another case, then
                // you need to define its equality in the Value struct
                if let Some(expected) = example.result {
                    if result != expected {
                        panic!(
                            "the example result is different to expected value: {:?} != {:?}",
                            result, expected
                        )
                    }
                }
            }
        }
    }
}
