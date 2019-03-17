#![forbid(unsafe_code)]
#[macro_use]
extern crate clap;

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

    // todo support opening multiple files at once
    let mut state = XvState::new();
    let file_name = matches.value_of_os("file").unwrap();
    let h_reader = state.open_reader(file_name)?;
    xv_tui::run_tui(h_reader, state);
    Ok(())
}
