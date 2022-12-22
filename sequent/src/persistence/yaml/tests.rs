// $coverage:ignore-start

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use flanker_assert_str::assert_loopback;
use flanker_temp::TempPath;
use crate::{Decoder, Event, ParseEventError, Parser, Queue, Scenario, StaticNamed, TransitionError};
use serde::{Deserialize, Serialize};
use crate::persistence::{PersistentEvent, PersistentScenario};
use crate::persistence::yaml::{Carrier, read_from_file, write_to_file};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestState {
    some_string: String,
    some_f64: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct TestEvent(Vec<String>);

impl StaticNamed for TestEvent {
    fn name() -> &'static str {
        "test"
    }
}

impl ToString for TestEvent {
    fn to_string(&self) -> String {
        self.0.join(" ")
    }
}

impl Event for TestEvent {
    type State = TestState;

    fn apply(&self, _: &mut Self::State, _: &mut Queue<Self::State>) -> Result<(), TransitionError> {
        unimplemented!()
    }
}

impl FromStr for TestEvent {
    type Err = ParseEventError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split_whitespace().map(ToString::to_string).collect()))
    }
}

fn persistent_scenario_fixture() -> PersistentScenario<TestState> {
    PersistentScenario {
        initial: TestState {
            some_string: "hello".to_string(),
            some_f64: 3.14,
        },
        timeline: vec![PersistentEvent {
            name: "test".into(),
            encoded: "a b c".into(),
        }],
    }
}

fn scenario_fixture() -> Scenario<TestState> {
    Scenario {
        initial: TestState {
            some_string: "hello".to_string(),
            some_f64: 3.14,
        },
        timeline: vec![
            Box::new(TestEvent(vec!["a".into(), "b".into(), "c".into()]))
        ]
    }
}

#[test]
fn scenario_round_trip() {
    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    let decoded = persistent_scenario_fixture().decode(&decoder).unwrap();
    let s = scenario_fixture();
    assert_eq!(s.initial, decoded.initial);
    assert_eq!(s.timeline.len(), decoded.timeline.len());

    let ps = PersistentScenario::from(&decoded);
    assert_eq!(ps, persistent_scenario_fixture());
}

#[test]
fn to_string() {
    let scenario = persistent_scenario_fixture();

    assert_eq!(
        "\
initial:
  some_string: hello
  some_f64: 3.14
timeline:
- name: test
  encoded: a b c
",
        Carrier::from(scenario).to_string()
    );
}

#[test]
fn str_loopback() {
    assert_loopback(&Carrier::from(persistent_scenario_fixture()));
}

#[test]
fn carrier_implements_partial_eq() {
    assert_eq!(persistent_scenario_fixture(), persistent_scenario_fixture());
}

#[test]
fn carrier_implements_debug() {
    let s = format!("{:?}", Carrier::from(persistent_scenario_fixture()));
    assert!(s.contains("Carrier"))
}

#[test]
#[should_panic(expected = "UnsupportedFileFormat(UnsupportedFileFormatError(\"expected file extension 'yaml', got 'json'\"))")]
fn read_from_file_invalid_format() {
    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    read_from_file(&decoder, PathBuf::from("data.json")).unwrap();
}

#[test]
#[should_panic(expected = "Io(Os { code: 2, kind: NotFound, message: \"No such file or directory\" })")]
fn read_from_file_nonexistent_file() {
    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    read_from_file(&decoder, PathBuf::from("nonexistent.yaml")).unwrap();
}

#[test]
#[should_panic(expected = "Deserializer(Error(\"invalid type: string \\\"not a valid scenario\\\", expected struct PersistentScenario\", line: 1, column: 1))")]
fn read_from_file_invalid_content() {
    let temp = TempPath::with_extension("yaml");
    let bad_yaml = "not a valid scenario";
    fs::write(&temp, bad_yaml).unwrap();

    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    read_from_file(&decoder, &temp).unwrap();
}

#[test]
#[should_panic(expected = "ParseEvent(ParseEventError(\"no event parser for 'test'\"))")]
fn read_from_file_misconfigured_decoder() {
    let temp = TempPath::with_extension("yaml");
    write_to_file(&scenario_fixture(), &temp).unwrap();

    let decoder = Decoder::new(vec![]);
    let original = read_from_file(&decoder, &temp).unwrap();
    let ps = persistent_scenario_fixture();
    assert_eq!(ps, PersistentScenario::from(&original));
}

#[test]
#[should_panic(expected = "UnsupportedFileFormat(UnsupportedFileFormatError(\"expected file extension 'yaml', got 'json'\"))")]
fn write_to_file_invalid_format() {
    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    let decoded = persistent_scenario_fixture().decode(&decoder).unwrap();
    write_to_file(&decoded, PathBuf::from("data.json")).unwrap();
}

#[test]
#[should_panic(expected = "Io(Os { code: 2, kind: NotFound, message: \"No such file or directory\" })")]
fn write_to_file_nonexistent_directory() {
    write_to_file(&scenario_fixture(), PathBuf::from("nonexistent_dir/data.yaml")).unwrap();
}

#[test]
fn write_then_read() {
    let ps = persistent_scenario_fixture();
    let decoder = Decoder::new(vec![Box::new(Parser::<TestEvent>::default())]);
    let decoded = ps.decode(&decoder).unwrap();

    let temp = TempPath::with_extension("yaml");
    write_to_file(&decoded, &temp).unwrap();

    let original = read_from_file(&decoder, &temp).unwrap();
    let ps = persistent_scenario_fixture();
    assert_eq!(ps, PersistentScenario::from(&original));
}