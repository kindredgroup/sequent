//! Aspects of the simulation relating to (discrete) events.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

/// A mutable view over the event timeline. The queue
/// is notionally subdivided into _past_, _current_ and _future_ events. The past events
/// are those that have already been executed. The current event is the one at which the
/// cursor is resting; the same event that is passed to [`Event::apply()`]. The future
/// events comprise the sequence that follows the current event.
pub struct Queue<'a, S> {
    offset: usize,
    timeline: &'a Vec<Box<dyn Event<S>>>,
    insertions: Vec<(usize, Box<dyn Event<S>>)>
}

impl<'a, S> Queue<'a, S> {
    /// Creates a new queue over an existing timeline. The `offset` parameter is the index of the
    /// first future event. (I.e., the immediate successor of the event at the cursor.)
    ///
    /// # Panics
    /// If the offset is less than 1 or exceeds the length of the timeline.
    pub fn new(offset: usize, timeline: &'a Vec<Box<dyn Event<S>>>) -> Self {
        assert!(offset >= 1, "offset ({offset}) cannot be less than 1");
        assert!(offset <= timeline.len(), "offset ({offset}) cannot exceed length of timeline {}", timeline.len());
        Self {
            offset,
            timeline,
            insertions: Vec::default()
        }
    }

    /// Insert an event into the specified location in the queue. The effect on the queue
    /// (and the underlying event timeline) will not persist until after [`Event::apply()`]
    /// returns. Equivalently, the [`Queue::future()`] view will not change after calling this method.
    ///
    /// # Panics
    /// If the insertion index exceeds the length of the timeline.
    pub fn insert_later(&mut self, index: usize, event: Box<dyn Event<S>>) {
        let lim = self.timeline.len() + self.insertions.len() - self.offset;
        assert!(index <= lim, "insertion index ({index}) cannot exceed length of queue ({lim})");
        self.insertions.push((index, event));
    }

    /// Push an event onto the end of the queue. The effect on the queue
    /// (and the underlying event timeline) will not persist until after [`Event::apply()`]
    /// returns. Equivalently, the [`Queue::future()`] view will not change after calling this method.
    pub fn push_later(&mut self, event: Box<dyn Event<S>>) {
        self.insert_later(self.timeline.len() + self.insertions.len() - self.offset, event);
    }

    /// A slice of past (already executed) events. This is an immutable view.
    pub fn past(&self) -> &[Box<dyn Event<S>>] {
        &self.timeline[..self.offset - 1]
    }

    /// A slice of future events, excluding the current. This is an immutable view; it does not include
    /// events added via [`Queue::insert_later()`] or [`Queue::push_later()`].
    pub fn future(&self) -> &[Box<dyn Event<S>>] {
        &self.timeline[self.offset..]
    }

    /// Consumes this queue, returning its constituents (`offset`, `timeline`, `insertions`).
    #[allow(clippy::type_complexity)]
    pub fn into_inner(self) -> (usize, &'a Vec<Box<dyn Event<S>>>, Vec<(usize, Box<dyn Event<S>>)>) {
        (self.offset, self.timeline, self.insertions)
    }
}

pub(crate) fn process_insertions<S>(offset: usize, insertions: Vec<(usize, Box<dyn Event<S>>)>, timeline: &mut Vec<Box<dyn Event<S>>>) {
    for (index, event) in insertions {
        timeline.insert(offset + index, event);
    }
}

/// Dereferencing a [`Queue`] is equivalent to [`Queue::future()`].
impl<S> Deref for Queue<'_, S> {
    type Target = [Box<dyn Event<S>>];

    fn deref(&self) -> &Self::Target {
        self.future()
    }
}

impl<S> Debug for Queue<'_, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

/// Something that has a constant name (i.e., independent of `Self`).
pub trait StaticNamed {
    /// The static name of this type.
    fn name() -> &'static str;
}

/// An entity that has a string-like name.
pub trait Named {
    /// The name of this entity.
    fn name(&self) -> Cow<'static, str>;
}

/// Acquired implementation of [`Named`] for any type that implements [`StaticNamed`].
impl<N: StaticNamed> Named for N {
    fn name(&self) -> Cow<'static, str> {
        <N as StaticNamed>::name().into()
    }
}

/// Specification of a discrete event.
pub trait Event<S>: Named + Debug + ToString {
    /// Evaluates the event in the course of a simulation, applying it to the current state to
    /// produce the next state (by mutating the `state` reference in-place). The event may
    /// also insert or append new events to the pending queue. (Changes to the queue, if any, will
    /// only be persisted after [`Event::apply()`] returns.)
    ///
    /// # Errors
    /// [`TransitionError`] if the event could not be evaluated.
    fn apply(&self, state: &mut S, queue: &mut Queue<S>) -> Result<(), TransitionError>;
}

/// Produced by [`Event::apply()`] if an error occurs.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct TransitionError(pub Cow<'static, str>);

/// A complete simulation scenario, comprising the initial state and a timeline of discrete
/// events.
#[derive(Debug)]
pub struct Scenario<S> {
    /// The initial simulation state.
    pub initial: S,

    /// Timeline of discrete [`Event`] objects.
    pub timeline: Vec<Box<dyn Event<S>>>,
}

impl<S: Default> Default for Scenario<S> {
    fn default() -> Self {
        Self {
            initial: S::default(),
            timeline: Vec::default(),
        }
    }
}

/// Produced if an [`Event`] object could not be decoded from its string equivalent.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct ParseEventError(pub Cow<'static, str>);

/// A parser for [`Event`] types.
pub trait NamedEventParser<S>: Named {
    /// Constructs an [`Event`] object from its string representation.
    ///
    /// # Errors
    /// [`ParseEventError`] if the given string slice could not be decoded.
    fn parse(&self, s: &str) -> Result<Box<dyn Event<S>>, ParseEventError>;
}

/// Decodes a name-value tuple into an [`Event`] object using a preconfigured map of
/// parsers.
pub struct Decoder<S> {
    by_name: BTreeMap<String, Box<dyn NamedEventParser<S>>>,
}

impl<S> Decoder<S> {
    /// Creates a new decoder from the given vector of parsers.
    ///
    /// # Panics
    /// If there was an error building a [`Decoder`] from the given parsers.
    pub fn new(parsers: Vec<Box<dyn NamedEventParser<S>>>) -> Self {
        parsers.try_into().unwrap()
    }

    /// An iterator over the underlying parsers.
    pub fn parsers(&self) -> impl Iterator<Item = &Box<dyn NamedEventParser<S>>> {
        self.by_name.values()
    }

    /// Decodes a given `encoded` representation for an event of a given `name` into a
    /// [`Event`] object.
    ///
    /// # Errors
    /// [`ParseEventError`] if an event could not be decoded from the given `name` and `encoded` pair.
    pub fn decode(&self, name: &str, encoded: &str) -> Result<Box<dyn Event<S>>, ParseEventError> {
        let parser = self
            .by_name
            .get(name)
            .ok_or_else(|| ParseEventError(format!("no event parser for '{name}'").into()))?;
        parser.parse(encoded)
    }
}

/// Raised by [`Decoder`] if there was something wrong with the parsers given to it. Perhaps
/// the parsers were incorrectly specified or conflicted amongst themselves.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct InvalidEventParserSpec(String);

impl<S> TryFrom<Vec<Box<dyn NamedEventParser<S>>>> for Decoder<S> {
    type Error = InvalidEventParserSpec;

    fn try_from(parsers: Vec<Box<dyn NamedEventParser<S>>>) -> Result<Self, Self::Error> {
        let mut by_name = BTreeMap::default();
        for parser in parsers {
            let name = parser.name();
            if by_name.insert(name.to_string(), parser).is_some() {
                return Err(InvalidEventParserSpec(format!(
                    "duplicate event parser for '{}'",
                    name
                )));
            }
        }

        Ok(Self { by_name })
    }
}

/// A generic parser for any event type.
pub struct Parser<E>(PhantomData<E>);

impl<E> Default for Parser<E> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

/// Acquired implementation of [`StaticNamed`] for any [`Parser`] that is parametrised with an
/// [`Event`] type that is also [`StaticNamed`].
impl<E> StaticNamed for Parser<E>
where
    E: StaticNamed,
{
    fn name() -> &'static str {
        <E as StaticNamed>::name()
    }
}

/// Blanket [`NamedEventParser`] implementation for any compliant [`Parser`].
impl<E, S> NamedEventParser<S> for Parser<E>
where
    E: StaticNamed + FromStr + Event<S> + 'static,
    ParseEventError: From<<E as FromStr>::Err>,
{
    fn parse(&self, s: &str) -> Result<Box<dyn Event<S>>, ParseEventError> {
        Ok(Box::new(E::from_str(s)?))
    }
}

#[cfg(test)]
mod tests;
