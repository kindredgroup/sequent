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
use std::marker::PhantomData;

/// Command to 'jump' to a specific event in the timeline. Upon completion, the simulation state
/// will reflect the sequential application of all events up to but not including the one at
/// the specified cursor location. Equivalently, 'jump 0' has the effect of resetting the simulation
/// state.
pub struct Jump<S, C> {
    location: usize,
    __phantom_data: PhantomData<(S, C)>
}

impl<S, C> Jump<S, C> {
    fn new(location: usize) -> Self {
        Self {
            location,
            __phantom_data: PhantomData::default()
        }
    }
}

impl<S: Clone, C: Context<State = S>, T: Terminal> Command<T> for Jump<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

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

impl<S: Clone + 'static, C: Context<State = S> + 'static, T: Terminal> NamedCommandParser<T> for Parser<S, C> {
    type Context = C;
    type Error = SimulationError<S>;

    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<T, Context = C, Error = SimulationError<S>>>, ParseCommandError> {
        if s.is_empty() {
            return Err(ParseCommandError(
                "empty arguments to 'jump'".into(),
            ));
        }
        let location = s.parse().map_err(ParseCommandError::convert)?;
        Ok(Box::new(Jump::new(location)))
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
