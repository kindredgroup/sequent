// $coverage:ignore-start

use crate::commands::load::{Load, Parser};
use crate::commands::test_fixtures::{write_str_to_file, TestContext, TestState};
use crate::Context;
use sequent::persistence::yaml::write_to_file;
use sequent::SimulationError;
use flanker_temp::TempPath;
use revolver::command::{assert_pedantic, ApplyOutcome, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{Mock, PrintOutput};

fn command_parsers<'d>(
) -> Vec<Box<dyn NamedCommandParser<TestContext, SimulationError<TestState>, Mock<'d>>>> {
    vec![Box::new(Parser)]
}

#[test]
fn apply() {
    let temp = TempPath::with_extension("yaml");
    {
        let mut context = TestContext::new(8);
        write_to_file(context.sim().scenario(), &temp).unwrap();
    }

    let mut term =  Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::new(4);
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut load = Load {
        path: temp.as_ref().to_string_lossy().to_string(),
    };
    assert_eq!(ApplyOutcome::Applied, load.apply(&mut looper).unwrap());
    assert!(!looper.terminal().invocations()[0]
        .print()
        .unwrap_output()
        .is_empty());
    assert_eq!(8, looper.context().sim().scenario().timeline.len());
}

#[test]
fn apply_corrupt_file() {
    const DUMMY_DATA: &str = "dummy data";
    let temp = TempPath::with_extension("yaml");
    write_str_to_file(&temp, DUMMY_DATA);

    let mut term =  Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::new(4);
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut load = Load {
        path: temp.as_ref().to_string_lossy().to_string(),
    };
    assert!(load
        .apply(&mut looper)
        .unwrap_err()
        .application()
        .unwrap()
        .read_scenario()
        .unwrap()
        .deserializer()
        .is_some());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("load in.yaml").unwrap();
}

#[test]
#[should_panic(expected = "empty arguments to 'load'")]
fn parse_empty_args_fails() {
    let commander = Commander::new(command_parsers());
    commander.parse("load").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser);
}
