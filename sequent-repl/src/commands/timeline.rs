//! Printing of the event timeline.

use std::borrow::Cow;
use stanza::renderer::console::{Console, Decor};
use stanza::renderer::Renderer;
use stanza::style::{Bold, HAlign, MinWidth, Palette16, Styles, TextFg};
use stanza::table::{Col, Row, Table};
use sequent::{Simulation, SimulationError};
use revolver::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use revolver::looper::Looper;
use revolver::terminal::Terminal;
use crate::Context;

/// Command to print the event timeline as a table to the terminal device.
pub struct Timeline;

impl<S, C: Context<S>, T: Terminal> Command<C, SimulationError<S>, T> for Timeline {
    fn apply(&mut self, looper: &mut Looper<C, SimulationError<S>, T>) -> Result<ApplyOutcome, ApplyCommandError<SimulationError<S>>> {
        let (terminal, _, context) = looper.split();
        let table = timeline(context.sim());
        let renderer = Console(
            Decor::default()
                .suppress_all_lines()
                .suppress_outer_border(),
        );
        terminal.print_line(&renderer.render(&table))?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Timeline`].
pub struct Parser;

impl<S, C: Context<S>, T: Terminal> NamedCommandParser<C, SimulationError<S>, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, SimulationError<S>, T>>, ParseCommandError> {
        self.parse_no_args(s, || Timeline)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("tl".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "timeline".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Displays the timeline of events.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

fn timeline<S>(simulation: &Simulation<S>) -> Table {
    const CURSOR: &str = "▶";

    let mut table = Table::default()
        .with_cols(vec![
            Col::new(Styles::default()),
            Col::new(Styles::default().with(HAlign::Right)),
            Col::new(Styles::default().with(MinWidth(15))),
            Col::new(Styles::default().with(MinWidth(40))),
        ])
        .with_row(Row::new(
            Styles::default()
                .with(Bold(true))
                .with(TextFg(Palette16::Yellow)),
            vec![
                "".into(),
                "".into(),
                "Event name".into(),
                "Encoded event arguments".into(),
            ],
        ));

    for (idx, event) in simulation.scenario().timeline.iter().enumerate() {
        let on_cursor = idx == simulation.cursor();
        table.push_row(Row::new(
            Styles::default().with(Bold(on_cursor)),
            vec![
                if on_cursor {
                    CURSOR
                } else {
                    ""
                }
                .into(),
                idx.into(),
                event.name().into(),
                event.to_string().into(),
            ],
        ));
    }

    if simulation.cursor() == simulation.scenario().timeline.len() {
        table.push_row(Row::new(Styles::default().with(Bold(true)), vec![
            CURSOR.into(),
            simulation.scenario().timeline.len().into(),
        ]));
    }

    table
}

#[cfg(test)]
mod tests;