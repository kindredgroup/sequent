//! Saving of the current scenario to a YAML document.

use crate::commands::prompt::YesNo;
use crate::Context;
use sequent::persistence::yaml;
use sequent::SimulationError;
use revolver::command::{
    ApplyCommandError, ApplyOutcome, Command, Description, Example, NamedCommandParser,
    ParseCommandError,
};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use serde::ser::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

/// Command to save the scenario to a user-specified output file. If the file exists, a yes/no prompt
/// will be presented before overwriting it.
pub struct Save {
    path: String,
}

impl<S: Clone + Serialize, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Save {
    fn apply(
        &mut self,
        looper: &mut Looper<C, SimulationError<S>, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let path = PathBuf::from(&self.path);
        if path.exists() {
            let response = looper
                .terminal()
                .read_from_str_default("Output file exists. Overwrite? [y/N]: ")?;

            if let YesNo::No = response {
                return Ok(ApplyOutcome::Skipped);
            }
        }
        yaml::write_to_file(looper.context().sim().scenario(), path)
            .map_err(SimulationError::from)
            .map_err(ApplyCommandError::Application)?;

        looper
            .terminal()
            .print_line(&format!("Saved scenario to '{}'.", self.path))?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Save`].
pub struct Parser;

impl<S: Clone + Serialize, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T>
    for Parser
{
    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        if s.is_empty() {
            return Err(ParseCommandError("empty arguments to 'save'".into()));
        }
        let path = s.into();
        Ok(Box::new(Save { path }))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn name(&self) -> Cow<'static, str> {
        "save".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Saves the current scenario to a file.".into(),
            usage: "<path>".into(),
            examples: vec![Example {
                scenario: "save to a file named 'trixie.yaml' in the working directory".into(),
                command: "trixie.yaml".into(),
            }],
        }
    }
}

#[cfg(test)]
mod tests;
