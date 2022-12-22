// $coverage:ignore-start

use crate::persistence::{ReadScenarioError, WriteScenarioError};
use crate::{Event, Queue, Scenario, Simulation, SimulationError, StaticNamed, TransitionError};
use std::io;
use std::io::ErrorKind;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TestState {
    transitions: Vec<usize>,
}

#[derive(Debug)]
struct Append {
    id: usize,
}

impl ToString for Append {
    fn to_string(&self) -> String {
        format!("{}", self.id)
    }
}

impl StaticNamed for Append {
    fn name() -> &'static str {
        "append"
    }
}

impl Event for Append {
    type State = TestState;

    fn apply(
        &self,
        state: &mut Self::State,
        _: &mut Queue<Self::State>,
    ) -> Result<(), TransitionError> {
        state.transitions.push(self.id);
        Ok(())
    }
}

fn fixture() -> Scenario<TestState> {
    const EVENTS: usize = 4;
    Scenario {
        initial: TestState::default(),
        timeline: (0..EVENTS)
            .map(|id| Box::new(Append { id }) as Box<dyn Event<State = TestState>>)
            .collect(),
    }
}

#[test]
fn default_simulation() {
    let sim = Simulation::<TestState>::default();
    assert_eq!(0, sim.scenario().timeline.len());
    assert_eq!(TestState::default(), sim.scenario().initial);
    assert_eq!(&TestState::default(), sim.current_state());
}

#[test]
fn step_and_reset() {
    let mut sim = Simulation::from(fixture());
    assert_eq!(vec![] as Vec<usize>, sim.current_state().transitions);
    assert_eq!(0, sim.cursor());

    sim.step().unwrap();
    assert_eq!(vec![0], sim.current_state().transitions);
    assert_eq!(1, sim.cursor());

    sim.step().unwrap();
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());

    sim.step().unwrap();
    assert_eq!(vec![0, 1, 2], sim.current_state().transitions);
    assert_eq!(3, sim.cursor());

    sim.step().unwrap();
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());

    assert!(sim.step().unwrap_err().is_timeline_exhausted());
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());

    sim.reset();
    assert_eq!(vec![] as Vec<usize>, sim.current_state().transitions);
    assert_eq!(0, sim.cursor());

    sim.step().unwrap();
    assert_eq!(vec![0], sim.current_state().transitions);
    assert_eq!(1, sim.cursor());
}

#[test]
fn jump() {
    let mut sim = Simulation::from(fixture());
    sim.jump(2).unwrap();
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());

    sim.jump(2).unwrap(); // no change
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());

    sim.jump(4).unwrap();
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());

    assert!(sim.jump(5).unwrap_err().is_timeline_exhausted());
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());

    sim.jump(2).unwrap();
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());

    sim.jump(1).unwrap();
    assert_eq!(vec![0], sim.current_state().transitions);
    assert_eq!(1, sim.cursor());

    sim.jump(0).unwrap(); // equivalent to a reset
    assert_eq!(vec![] as Vec<usize>, sim.current_state().transitions);
    assert_eq!(0, sim.cursor());
}

#[test]
fn run() {
    let mut sim = Simulation::from(fixture());
    sim.run().unwrap();
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());
}

#[test]
fn push_event() {
    let mut sim = Simulation::from(fixture());
    sim.jump(4).unwrap();
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());
    assert_eq!(4, sim.scenario().timeline.len());

    // push events at the end of the timeline -- should work
    sim.push_event(Box::new(Append { id: 4 })).unwrap();
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());
    assert_eq!(5, sim.scenario().timeline.len());

    // push again at the same cursor location (one off the end) -- should fail
    assert!(sim
        .push_event(Box::new(Append { id: 4 }))
        .unwrap_err()
        .truncation_required()
        .is_some());
    assert_eq!(vec![0, 1, 2, 3], sim.current_state().transitions);
    assert_eq!(4, sim.cursor());
    assert_eq!(5, sim.scenario().timeline.len());
}

#[test]
fn truncate() {
    let mut sim = Simulation::from(fixture());
    sim.truncate();
    assert_eq!(vec![] as Vec<usize>, sim.current_state().transitions);
    assert_eq!(0, sim.cursor());
    assert_eq!(0, sim.scenario().timeline.len());

    let mut sim = Simulation::from(fixture());
    sim.jump(2).unwrap();
    sim.truncate();
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());
    assert_eq!(2, sim.scenario().timeline.len());

    // repeat truncation does nothing
    sim.truncate();
    assert_eq!(vec![0, 1], sim.current_state().transitions);
    assert_eq!(2, sim.cursor());
    assert_eq!(2, sim.scenario().timeline.len());
}

#[derive(Debug)]
struct Faulty;

impl ToString for Faulty {
    fn to_string(&self) -> String {
        "".into()
    }
}

impl StaticNamed for Faulty {
    fn name() -> &'static str {
        "faulty"
    }
}

impl Event for Faulty {
    type State = TestState;

    fn apply(&self, _: &mut Self::State, _: &mut Queue<Self::State>) -> Result<(), TransitionError> {
        Err(TransitionError("boom".into()))
    }
}

#[test]
fn set_scenario_triggers_reset() {
    let mut sim = Simulation::from(fixture());
    sim.step().unwrap();
    assert_eq!(vec![0], sim.current_state().transitions);
    assert_eq!(1, sim.cursor());

    sim.set_scenario(fixture());
    assert_eq!(vec![] as Vec<usize>, sim.current_state().transitions);
    assert_eq!(0, sim.cursor());
}

fn timeline_exhausted_error() -> SimulationError<TestState> {
    SimulationError::TimelineExhausted
}

fn transition_error() -> SimulationError<TestState> {
    SimulationError::Transition(TransitionError("bad transition".into()))
}

fn truncation_required_error() -> SimulationError<TestState> {
    SimulationError::TruncationRequired(Box::new(Append { id: 0 }))
}

fn read_scenario_error() -> SimulationError<TestState> {
    SimulationError::ReadScenario(ReadScenarioError::Io(io::Error::new(
        ErrorKind::BrokenPipe,
        "broken pipe",
    )))
}

fn write_scenario_error() -> SimulationError<TestState> {
    SimulationError::WriteScenario(WriteScenarioError::Io(io::Error::new(
        ErrorKind::BrokenPipe,
        "broken pipe",
    )))
}

#[test]
fn error_variants() {
    assert_eq!("timeline exhausted", timeline_exhausted_error().to_string());
    assert_eq!(
        "TimelineExhausted",
        format!("{:?}", timeline_exhausted_error())
    );
    assert!(timeline_exhausted_error().is_timeline_exhausted());
    assert!(timeline_exhausted_error().transition().is_none());

    assert_eq!("transition: bad transition", transition_error().to_string());
    assert_eq!(
        "Transition(TransitionError(\"bad transition\"))",
        format!("{:?}", transition_error())
    );
    assert!(transition_error().transition().is_some());
    assert!(transition_error().truncation_required().is_none());

    assert_eq!(
        "truncation required",
        truncation_required_error().to_string()
    );
    assert_eq!(
        "TruncationRequired(Append { id: 0 })",
        format!("{:?}", truncation_required_error())
    );
    assert!(truncation_required_error().truncation_required().is_some());
    assert!(truncation_required_error().read_scenario().is_none());

    assert_eq!(
        "read scenario: io: broken pipe",
        read_scenario_error().to_string()
    );
    assert_eq!(
        "ReadScenario(Io(Custom { kind: BrokenPipe, error: \"broken pipe\" }))",
        format!("{:?}", read_scenario_error())
    );
    assert!(read_scenario_error().read_scenario().is_some());
    assert!(read_scenario_error().write_scenario().is_none());

    assert_eq!(
        "write scenario: io: broken pipe",
        write_scenario_error().to_string()
    );
    assert_eq!(
        "WriteScenario(Io(Custom { kind: BrokenPipe, error: \"broken pipe\" }))",
        format!("{:?}", write_scenario_error())
    );
    assert!(write_scenario_error().write_scenario().is_some());
    assert!(write_scenario_error().read_scenario().is_none());
}

#[test]
fn simulation_implements_debug() {
    let sim = Simulation::<()>::default();
    let s = format!("{sim:?}");
    assert!(s.contains("Simulation"));
}

#[derive(Debug)]
struct UpdateQueue {
    insert_index: usize,
    id_to_insert: usize,
}

impl ToString for UpdateQueue {
    fn to_string(&self) -> String {
        format!("{}|{}", self.insert_index, self.id_to_insert)
    }
}

impl StaticNamed for UpdateQueue {
    fn name() -> &'static str {
        "append"
    }
}

impl Event for UpdateQueue {
    type State = TestState;

    fn apply(
        &self,
        _: &mut Self::State,
        queue: &mut Queue<Self::State>,
    ) -> Result<(), TransitionError> {
        queue.insert_later(
            self.insert_index,
            Box::new(Append {
                id: self.id_to_insert,
            }),
        );
        Ok(())
    }
}

fn slice_to_string(slice: &[Box<dyn Event<State = TestState>>]) -> String {
    let strings = slice
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    let joined = strings.join(", ");
    format!("[{}]", joined)
}

#[test]
fn insert_queue() {
    {
        let timeline: Vec<Box<dyn Event<State = TestState>>> = vec![Box::new(UpdateQueue {
            insert_index: 0,
            id_to_insert: 100,
        })];
        let scenario = Scenario {
            initial: TestState::default(),
            timeline,
        };
        let mut sim = Simulation::from(scenario);
        sim.step().unwrap();
        assert_eq!("[0|100, 100]", slice_to_string(&sim.scenario.timeline));
    }
    {
        let timeline: Vec<Box<dyn Event<State = TestState>>> = vec![
            Box::new(UpdateQueue {
                insert_index: 0,
                id_to_insert: 100,
            }),
            Box::new(UpdateQueue {
                insert_index: 6,
                id_to_insert: 600,
            }),
        ];
        let scenario = Scenario {
            initial: TestState::default(),
            timeline,
        };
        let mut sim = Simulation::from(scenario);
        sim.step().unwrap();
        assert_eq!("[0|100, 100, 6|600]", slice_to_string(&sim.scenario.timeline));
    }
    {
        let timeline: Vec<Box<dyn Event<State = TestState>>> = vec![
            Box::new(UpdateQueue {
                insert_index: 1,
                id_to_insert: 100,
            }),
            Box::new(UpdateQueue {
                insert_index: 6,
                id_to_insert: 600,
            }),
        ];
        let scenario = Scenario {
            initial: TestState::default(),
            timeline,
        };
        let mut sim = Simulation::from(scenario);
        sim.step().unwrap();
        assert_eq!("[1|100, 6|600, 100]", slice_to_string(&sim.scenario.timeline));
    }
}
