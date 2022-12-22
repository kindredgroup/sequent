//! Contains the bulk of the simulation logic.

use crate::persistence::{ReadScenarioError, WriteScenarioError};
use crate::{Event, Queue, Scenario, TransitionError};
use thiserror::Error;
use crate::event::process_insertions;

/// The overarching simulation state. Contains the scenario being modelled, the current state
/// of the simulation, as well as a cursor pointing to the next event in the timeline that is scheduled
/// to be applied.
#[derive(Debug)]
pub struct Simulation<S> {
    scenario: Scenario<S>,
    current_state: S,
    cursor: usize,
}

impl<S: Default + Clone> Default for Simulation<S> {
    fn default() -> Self {
        Simulation::from(Scenario::default())
    }
}

/// Methods for affecting control over the simulation. Most of these emit a broad [`SimulationError`]
/// if _something_ goes wrong, with a specific error variant for each envisaged error type. Not all
/// methods use every available [`SimulationError`] variant.
///
/// Check the documentation of each method for the specific
/// variants that are returned by that method, bearing in mind that the variants that a method
/// is allowed to return may change over time.
impl<S> Simulation<S> {
    /// Evaluates the next event in the timeline, applying it to the current state
    /// to transition to the next state.
    ///
    /// # Errors
    /// [`SimulationError`] if an error occurs. Expected variants:
    ///
    /// * [`SimulationError::TimelineExhausted`], if the cursor is already parked at the end of the timeline.
    /// * [`SimulationError::Transition`], if the event could not be evaluated.
    pub fn step(&mut self) -> Result<(), SimulationError<S>> {
        if self.cursor == self.scenario.timeline.len() {
            return Err(SimulationError::TimelineExhausted);
        }
        let event = &self.scenario.timeline[self.cursor];
        let mut queue = Queue::new(self.cursor + 1, &self.scenario.timeline);
        event.apply(&mut self.current_state, &mut queue)?;
        let (offset, _, insertions) = queue.into_inner();
        process_insertions(offset, insertions, &mut self.scenario.timeline);
        self.cursor += 1;
        Ok(())
    }

    /// Resets the simulation, reinitialising the current state from the initial state
    /// specified in the simulation scenario, and resetting the cursor to location 0.
    pub fn reset(&mut self)
    where
        S: Clone,
    {
        self.current_state = self.scenario.initial.clone();
        self.cursor = 0;
    }

    /// Jumps to a specified location in the timeline and evaluates the event at that location.
    ///
    /// # Errors
    /// [`SimulationError`] if an error occurs. Expected variants:
    ///
    /// * [`SimulationError::TimelineExhausted`], if the cursor is already parked at the end of the timeline.
    /// * [`SimulationError::Transition`], if the event could not be evaluated.
    pub fn jump(&mut self, location: usize) -> Result<(), SimulationError<S>>
    where
        S: Clone,
    {
        if location > self.scenario.timeline.len() {
            return Err(SimulationError::TimelineExhausted);
        }

        if location < self.cursor {
            self.reset();
        }

        while self.cursor < location {
            self.step()?;
        }

        Ok(())
    }

    /// Evaluates the remaining events in the timeline.
    ///
    /// # Errors
    /// [`SimulationError`] if an error occurs. Expected variants:
    ///
    /// * [`SimulationError::TimelineExhausted`], if the cursor is already parked at the end of the timeline.
    /// * [`SimulationError::Transition`], if the event could not be evaluated.
    pub fn run(&mut self) -> Result<(), SimulationError<S>> {
        while self.cursor < self.scenario.timeline.len() {
            self.step()?;
        }

        Ok(())
    }

    /// Appends an event to the timeline at the current cursor location, assuming that there
    /// are no events at and beyond that location.
    ///
    /// # Errors
    /// [`SimulationError`] if an error occurs. Expected variants:
    ///
    /// * [`SimulationError::TruncationRequired`], if there is already an event
    /// at the cursor location. The error returns the event object that is otherwise consumed by
    /// this method.
    pub fn push_event(&mut self, event: Box<dyn Event<State = S>>) -> Result<(), SimulationError<S>> {
        if self.cursor != self.scenario.timeline.len() {
            return Err(SimulationError::TruncationRequired(event));
        }
        self.scenario.timeline.push(event);
        Ok(())
    }

    /// Truncates the timeline at the current cursor location, dropping all events at and beyond
    /// this point.
    pub fn truncate(&mut self) {
        self.scenario.timeline.truncate(self.cursor);
    }

    /// A reference to the underlying scenario.
    pub fn scenario(&self) -> &Scenario<S> {
        &self.scenario
    }

    /// Assigns a new scenario, resetting the simulation in the process.
    pub fn set_scenario(&mut self, scenario: Scenario<S>)
    where
        S: Clone,
    {
        self.scenario = scenario;
        self.reset();
    }

    /// A reference to the current simulation state.
    pub fn current_state(&self) -> &S {
        &self.current_state
    }

    /// The current cursor location.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

impl<S: Clone> From<Scenario<S>> for Simulation<S> {
    fn from(scenario: Scenario<S>) -> Self {
        let current_state = scenario.initial.clone();
        Self {
            scenario,
            current_state,
            cursor: 0,
        }
    }
}

/// Known errors that could be produced during the course of simulation, including the loading
/// and saving of simulation scenarios.
#[derive(Debug, Error)]
pub enum SimulationError<S> {
    #[error("timeline exhausted")]
    TimelineExhausted,

    #[error("transition: {0}")]
    Transition(#[from] TransitionError),

    #[error("truncation required")]
    TruncationRequired(Box<dyn Event<State = S>>),

    #[error("read scenario: {0}")]
    ReadScenario(#[from] ReadScenarioError),

    #[error("write scenario: {0}")]
    WriteScenario(#[from] WriteScenarioError),
}

/// Conversions from the blanket [`SimulationError`] type to the underlying variant arguments.
impl<S> SimulationError<S> {
    /// Returns `true` if and only if this is a [`SimulationError::TimelineExhausted`] variant.
    pub fn is_timeline_exhausted(&self) -> bool {
        matches!(self, Self::TimelineExhausted)
    }

    /// Converts the error into a [`Option<TransitionError>`].
    pub fn transition(self) -> Option<TransitionError> {
        match self {
            SimulationError::Transition(err) => Some(err),
            _ => None,
        }
    }

    /// Converts the error into a [`Option<TruncationRequired>`].
    pub fn truncation_required(self) -> Option<Box<dyn Event<State = S>>> {
        match self {
            SimulationError::TruncationRequired(err) => Some(err),
            _ => None,
        }
    }

    /// Converts the error into a [`Option<ReadScenarioError>`].
    pub fn read_scenario(self) -> Option<ReadScenarioError> {
        match self {
            SimulationError::ReadScenario(err) => Some(err),
            _ => None,
        }
    }

    /// Converts the error into a [`Option<WriteScenarioError>`].
    pub fn write_scenario(self) -> Option<WriteScenarioError> {
        match self {
            SimulationError::WriteScenario(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
