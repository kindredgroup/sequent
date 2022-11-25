//! Evaluation of a specific event.

use crate::commands::prompt::YesNo;
use crate::Context;
use sequent::{Event, SimulationError, StaticNamed};
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::{Terminal};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;

/// Command that delegates its evaluation to that of an [`Event`] object.
pub struct EventProxy<S> {
    event: Option<Box<dyn Event<S>>>,
}

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for EventProxy<S> {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let mut event = self.event.take().unwrap();
        loop {
            let result = looper.context().sim().push_event(event);
            match result {
                Ok(_) => {
                    let (terminal, _, context) = looper.split();
                    context.sim().step().map_err(ApplyCommandError::Application)?;
                    context.print_state(terminal)?;
                    return Ok(ApplyOutcome::Applied);
                }
                Err(SimulationError::TruncationRequired(rejected)) => {
                    event = rejected;
                    let response = looper.terminal().read_from_str_default(
                        "Inserting into the timeline requires truncation. Continue? [y/N]: ",
                    )?;
                    match response {
                        YesNo::Yes => {
                            looper.context().sim().truncate();
                        }
                        YesNo::No => {
                            return Ok(ApplyOutcome::Skipped);
                        }
                    }
                }
                Err(_) => unreachable!()
            }
        }
    }
}

/// Parser for [`EventProxy`] of some event type.
pub struct Parser<S, E: Event<S>> {
    shorthand: Option<Cow<'static, str>>,
    name: &'static str,
    description: Description,
    __phantom_data: PhantomData<(S, E)>,
}

impl<S, E> Parser<S, E>
    where
        E: Event<S> + FromStr + StaticNamed,
{
    /// Creates a new parser. The name is taken from the event type, provided the latter implements
    /// [`StaticNamed`].
    pub fn new(shorthand: Option<Cow<'static, str>>, description: Description) -> Self {
        Self {
            shorthand,
            name: <E as StaticNamed>::name(),
            description,
            __phantom_data: PhantomData::default(),
        }
    }
}

impl<S: 'static, C: Context<S>, E, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser<S, E>
    where
        E: FromStr + Event<S> + 'static,
        E::Err: ToString
{
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        let event = E::from_str(s).map_err(ParseCommandError::convert)?;
        let proxy = EventProxy {
            event: Some(Box::new(event)),
        };
        Ok(Box::new(proxy))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        self.shorthand.clone()
    }

    fn name(&self) -> Cow<'static, str> {
        self.name.into()
    }

    fn description(&self) -> Description {
        self.description.clone()
    }
}

#[cfg(test)]
mod tests;