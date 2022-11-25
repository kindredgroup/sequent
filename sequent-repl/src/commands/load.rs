//! Loading of a simulation from a YAML document.

use crate::{Context};
use sequent::persistence::yaml;
use sequent::{SimulationError};
use revolver::command::{
    ApplyCommandError, ApplyOutcome, Command, Description, Example, NamedCommandParser,
    ParseCommandError,
};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use serde::Deserialize;
use std::borrow::Cow;
use std::path::PathBuf;

/// Command that will load the simulation from a user-specified YAML file. Upon completion, the
/// simulation will be reset to the initial state, as per the loaded file, and the cursor
/// position reset to 0.
pub struct Load {
    path: String,
}

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Load
where
    for<'de> S: Clone + Deserialize<'de>,
{
    fn apply(
        &mut self,
        looper: &mut Looper<C, SimulationError<S>, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let path = PathBuf::from(&self.path);
        let decoder = looper.context().decoder();
        let scenario = yaml::read_from_file(decoder, path)
            .map_err(SimulationError::from)
            .map_err(ApplyCommandError::Application)?;
        looper.context().sim().set_scenario(scenario);
        looper
            .terminal()
            .print_line(&format!("Loaded scenario from '{}'.", self.path))?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Load`].
pub struct Parser;

impl<S, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser
where
    for<'de> S: Clone + Deserialize<'de>,
{
    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        if s.is_empty() {
            return Err(ParseCommandError("empty arguments to 'load'".into()));
        }
        let path = s.into();
        Ok(Box::new(Load { path }))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn name(&self) -> Cow<'static, str> {
        "load".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Loads a scenario from a file.".into(),
            usage: "<path>".into(),
            examples: vec![Example {
                scenario: "load from a file named 'trixie.yaml' in the working directory".into(),
                command: "trixie.yaml".into(),
            }],
        }
    }
}

#[cfg(test)]
mod tests;
