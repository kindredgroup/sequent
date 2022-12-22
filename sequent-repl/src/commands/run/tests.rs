// $coverage:ignore-start

use sequent::SimulationError;
use revolver::command::{ApplyOutcome, assert_pedantic, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{Mock, PrintOutput};
use crate::commands::run::{Parser, Run};
use crate::commands::test_fixtures::{TestContext, TestState};
use crate::Context;

fn command_parsers<'d>() -> Vec<Box<dyn NamedCommandParser<Mock<'d>, Context = TestContext, Error = SimulationError<TestState>>>> {
    vec! [
        Box::new(Parser::default())
    ]
}

#[test]
fn apply() {
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    assert_eq!(0, looper.context().sim().cursor());
    assert_eq!(ApplyOutcome::Applied, Run::default().apply(&mut looper).unwrap());
    assert!(!looper.terminal().invocations()[0].print().unwrap_output().is_empty());
    assert_eq!(4, looper.context().sim().cursor());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("run").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser::default());
}