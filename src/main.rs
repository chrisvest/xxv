//   Copyright 2019 Chris Vest
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#![forbid(unsafe_code)]
extern crate serde;
extern crate serde_derive;

use std::error::Error;
use std::io::ErrorKind;
use std::process::exit;

use crate::utilities::{PKG_DESCRIPTION, PKG_NAME, PKG_VERSION};
use crate::xv_state::XvState;

mod utilities;
mod panic_hook;
mod xv_state;
mod byte_reader;
mod hex_tables;
mod hex_reader;
mod hex_view;
mod hex_view_printers;
mod set_width_dialog;
mod goto_dialog;
mod open_file_dialog;
mod switch_file_dialog;
mod status_bar;
mod help_text;
mod xv_tui;

fn main() {
    panic_hook::install();

    let mut args = std::env::args_os();
    args.next(); // The first argument is (most likely) the path to our executable.
    let file_arg = args.next();

    if file_arg.is_none() {
        eprintln!("Error: The 'file' argument is required.");
        eprintln!();
        eprintln!("For more information, try --help.");
        exit(64); // EX_USAGE from sysexits.h
    }
    
    let file_name = file_arg.unwrap();
    
    if file_name.eq("-h") || file_name.eq("--help") {
        eprintln!("{} {}", PKG_NAME, PKG_VERSION);
        eprintln!("{}", PKG_DESCRIPTION);
        eprintln!();
        eprintln!("{}", include_str!("usage.txt"));
        return;
    }

    if file_name.eq("-v") || file_name.eq("--version") {
        eprintln!("{} {}", PKG_NAME, PKG_VERSION);
        return;
    }
    
    let mut state = XvState::load();
    match state.open_reader(&file_name) {
        Ok(h_reader) => xv_tui::run_tui(h_reader, state),
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("File not found: {:#?}", &file_name);
                },
                ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied: {:#?}", &file_name);
                },
                _ => {
                    eprintln!("{}: {:#?}", e.description(), &file_name);
                }
            }
            exit(1)
        }
    }
}
