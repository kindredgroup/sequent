// $coverage:ignore-start

use crate::commands::event_proxy::{EventProxy, Parser};
use crate::commands::test_fixtures::{Append, TestContext, TestState};
use crate::Context;
use sequent::{SimulationError, TransitionError};
use revolver::command::{
    assert_pedantic, ApplyOutcome, Command, Commander, Description, NamedCommandParser,
};
use revolver::looper::Looper;
use revolver::terminal::{AccessTerminalError, lines, Mock, PrintOutput};
use std::borrow::Cow;

fn parser() -> Parser<TestState, Append> {
    Parser::new(
        None,
        Description {
            purpose: "Appends an ID tag.".into(),
            usage: Cow::default(),
            examples: vec![],
        },
    )
}

fn command_parsers<'d>(
) -> Vec<Box<dyn NamedCommandParser<TestContext, SimulationError<TestState>, Mock<'d>>>> {
    vec![Box::new(parser())]
}

#[test]
fn apply_at_cursor() {
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::new(0);
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut proxy = EventProxy {
        event: Some(Box::new(Append { id: 0 })),
    };
    assert_eq!(ApplyOutcome::Applied, proxy.apply(&mut looper).unwrap());
    assert_eq!(vec![0], looper.context().sim().current_state().transitions);
}

#[test]
fn apply_with_truncate_yes() {
    let mut term = Mock::default().on_read_line(lines(&["yes"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut proxy = EventProxy {
        event: Some(Box::new(Append { id: 0 })),
    };
    assert_eq!(ApplyOutcome::Applied, proxy.apply(&mut looper).unwrap());
    assert_eq!(
        "Inserting into the timeline requires truncation. Continue? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    assert_eq!(vec![0], looper.context().sim().current_state().transitions);
}

#[test]
fn apply_with_truncate_no() {
    let mut term = Mock::default().on_read_line(lines(&["no"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context
    );
    let mut proxy = EventProxy {
        event: Some(Box::new(Append { id: 0 })),
    };
    assert_eq!(ApplyOutcome::Skipped, proxy.apply(&mut looper).unwrap());
    assert_eq!(
        "Inserting into the timeline requires truncation. Continue? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    assert_eq!(
        vec![] as Vec<usize>,
        looper.context().sim().current_state().transitions
    );
}

#[test]
fn apply_with_truncate_terminal_error() {
    let mut term =
        Mock::default().on_read_line(|| Err(AccessTerminalError("terminal exploded".into())));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut proxy = EventProxy {
        event: Some(Box::new(Append { id: 0 })),
    };
    assert_eq!(
        AccessTerminalError("terminal exploded".into()),
        proxy.apply(&mut looper)
            .unwrap_err()
            .access_terminal()
            .unwrap());
}

#[test]
fn apply_with_duplicate_raises_transition_error() {
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::new(1);
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut proxy = EventProxy {
        event: Some(Box::new(Append { id: 0 })),
    };
    looper.context().sim().step().unwrap();
    assert_eq!(vec![0], looper.context().sim().current_state().transitions);
    assert_eq!(
        TransitionError("duplicate ID 0".into()),
        proxy
            .apply(&mut looper)
            .unwrap_err()
            .application()
            .unwrap()
            .transition()
            .unwrap()
    );
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("append 42").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&parser());
}
