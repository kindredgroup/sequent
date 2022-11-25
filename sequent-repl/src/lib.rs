//! An adapter for using Sequent with Revolver.

pub mod commands;

use sequent::{Decoder, Simulation};
use revolver::terminal::{AccessTerminalError, Terminal};

/// Specification of a minimal application context for simulations.
pub trait Context<S> {
    /// A mutable reference to the simulation.
    fn sim(&mut self) -> &mut Simulation<S>;

    /// Prints the current state to the given terminal interface.
    ///
    /// # Errors
    /// [`AccessTerminalError`] if an error occurs while writing to the terminal.
    fn print_state(&self, terminal: &mut impl Terminal) -> Result<(), AccessTerminalError>;

    /// A reference to a decoder for parsing events.
    fn decoder(&self) -> &Decoder<S>;
}
