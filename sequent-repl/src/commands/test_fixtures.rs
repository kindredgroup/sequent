//! Reusable test fixtures.

// $coverage:ignore-start

use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::{Context};
use sequent::{Decoder, Event, NamedEventParser, ParseEventError, Parser, Queue, Scenario, Simulation, StaticNamed, TransitionError};
use revolver::terminal::{AccessTerminalError, Terminal};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Test state, which simply accumulates a vector of ID tags corresponding to the
/// [`Append`] events that have been applied to it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestState {
    pub transitions: Vec<usize>,
}

impl ToString for TestState {
    fn to_string(&self) -> String {
        let s = self
            .transitions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{s}]")
    }
}

/// Appends an ID tag to [`TestState::transitions`].
#[derive(Debug)]
pub struct Append {
    pub id: usize,
}

impl ToString for Append {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

/// Produced by a broad range of types during parsing, typically when calling
/// [`FromStr`](std::str::FromStr).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct ParseError(pub Cow<'static, str>);

impl ParseError {
    /// Converts anything representable as a [`String`] into a [`ParseError`], consuming
    /// the original. This is mostly used in error conversion; e.g., in [`Result::map_err()`].
    #[allow(clippy::needless_pass_by_value)]
    pub fn convert<E: ToString>(err: E) -> Self {
        Self(err.to_string().into())
    }
}

impl From<ParseError> for ParseEventError {
    fn from(err: ParseError) -> Self {
        Self(err.0)
    }
}

impl FromStr for Append {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = usize::from_str(s).map_err(ParseError::convert)?;
        Ok(Self { id })
    }
}

impl StaticNamed for Append {
    fn name() -> &'static str {
        "append"
    }
}

impl Event for Append {
    type State = TestState;

    fn apply(&self, state: &mut TestState, _: &mut Queue<'_, TestState>) -> Result<(), TransitionError> {
        if state.transitions.contains(&self.id) {
            Err(TransitionError(format!("duplicate ID {}", self.id).into()))
        } else {
            state.transitions.push(self.id);
            Ok(())
        }
    }
}

/// Minimal context for testing.
pub struct TestContext {
    sim: Simulation<TestState>,
    decoder: Decoder<TestState>,
}

impl Context<TestState> for TestContext {
    fn sim(&mut self) -> &mut Simulation<TestState> {
        &mut self.sim
    }

    fn print_state(&self, terminal: &mut impl Terminal) -> Result<(), AccessTerminalError> {
        terminal.print_line(&self.sim.current_state().to_string())
    }

    fn decoder(&self) -> &Decoder<TestState> {
        &self.decoder
    }
}

fn event_parsers() -> Vec<Box<dyn NamedEventParser<State = TestState>>> {
    vec![Box::new(Parser::<Append>::default())]
}

impl TestContext {
    pub fn new(num_events: usize) -> Self {
        let scenario = Scenario {
            initial: TestState::default(),
            timeline: (0..num_events)
                .map(|id| Box::new(Append { id }) as Box<dyn Event<State = TestState>>)
                .collect(),
        };
        let sim = Simulation::from(scenario);
        let decoder = Decoder::new(event_parsers());
        Self { sim, decoder }
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new(4)
    }
}

/// Writes a string slice to the specified path.
///
/// # Panics
/// If an I/O error occurs.
pub fn write_str_to_file<P: AsRef<Path>>(path: P, s: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(s.as_bytes()).unwrap();
}

/// Reads a string from the specified path.
///
/// # Panics
/// If an I/O error occurs.
pub fn read_str_from_file<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut buf = String::default();
    file.read_to_string(&mut buf).unwrap();
    buf
}