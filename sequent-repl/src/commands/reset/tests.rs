// $coverage:ignore-start

use sequent::SimulationError;
use revolver::command::{ApplyOutcome, assert_pedantic, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{Mock, PrintOutput};
use crate::commands::reset::{Parser, Reset};
use crate::commands::test_fixtures::{TestContext, TestState};
use crate::Context;

fn command_parsers<'d>() -> Vec<Box<dyn NamedCommandParser<TestContext, SimulationError<TestState>, Mock<'d>>>> {
    vec! [
        Box::new(Parser)
    ]
}

#[test]
fn apply() {
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.context().sim().step().unwrap();
    assert_eq!(1, looper.context().sim().cursor());
    assert_eq!(ApplyOutcome::Applied, Reset.apply(&mut looper).unwrap());
    assert!(!looper.terminal().invocations()[0].print().unwrap_output().is_empty());
    assert_eq!(0, looper.context().sim().cursor());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("reset").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser);
}