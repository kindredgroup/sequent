// $coverage:ignore-start

use sequent::SimulationError;
use revolver::command::{ApplyOutcome, assert_pedantic, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{Mock, PrintOutput};
use crate::commands::print::{Parser, Print};
use crate::commands::test_fixtures::{TestContext, TestState};

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
    assert_eq!(ApplyOutcome::Applied, Print.apply(&mut looper).unwrap());
    assert_eq!("[]\n", looper.terminal().invocations()[0].print().unwrap_output());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("print").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser);
}