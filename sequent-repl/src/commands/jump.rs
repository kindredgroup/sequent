//! Navigation to a point in the timeline.

use crate::Context;
use sequent::SimulationError;
use revolver::command::{
    ApplyCommandError, ApplyOutcome, Command, Description, Example, NamedCommandParser,
    ParseCommandError,
};
use revolver::looper::Looper;
use revolver::terminal::{Terminal};
use std::borrow::Cow;

/// Command to 'jump' to a specific event in the timeline. Upon completion, the simulation state
/// will reflect the sequential application of all events up to but not including the one at
/// the specified cursor location. Equivalently, 'jump 0' has the effect of resetting the simulation
/// state.
pub struct Jump {
    location: usize,
}

impl<S: Clone, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Jump {
    fn apply(
        &mut self,
        looper: &mut Looper<C, SimulationError<S>, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.sim().jump(self.location).map_err(ApplyCommandError::Application)?;
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Jump`].
pub struct Parser;

impl<S: Clone, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        if s.is_empty() {
            return Err(ParseCommandError(
                "empty arguments to 'jump'".into(),
            ));
        }
        let location = s.parse().map_err(ParseCommandError::convert)?;
        Ok(Box::new(Jump { location }))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("j".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "jump".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Jumps to a specified cursor location in the timeline.".into(),
            usage: "<location>".into(),
            examples: vec![Example {
                scenario: "jump to location 2".into(),
                command: "2".into(),
            }],
        }
    }
}

#[cfg(test)]
mod tests;
