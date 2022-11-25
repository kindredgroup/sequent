// $coverage:ignore-start

use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;
use crate::ParseEventError;
use crate::persistence::{check_ext, ReadScenarioError, UnsupportedFileFormatError, WriteScenarioError};

#[test]
fn check_ext_passes() {
    assert_eq!(Ok(()), check_ext(&PathBuf::from("data.txt"), "txt"));
}

#[test]
fn check_ext_fails() {
    assert_eq!(Err(UnsupportedFileFormatError("expected file extension 'txt', got 'log'".into())), check_ext(&PathBuf::from("data.log"), "txt"));
}

#[test]
fn unsupported_file_format_error_implements_display() {
    let err = UnsupportedFileFormatError("expected file extension 'txt', got 'log'".into());
    assert_eq!("expected file extension 'txt', got 'log'", err.to_string());
}

fn write_scenario_error_io() -> WriteScenarioError {
   io::Error::new(ErrorKind::BrokenPipe, "broken pipe").into()
}

fn write_scenario_error_unsupported_file_format() -> WriteScenarioError {
    UnsupportedFileFormatError("data".into()).into()
}

#[test]
fn write_scenario_error_implements_display() {
    assert_eq!("io: broken pipe", write_scenario_error_io().to_string());
    assert_eq!("unsupported file format: data", write_scenario_error_unsupported_file_format().to_string());
}

#[test]
fn write_scenario_error_variants() {
    assert!(write_scenario_error_io().io().is_some());
    assert!(write_scenario_error_io().unsupported_file_format().is_none());

    assert!(write_scenario_error_unsupported_file_format().unsupported_file_format().is_some());
    assert!(write_scenario_error_unsupported_file_format().io().is_none());
}

fn read_scenario_error_io() -> ReadScenarioError {
    io::Error::new(ErrorKind::BrokenPipe, "broken pipe").into()
}

fn read_scenario_error_unsupported_file_format() -> ReadScenarioError {
    UnsupportedFileFormatError("data".into()).into()
}

fn read_scenario_error_parse_event() -> ReadScenarioError {
    ParseEventError("data".into()).into()
}

fn read_scenario_error_deserializer() -> ReadScenarioError {
    let err: Box<dyn Error> = "data".into();
    err.into()
}

#[test]
fn read_scenario_error_implements_display() {
    assert_eq!("io: broken pipe", read_scenario_error_io().to_string());
    assert_eq!("unsupported file format: data", read_scenario_error_unsupported_file_format().to_string());
    assert_eq!("parse event: data", read_scenario_error_parse_event().to_string());
    assert_eq!("deserializer: data", read_scenario_error_deserializer().to_string());
}

#[test]
fn read_scenario_error_variants() {
    assert!(read_scenario_error_io().io().is_some());
    assert!(read_scenario_error_io().unsupported_file_format().is_none());

    assert!(read_scenario_error_unsupported_file_format().unsupported_file_format().is_some());
    assert!(read_scenario_error_unsupported_file_format().parse_event().is_none());

    assert!(read_scenario_error_parse_event().parse_event().is_some());
    assert!(read_scenario_error_parse_event().deserializer().is_none());

    assert!(read_scenario_error_deserializer().deserializer().is_some());
    assert!(read_scenario_error_deserializer().io().is_none());
}