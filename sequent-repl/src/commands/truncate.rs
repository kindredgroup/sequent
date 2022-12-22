//! Truncation of the event timeline.

use crate::commands::prompt::YesNo;
use crate::Context;
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::{Terminal};
use std::borrow::Cow;
use std::marker::PhantomData;
use sequent::SimulationError;

/// Command to truncate the event timeline at the current cursor location. The user will be given a
/// yes/no prompt before proceeding with truncation.
pub struct Truncate<S, C> {
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Default for Truncate<S, C> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S, C: Context<State = S>, T: Terminal> Command<T> for Truncate<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        let response = terminal.read_from_str_default(
            &format!(
                "Truncate timeline from cursor location {}? [y/N]: ",
                context.sim().cursor()
            ),
        )?;
        match response {
            YesNo::Yes => {
                looper.context().sim().truncate();
                Ok(ApplyOutcome::Applied)
            }
            YesNo::No => Ok(ApplyOutcome::Skipped),
        }
    }
}

/// Parser for [`Truncate`].
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
        self.parse_no_args(s, Truncate::default)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("tr".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "truncate".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Truncates the remaining events in the timeline.".into(),
            usage: Cow::default(),
            examples: Vec::default(),
        }
    }
}

#[cfg(test)]
mod tests;