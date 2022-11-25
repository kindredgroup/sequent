//! Persistence extensions for working with YAML files.

use crate::persistence::{
    check_ext, read, write, IntoInner, PersistentScenario, ReadScenarioError, WriteScenarioError,
};
use crate::{Decoder, Scenario};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

const EXT: &str = "yaml";

/// A container for de/serializing arbitrary types from/into YAML.
pub struct Carrier<T>(T);

impl<T: PartialEq> PartialEq for Carrier<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Debug> Debug for Carrier<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Carrier({:?})", self.0)
    }
}

impl<T> From<T> for Carrier<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Serializes the content of a [`Carrier`] to its YAML representation.
impl<T: Serialize> ToString for Carrier<T> {
    fn to_string(&self) -> String {
        serde_yaml::to_string(&self.0).unwrap()
    }
}

/// Populates the contents of a new [`Carrier`] from a YAML string.
impl<T> FromStr for Carrier<T>
where
    for<'a> T: Deserialize<'a>,
{
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: T = serde_yaml::from_str(s)?;
        Ok(Self(value))
    }
}

impl<T> IntoInner<T> for Carrier<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

/// Reads and decodes a scenario from a given YAML file.
///
/// # Errors
/// [`ReadScenarioError`] if the scenario could not be read.
pub fn read_from_file<S>(
    decoder: &Decoder<S>,
    path: impl AsRef<Path>,
) -> Result<Scenario<S>, ReadScenarioError>
where
    for<'de> S: Deserialize<'de>,
{
    check_ext(path.as_ref(), EXT)?;
    let mut r = BufReader::new(File::open(&path)?);
    read::<Carrier<PersistentScenario<S>>, _, _>(decoder, &mut r)
}

/// Writs a scenario to a YAML file.
///
/// # Errors
/// [`WriteScenarioError`] if the scenario could not be written.
pub fn write_to_file<S: Clone + Serialize>(
    scenario: &Scenario<S>,
    path: impl AsRef<Path>,
) -> Result<(), WriteScenarioError> {
    check_ext(path.as_ref(), EXT)?;
    let mut w = BufWriter::new(File::create(&path)?);
    write::<Carrier<PersistentScenario<S>>, _>(scenario, &mut w)?;
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests;
