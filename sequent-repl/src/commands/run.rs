//! Evaluation of the remaining events in the timeline.

use std::borrow::Cow;
use std::marker::PhantomData;
use sequent::SimulationError;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to evaluate the remaining events in the timeline. By completion, the simulation state will
/// reflect the sequential application of all events.
pub struct Run<S, C> {
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Default for Run<S, C> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S, C: Context<State = S>, T: Terminal> Command<T> for Run<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        context.sim().run().map_err(ApplyCommandError::Application)?;
        context.print_state(terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Run`].
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

impl<S: 'static, C: Context<State = S> + 'static, T: Terminal> NamedCommandParser<T> for Parser<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C, Error = SimulationError<S>>>, ParseCommandError> {
        self.parse_no_args(s, Run::default)
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
