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
pub struct EventProxy<S, C> {
    event: Option<Box<dyn Event<State = S>>>,
    __phantom_data: PhantomData<C>
}

impl<S, C> EventProxy<S, C> {
    pub fn new(event: Option<Box<dyn Event<State = S>>>) -> Self {
        Self {
            event,
            __phantom_data: PhantomData::default(),
        }
    }
}

impl<S, C: Context<S>, T: Terminal> Command<T> for EventProxy<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

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
pub struct Parser<S, C, E: Event<State = S>> {
    shorthand: Option<Cow<'static, str>>,
    name: &'static str,
    description: Description,
    __phantom_data: PhantomData<(S, C, E)>,
}

impl<S, C, E> Parser<S, C, E>
    where
        E: Event<State = S> + FromStr + StaticNamed,
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

impl<S: 'static, C: Context<S> + 'static, E, T: Terminal> NamedCommandParser<T> for Parser<S, C, E>
    where
        E: FromStr + Event<State = S> + 'static,
        E::Err: ToString
{
    type Context = C;
    type Error = SimulationError<S>;

    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C, Error = SimulationError<S>>>, ParseCommandError> {
        let event = E::from_str(s).map_err(ParseCommandError::convert)?;
        let proxy = EventProxy::new(Some(Box::new(event)));
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