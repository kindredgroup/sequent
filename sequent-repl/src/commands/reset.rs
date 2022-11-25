//! Resetting of the simulation.

use std::borrow::Cow;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to reset the simulation to its initial state. Upon completion, the current state will be
/// a replica of the initial state depicted in the scenario, and the cursor location will be set to 0.
pub struct Reset;

impl<S: Clone, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Reset {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        looper.context().sim().reset();
        let (terminal, _, context) = looper.split();
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Reset`].
pub struct Parser;

impl<S: Clone, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        self.parse_no_args(s, || Reset)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn name(&self) -> Cow<'static, str> {
        "reset".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Resets the simulation to its initial state.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[cfg(test)]
mod tests;