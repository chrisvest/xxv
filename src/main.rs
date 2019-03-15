#![forbid(unsafe_code)]
#[macro_use]
extern crate clap;

use std::io::Result;

use clap::{App, Arg};

use crate::byte_reader::TilingByteReader;
use crate::hex_reader::HexReader;

mod utilities;
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
    let file_name = matches.value_of_os("file").unwrap();
    let b_reader = TilingByteReader::new(file_name)?;
    let h_reader = HexReader::new(b_reader)?;
    xv_tui::run_tui(h_reader);
    Ok(())
}
