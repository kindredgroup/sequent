//! Printing of the simulation state.

use std::borrow::Cow;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command that prints the current simulation state to the terminal device.
pub struct Print;

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Print {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Print`].
pub struct Parser;

impl<S, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        self.parse_no_args(s, || Print)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("p".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "print".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Prints the current simulation state.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[cfg(test)]
mod tests;
