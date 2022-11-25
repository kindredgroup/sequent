// $coverage:ignore-start

use std::str::FromStr;
use crate::commands::prompt::YesNo;

#[test]
fn yes_no_from_str() {
    assert_eq!(Ok(YesNo::Yes), "y".parse());
    assert_eq!(Ok(YesNo::Yes), "yes".parse());
    assert_eq!(Ok(YesNo::Yes), "Yes".parse());
    assert_eq!(Ok(YesNo::Yes), "YES".parse());
    assert_eq!(Ok(YesNo::No), "n".parse());
    assert_eq!(Ok(YesNo::No), "no".parse());
    assert_eq!(Ok(YesNo::No), "No".parse());
    assert_eq!(Ok(YesNo::No), "No".parse());
    assert_eq!(Err("'nes' is not a yes/no".into()), YesNo::from_str("nes"));
}

#[test]
fn yes_no_implements_default() {
    assert_eq!(YesNo::No, YesNo::default());
}