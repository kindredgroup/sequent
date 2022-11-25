// $coverage:ignore-start

use crate::commands::test_fixtures::{TestContext, TestState};
use crate::commands::timeline::{timeline, Parser, Timeline};
use sequent::{Event, Named, Queue, Scenario, Simulation, SimulationError, TransitionError};
use revolver::command::{assert_pedantic, ApplyOutcome, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{Mock, PrintOutput};
use stanza::renderer::console::{Console, Decor};
use stanza::renderer::Renderer;
use std::borrow::Cow;

fn command_parsers<'d>(
) -> Vec<Box<dyn NamedCommandParser<TestContext, SimulationError<TestState>, Mock<'d>>>> {
    vec![Box::new(Parser)]
}

#[test]
fn apply() {
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    assert_eq!(ApplyOutcome::Applied, Timeline.apply(&mut looper).unwrap());
    assert!(!looper.terminal().invocations()[0]
        .print()
        .unwrap_output()
        .is_empty());
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("timeline").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser);
}

#[derive(Debug, Clone)]
struct SampleState;

#[derive(Debug)]
struct SampleEvent {
    args: Vec<char>,
}

impl Named for SampleEvent {
    fn name(&self) -> Cow<'static, str> {
        "test-event".into()
    }
}

impl ToString for SampleEvent {
    fn to_string(&self) -> String {
        self.args
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Event<SampleState> for SampleEvent {
    fn apply(&self, _: &mut SampleState, _: &mut Queue<SampleState>) -> Result<(), TransitionError> {
        Ok(())
    }
}

#[test]
fn timeline_content() {
    let scenario = Scenario {
        initial: SampleState,
        timeline: vec![
            Box::new(SampleEvent {
                args: vec!['a', 'b'],
            }),
            Box::new(SampleEvent {
                args: vec!['c', 'd'],
            }),
        ],
    };
    let mut simulation = Simulation::from(scenario);
    let renderer = Console(
        Decor::default()
            .suppress_escape_codes()
            .suppress_inner_horizontal_border(),
    );

    let s = renderer.render(&timeline(&simulation)).to_string();
    assert_eq!(
        "\
    ╔═╤═╤═══════════════╤════════════════════════════════════════╗\n\
    ║ │ │Event name     │Encoded event arguments                 ║\n\
    ║▶│0│test-event     │a b                                     ║\n\
    ║ │1│test-event     │c d                                     ║\n\
    ╚═╧═╧═══════════════╧════════════════════════════════════════╝",
        s
    );

    simulation.step().unwrap();
    let s = renderer.render(&timeline(&simulation)).to_string();
    assert_eq!(
        "\
    ╔═╤═╤═══════════════╤════════════════════════════════════════╗\n\
    ║ │ │Event name     │Encoded event arguments                 ║\n\
    ║ │0│test-event     │a b                                     ║\n\
    ║▶│1│test-event     │c d                                     ║\n\
    ╚═╧═╧═══════════════╧════════════════════════════════════════╝",
        s
    );

    simulation.step().unwrap();
    let s = renderer.render(&timeline(&simulation)).to_string();
    assert_eq!(
        "\
    ╔═╤═╤═══════════════╤════════════════════════════════════════╗\n\
    ║ │ │Event name     │Encoded event arguments                 ║\n\
    ║ │0│test-event     │a b                                     ║\n\
    ║ │1│test-event     │c d                                     ║\n\
    ║▶│2│               │                                        ║\n\
    ╚═╧═╧═══════════════╧════════════════════════════════════════╝",
        s
    );
}
