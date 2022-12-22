//! Stepping through the next event.

use std::borrow::Cow;
use std::marker::PhantomData;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to step through the next event in the timeline. Upon completion, the current simulation state
/// will reflect the sequential application of all events up to and including the upcoming event, and
/// the cursor location will advance to the subsequent event (one after the upcoming event).
pub struct Next<S, C> {
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Default for Next<S, C> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S, C: Context<S>, T: Terminal> Command<T> for Next<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.sim().step().map_err(ApplyCommandError::Application)?;
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Next`].
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

impl<S: 'static, C: Context<S> + 'static, T: Terminal> NamedCommandParser<T> for Parser<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C, Error = SimulationError<S>>>, ParseCommandError> {
        self.parse_no_args(s, Next::default)
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
