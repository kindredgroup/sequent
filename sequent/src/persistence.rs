//! Persistence of a scenario.

pub mod yaml;

use crate::{Decoder, ParseEventError, Scenario};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug};
use std::io;
use std::io::{BufRead, Write};
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// A DTO for shuttling a scenario in a persistence-friendly form. Here, the timeline is replaced
/// with a vector of [`PersistentEvent`]s, which are encoded versions of the [`Event`](crate::Event) objects.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentScenario<S> {
    /// Initial simulation state.
    pub initial: S,

    /// Timeline of encoded [`PersistentEvent`]s.
    pub timeline: Vec<PersistentEvent>,
}

/// A persistence-friendly representation of an [`Event`](crate::Event).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentEvent {
    /// The name of the event. Taken from [`crate::Named::name`].
    pub name: String,

    /// An encoded form. Taken from [`ToString::to_string`].
    pub encoded: String,
}

/// Creates a [`PersistentScenario`] from a [`Scenario`] reference.
impl<S: Clone> From<&Scenario<S>> for PersistentScenario<S> {
    fn from(scenario: &Scenario<S>) -> Self {
        Self {
            initial: scenario.initial.clone(),
            timeline: scenario
                .timeline
                .iter()
                .map(|event| PersistentEvent {
                    name: event.name().into(),
                    encoded: event.to_string(),
                })
                .collect(),
        }
    }
}

impl<S> PersistentScenario<S> {
    /// Decodes a [`PersistentScenario`] into a [`Scenario`] instance, using the supplied `decoder`.
    /// This will iterate over all [`PersistentEvent`]s, converting them to their [`Event`](crate::Event) equivalents.
    ///
    /// # Errors
    /// [`ParseEventError`] if the event could not be decoded.
    pub fn decode(self, decoder: &Decoder<S>) -> Result<Scenario<S>, ParseEventError> {
        let mut timeline = Vec::with_capacity(self.timeline.len());
        for event in self.timeline {
            let event = decoder.decode(&event.name, &event.encoded)?;
            timeline.push(event);
        }

        Ok(Scenario {
            initial: self.initial,
            timeline,
        })
    }
}

/// Unwraps a container type into its inner value, consuming the container in the process.
trait IntoInner<T> {
    /// Obtains the inner value.
    fn into_inner(self) -> T;
}

/// Produced when reading from or writing to a file when the format does not match the requirements
/// of persistence.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct UnsupportedFileFormatError(String);

/// Produced when the scenario could not be saved to an output stream or a file. Encompasses all
/// possible error variants, some of which may not apply in all persistence scenarios.
#[derive(Debug, Error)]
pub enum WriteScenarioError {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("unsupported file format: {0}")]
    UnsupportedFileFormat(#[from] UnsupportedFileFormatError),
}

/// Error variant conversions.
impl WriteScenarioError {
    /// Converts the error into an [`Option<io::Error>`].
    pub fn io(self) -> Option<io::Error> {
        match self {
            WriteScenarioError::Io(err) => Some(err),
            WriteScenarioError::UnsupportedFileFormat(_) => None
        }
    }

    /// Converts the error into an [`Option<UnsupportedFileFormatError>`].
    pub fn unsupported_file_format(self) -> Option<UnsupportedFileFormatError> {
        match self {
            WriteScenarioError::Io(_) => None,
            WriteScenarioError::UnsupportedFileFormat(err) => Some(err)
        }
    }
}

/// Produced when a scenario could not be read from an input stream or a file. Encompasses all
// possible error variants, some of which may not apply in all persistence scenarios.
#[derive(Debug, Error)]
pub enum ReadScenarioError {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("unsupported file format: {0}")]
    UnsupportedFileFormat(#[from] UnsupportedFileFormatError),

    #[error("parse event: {0}")]
    ParseEvent(#[from] ParseEventError),

    #[error("deserializer: {0}")]
    Deserializer(#[from] Box<dyn Error>),
}

/// Error variant conversions.
impl ReadScenarioError {
    /// Converts the error into an [`Option<io::Error>`].
    pub fn io(self) -> Option<io::Error> {
        match self {
            ReadScenarioError::Io(err) => Some(err),
            _ => None
        }
    }

    /// Converts the error into an [`Option<UnsupportedFileFormatError>`].
    pub fn unsupported_file_format(self) -> Option<UnsupportedFileFormatError> {
        match self {
            ReadScenarioError::UnsupportedFileFormat(err) => Some(err),
            _ => None
        }
    }

    /// Converts the error into an [`Option<ParseEventError>`].
    pub fn parse_event(self) -> Option<ParseEventError> {
        match self {
            ReadScenarioError::ParseEvent(err) => Some(err),
            _ => None
        }
    }

    /// Converts the error into an [`Option<Box<dyn Error>>`].
    pub fn deserializer(self) -> Option<Box<dyn Error>> {
        match self {
            ReadScenarioError::Deserializer(err) => Some(err),
            _ => None
        }
    }
}

fn check_ext(path: &Path, expected: &str) -> Result<(), UnsupportedFileFormatError> {
    let ext = path
        .extension()
        .map(|ext| ext.to_str().unwrap_or_default())
        .unwrap_or_default();
    if ext == expected {
        Ok(())
    } else {
        Err(UnsupportedFileFormatError(format!(
            "expected file extension '{expected}', got '{ext}'"
        )))
    }
}

fn write<C, S>(scenario: &Scenario<S>, w: &mut impl Write) -> Result<(), WriteScenarioError>
where
    S: Clone + Serialize,
    C: From<PersistentScenario<S>> + ToString,
{
    let persistent = PersistentScenario::from(scenario);
    let data = C::from(persistent).to_string();
    w.write_all(data.as_bytes())?;
    Ok(())
}

fn read<C, CE, S>(
    decoder: &Decoder<S>,
    r: &mut impl BufRead,
) -> Result<Scenario<S>, ReadScenarioError>
where
    for<'de> S: Deserialize<'de>,
    CE: Error + 'static,
    C: FromStr<Err = CE> + IntoInner<PersistentScenario<S>>,
{
    let mut buf = String::default();
    r.read_to_string(&mut buf)?;
    let carrier = C::from_str(&buf).map_err(|err| Box::new(err) as Box<dyn Error>)?;
    let persistent = carrier.into_inner();
    Ok(persistent.decode(decoder)?)
}

#[cfg(test)]
mod tests;