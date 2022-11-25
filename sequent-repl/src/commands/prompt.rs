//! Types for prompting the user over a terminal interface.

use std::str::FromStr;

/// A yes/no prompt. Defaults to 'no'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YesNo {
    Yes, No
}

impl Default for YesNo {
    fn default() -> Self {
        Self::No
    }
}

impl FromStr for YesNo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "y" | "yes" => Ok(Self::Yes),
            "n" | "no" => Ok(Self::No),
            other => Err(format!("'{other}' is not a yes/no"))
        }
    }
}

#[cfg(test)]
mod tests;