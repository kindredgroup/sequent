//! Resetting of the simulation.

use std::borrow::Cow;
use std::marker::PhantomData;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to reset the simulation to its initial state. Upon completion, the current state will be
/// a replica of the initial state depicted in the scenario, and the cursor location will be set to 0.
pub struct Reset<S, C> {
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Default for Reset<S, C> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S: Clone, C: Context<S>, T: Terminal> Command<T> for Reset<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        looper.context().sim().reset();
        let (terminal, _, context) = looper.split();
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Reset`].
pub struct Parser<S, C> {
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Default for Parser<S, C> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S: Clone + 'static, C: Context<S> + 'static, T: Terminal> NamedCommandParser<T> for Parser<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C, Error = SimulationError<S>>>, ParseCommandError> {
        self.parse_no_args(s, Reset::default)
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