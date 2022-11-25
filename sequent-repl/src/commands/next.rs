//! Stepping through the next event.

use std::borrow::Cow;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to step through the next event in the timeline. Upon completion, the current simulation state
/// will reflect the sequential application of all events up to and including the upcoming event, and
/// the cursor location will advance to the subsequent event (one after the upcoming event).
pub struct Next;

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Next {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.sim().step().map_err(ApplyCommandError::Application)?;
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Next`].
pub struct Parser;

impl<S, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        self.parse_no_args(s, || Next)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("n".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "next".into()
    }
    
    fn description(&self) -> Description {
        Description {
            purpose: "Steps through the next event in the timeline.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[cfg(test)]
mod tests;
