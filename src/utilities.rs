use std::error::Error as ErrorTrait;
use std::ffi::OsStr;
use std::io::Error;
use std::io::ErrorKind;
use std::num::ParseIntError;
use std::process::exit;
use std::rc::Rc;

use cursive::views::EditView;
use directories::ProjectDirs;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub fn parse_number(number_str: &str) -> Result<u64, ParseIntError> {
    if number_str.starts_with("0x") {
        u64::from_str_radix(&number_str[2..], 16)
    } else if number_str.starts_with('0') {
        u64::from_str_radix(number_str, 8)
    } else {
        u64::from_str_radix(number_str, 10)
    }
}

pub fn parse_number_or_zero(number_str: &str) -> u64 {
    match parse_number(number_str) {
        Ok(number) => number,
        _ => 0
    }
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("io.github.chrisvest", "", "xv")
}

pub fn get_content(ev: &mut EditView) -> Rc<String> {
    ev.get_content()
}

pub fn exit_reader_open_error<T>(error: Error, file_name: T) -> !
    where T: AsRef<OsStr> {
    let name = file_name.as_ref();
    match error.kind() {
        ErrorKind::NotFound => {
            eprintln!("File not found: {:#?}", name);
        },
        ErrorKind::PermissionDenied => {
            eprintln!("Permission denied: {:#?}", name);
        },
        _ => {
            eprintln!("{}: {:#?}", error.description(), name);
        }
    }
    exit(1)
}
