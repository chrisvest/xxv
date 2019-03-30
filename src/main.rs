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
#[macro_use]
extern crate clap;
extern crate serde;
extern crate serde_derive;

use std::io::Result;

use clap::{App, Arg};

use crate::xv_state::XvState;

mod utilities;
mod xv_state;
mod byte_reader;
mod hex_tables;
mod hex_reader;
mod hex_view;
mod set_width_dialog;
mod goto_dialog;
mod open_file_dialog;
mod switch_file_dialog;
mod status_bar;
mod xv_tui;

fn main() -> Result<()> {
    let matches = App::new("XV Hex Viewer")
        .version(crate_version!())
        .about(crate_description!())
        .arg(Arg::with_name("file")
            .help("File or files to open.")
            .multiple(true)
            .required(true))
        .get_matches();

    let mut state = XvState::load();
    let file_name = matches.value_of_os("file").unwrap();
    let h_reader = state.open_reader(file_name)?;
    xv_tui::run_tui(h_reader, state);
    Ok(())
}
