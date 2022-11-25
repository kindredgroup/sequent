//! Evaluation of the remaining events in the timeline.

use std::borrow::Cow;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to evaluate the remaining events in the timeline. By completion, the simulation state will
/// reflect the sequential application of all events.
pub struct Run;

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Run {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.sim().run().map_err(ApplyCommandError::Application)?;
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Run`].
pub struct Parser;

impl<S, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        self.parse_no_args(s, || Run)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("r".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "run".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Evaluates the remaining events in the timeline.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[cfg(test)]
mod tests;
