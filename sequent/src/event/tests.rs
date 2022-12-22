// $coverage:ignore-start

use super::*;
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::Range;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct TestState;

#[derive(Debug)]
struct SampleEvent;

impl Named for SampleEvent {
    fn name(&self) -> Cow<'static, str> {
        "test-event".into()
    }
}

impl ToString for SampleEvent {
    fn to_string(&self) -> String {
        "".into()
    }
}

impl Event for SampleEvent {
    type State = TestState;

    fn apply(&self, _: &mut TestState, _: &mut Queue<Self::State>) -> Result<(), TransitionError> {
        Ok(())
    }
}

impl FromStr for SampleEvent {
    type Err = ParseEventError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self)
        } else {
            Err(ParseEventError(
                format!("invalid arguments to 'sample': '{s}'").into(),
            ))
        }
    }
}

struct Parser {
    name: &'static str,
}

impl Named for Parser {
    fn name(&self) -> Cow<'static, str> {
        self.name.into()
    }
}

impl NamedEventParser for Parser {
    type State = TestState;

    fn parse(&self, s: &str) -> Result<Box<dyn Event<State = Self::State>>, ParseEventError> {
        Ok(Box::new(SampleEvent::from_str(s)?))
    }
}

#[test]
fn invalid_event_parser_spec_implements_display() {
    let s = format!("{}", InvalidEventParserSpec("foo".into()));
    assert_eq!("foo", s);
}

#[test]
fn parse_event_error_implements_display() {
    let s = format!("{}", ParseEventError("foo".into()));
    assert_eq!("foo", s);
}

#[test]
fn decoder() {
    let parsers: Vec<Box<dyn NamedEventParser<State = _>>> = vec![Box::new(Parser { name: "sample" })];
    let decoder = Decoder::new(parsers);
    assert_eq!(1, decoder.parsers().count());
    assert_eq!(None, decoder.decode("sample", "").err());
    assert_eq!(
        Some(ParseEventError("no event parser for 'x'".into())),
        decoder.decode("x", "").err()
    );
    assert_eq!(
        Some(ParseEventError("invalid arguments to 'sample': 'z'".into())),
        decoder.decode("sample", "z").err()
    );
}

#[test]
fn decoder_duplicate() {
    let parsers: Vec<Box<dyn NamedEventParser<State = _>>> = vec![
        Box::new(Parser { name: "gg" }),
        Box::new(Parser { name: "gg" }),
    ];
    assert_eq!(
        Some(InvalidEventParserSpec(
            "duplicate event parser for 'gg'".into()
        )),
        Decoder::try_from(parsers).err()
    );
}

#[test]
fn scenario_implements_debug() {
    let scenario = Scenario::<()>::default();
    let s = format!("{scenario:?}");
    assert!(s.contains("Scenario"));
}

#[derive(Debug)]
struct IndexedEvent(usize);

impl StaticNamed for IndexedEvent {
    fn name() -> &'static str {
        "indexed".into()
    }
}

impl ToString for IndexedEvent {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Event for IndexedEvent {
    type State = TestState;

    fn apply(&self, _: &mut TestState, _: &mut Queue<Self::State>) -> Result<(), TransitionError> {
        unimplemented!();
    }
}

fn indexed_events(range: Range<usize>) -> Vec<Box<dyn Event<State = TestState>>> {
    range
        .into_iter()
        .map(|idx| Box::new(IndexedEvent(idx)) as Box<dyn Event<State = TestState>>)
        .collect()
}

fn indexes(events: &[Box<dyn Event<State = TestState>>]) -> Vec<usize> {
    events.iter().map(|event| usize::from_str(&event.to_string()).unwrap()).collect()
}

#[test]
fn queue_past() {
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(1, &timeline);
        assert_eq!(vec![] as Vec<usize>, indexes(queue.past()));
    }
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(2, &timeline);
        assert_eq!(vec![0], indexes(queue.past()));
    }
}

#[test]
fn queue_future() {
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(1, &timeline);
        assert_eq!(vec![1], indexes(queue.future()));
    }
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(2, &timeline);
        assert_eq!(vec![] as Vec<usize>, indexes(queue.future()));
    }
}

#[test]
fn queue_implements_deref() {
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(1, &timeline);
        assert_eq!(vec![1], indexes(&*queue));
    }
    {
        let timeline = indexed_events(0..2);
        let queue = Queue::new(2, &timeline);
        assert_eq!(vec![] as Vec<usize>, indexes(&*queue));
    }
}

#[test]
fn queue_implements_debug() {
    let timeline = indexed_events(0..3);
    let queue = Queue::new(1, &timeline);
    let s = format!("{queue:?}");
    assert!(s.contains("[IndexedEvent(1), IndexedEvent(2)]"), "s={s}");
}

#[test]
fn queue_insert_later() {
    {
        let mut timeline = indexed_events(0..3);
        let mut queue = Queue::new(1, &timeline);
        queue.insert_later(0, Box::new(IndexedEvent(10)));
        process_insertions(queue.offset, queue.insertions, &mut timeline);
        assert_eq!(vec![0, 10, 1, 2], indexes(&timeline));
    }
    {
        let mut timeline = indexed_events(0..3);
        let mut queue = Queue::new(1, &timeline);
        queue.insert_later(1, Box::new(IndexedEvent(10)));
        process_insertions(queue.offset, queue.insertions, &mut timeline);
        assert_eq!(vec![0, 1, 10, 2], indexes(&timeline));
    }
    {
        let mut timeline = indexed_events(0..3);
        let mut queue = Queue::new(1, &timeline);
        queue.insert_later(2, Box::new(IndexedEvent(10)));
        queue.insert_later(2, Box::new(IndexedEvent(20)));
        queue.insert_later(2, Box::new(IndexedEvent(30)));
        process_insertions(queue.offset, queue.insertions, &mut timeline);
        assert_eq!(vec![0, 1, 2, 30, 20, 10], indexes(&timeline));
    }
}

#[test]
#[should_panic(expected = "insertion index (3) cannot exceed length of queue (2)")]
fn queue_insert_later_bad_index() {
    let timeline = indexed_events(0..3);
    let mut queue = Queue::new(1, &timeline);
    queue.insert_later(3, Box::new(IndexedEvent(10)));
}

#[test]
fn queue_push_later() {
    let mut timeline = indexed_events(0..3);
    let mut queue = Queue::new(1, &timeline);
    queue.push_later(Box::new(IndexedEvent(10)));
    queue.push_later(Box::new(IndexedEvent(20)));
    queue.push_later(Box::new(IndexedEvent(30)));
    process_insertions(queue.offset, queue.insertions, &mut timeline);
    assert_eq!(vec![0, 1, 2, 10, 20, 30], indexes(&timeline));
}

#[test]
#[should_panic(expected = "offset (0) cannot be less than 1")]
fn queue_construct_offset_less_than_one() {
    let timeline = indexed_events(0..3);
    Queue::new(0, &timeline);
}

#[test]
#[should_panic(expected = "offset (4) cannot exceed length of timeline 3")]
fn queue_construct_offset_exceeds_length() {
    let timeline = indexed_events(0..3);
    Queue::new(4, &timeline);
}
