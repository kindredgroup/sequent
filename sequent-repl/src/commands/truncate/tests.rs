// $coverage:ignore-start

use crate::commands::test_fixtures::{TestContext, TestState};
use crate::commands::truncate::{Parser, Truncate};
use sequent::SimulationError;
use revolver::command::{assert_pedantic, ApplyOutcome, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{lines, Mock, PrintOutput};
use crate::Context;

fn command_parsers<'d>(
) -> Vec<Box<dyn NamedCommandParser<TestContext, SimulationError<TestState>, Mock<'d>>>> {
    vec![Box::new(Parser)]
}

#[test]
fn apply_with_no() {
    let mut term = Mock::default().on_read_line(lines(&["no"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    assert_eq!(ApplyOutcome::Skipped, Truncate.apply(&mut looper).unwrap());
    assert_eq!(
        "Truncate timeline from cursor location 0? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    assert_eq!(4, looper.context().sim().scenario().timeline.len());
}

#[test]
fn apply_with_yes() {
    let mut term = Mock::default().on_read_line(lines(&["yes"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    assert_eq!(ApplyOutcome::Applied, Truncate.apply(&mut looper).unwrap());
    assert_eq!(
        "Truncate timeline from cursor location 0? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    assert_eq!(0, looper.context().sim().scenario().timeline.len());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("truncate").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser);
}