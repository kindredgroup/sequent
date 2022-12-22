// $coverage:ignore-start

use crate::commands::save::{Parser, Save};
use crate::commands::test_fixtures::{read_str_from_file, write_str_to_file, TestContext, TestState};
use sequent::SimulationError;
use flanker_temp::TempPath;
use revolver::command::{assert_pedantic, ApplyOutcome, Command, Commander, NamedCommandParser};
use revolver::looper::Looper;
use revolver::terminal::{lines, AccessTerminalError, Mock, PrintOutput};
use std::fs;

fn command_parsers<'d>(
) -> Vec<Box<dyn NamedCommandParser<Mock<'d>, Context = TestContext, Error = SimulationError<TestState>>>> {
    vec![Box::new(Parser::default())]
}

#[test]
fn apply_new_file() {
    let temp = TempPath::with_extension("yaml");
    let path = temp.as_ref().to_string_lossy().to_string();
    let mut term = Mock::default();
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut save = Save::new(path.clone());
    assert_eq!(ApplyOutcome::Applied, save.apply(&mut looper).unwrap());
    assert_eq!(
        format!("Saved scenario to '{}'.\n", path),
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    drop(temp);
}

#[test]
fn apply_existing_file_is_directory_io_error() {
    let temp = TempPath::with_extension("yaml");
    let path = temp.as_ref().to_string_lossy().to_string();
    fs::create_dir(&temp).unwrap();

    let mut term = Mock::default().on_read_line(lines(&["yes"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut save = Save::new(path.clone());
    assert_eq!(
        "is a directory",
        save.apply(&mut looper)
            .unwrap_err()
            .application()
            .unwrap()
            .write_scenario()
            .unwrap()
            .io()
            .unwrap()
            .kind()
            .to_string()
    );
    drop(temp);
}

#[test]
fn apply_existing_file_overwrite() {
    const DUMMY_DATA: &str = "dummy data";
    let temp = TempPath::with_extension("yaml");
    write_str_to_file(&temp, DUMMY_DATA);

    let path = temp.as_ref().to_string_lossy().to_string();
    let mut term = Mock::default().on_read_line(lines(&["yes"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut save = Save::new(path.clone());
    assert_eq!(ApplyOutcome::Applied, save.apply(&mut looper).unwrap());
    assert_eq!(
        "Output file exists. Overwrite? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );
    assert_eq!(
        format!("Saved scenario to '{}'.\n", path),
        looper.terminal().invocations()[2].print().unwrap_output()
    );

    let written = read_str_from_file(&temp);
    assert_ne!(DUMMY_DATA, written);
    drop(temp);
}

#[test]
fn apply_existing_file_terminal_error() {
    const DUMMY_DATA: &str = "dummy data";
    let temp = TempPath::with_extension("yaml");
    write_str_to_file(&temp, DUMMY_DATA);

    let path = temp.as_ref().to_string_lossy().to_string();
    let mut term =
        Mock::default().on_read_line(|| Err(AccessTerminalError("terminal exploded".into())));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut save = Save::new(path.clone());
    assert_eq!(
        AccessTerminalError("terminal exploded".into()),
        save.apply(&mut looper)
            .unwrap_err()
            .access_terminal()
            .unwrap()
    );
    drop(temp);
}

#[test]
fn apply_existing_file_skip() {
    const DUMMY_DATA: &str = "dummy data";
    let temp = TempPath::with_extension("yaml");
    write_str_to_file(&temp, DUMMY_DATA);

    let path = temp.as_ref().to_string_lossy().to_string();
    let mut term = Mock::default().on_read_line(lines(&["no"]));
    let commander = Commander::new(command_parsers());
    let mut context = TestContext::default();
    let mut looper = Looper::new(
        &mut term,
        &commander,
        &mut context,
    );
    let mut save = Save::new(path.clone());
    assert_eq!(ApplyOutcome::Skipped, save.apply(&mut looper).unwrap());
    assert_eq!(
        "Output file exists. Overwrite? [y/N]: ",
        looper.terminal().invocations()[0].print().unwrap_output()
    );

    let written = read_str_from_file(&temp);
    assert_eq!(DUMMY_DATA, written);
    drop(temp);
}

#[test]
fn parse() {
    let commander = Commander::new(command_parsers());
    commander.parse("save out.yaml").unwrap();
}

#[test]
#[should_panic(expected = "empty arguments to 'save'")]
fn parse_empty_args_fails() {
    let commander = Commander::new(command_parsers());
    commander.parse("save").unwrap();
}

#[test]
fn parser_lints() {
    assert_pedantic::<TestContext, _, Mock>(&Parser::default());
}
