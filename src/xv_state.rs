use std::io::Result;
use std::path::{Path, PathBuf};
use std::env;

use directories::BaseDirs;

use cursive::theme::{Palette, Theme};
use cursive::theme::BaseColor::*;
use cursive::theme::Color::*;

use crate::byte_reader::TilingByteReader;
use crate::hex_reader::HexReader;
use std::fs;
use std::ffi::OsStr;

pub struct XvState {
    theme: bool,
    current_dir: PathBuf,
}

impl XvState {
    pub fn new() -> XvState {
        let current_dir = env::current_dir().unwrap_or_else(|_e| {
            let user_dirs = BaseDirs::new();
            if let Some(ud) = user_dirs {
                ud.home_dir().to_path_buf()
            } else {
                PathBuf::default()
            }
        });
        XvState {theme: true, current_dir}
    }
    
    pub fn open_reader<P: AsRef<Path>>(&mut self, file_name: P) -> Result<HexReader> {
        let b_reader = TilingByteReader::new(file_name)?;
        HexReader::new(b_reader)
    }
    
    pub fn change_directory(&mut self, cd: &OsStr) {
        if cd == ".." {
            self.current_dir.pop();
        } else {
            self.current_dir.push(cd);
        }
    }
    
    pub fn list_directory(&self) -> Result<fs::ReadDir> {
        fs::read_dir(&self.current_dir)
    }
    
    pub fn current_directory(&self) -> &Path {
        &self.current_dir
    }
    
    pub fn toggle_theme(&mut self) {
        self.theme = !self.theme;
    }
    
    pub fn current_theme(&self) -> Theme {
        if self.theme {
            Theme::default()
        } else {
            let mut palette = Palette::default();
            palette.set_color("background", TerminalDefault);
            palette.set_color("shadow", Dark(White));
            palette.set_color("view", TerminalDefault);
            palette.set_color("primary", Light(White));
            palette.set_color("secondary", Light(Blue));
            palette.set_color("tertiary", Dark(White));
            palette.set_color("title_primary", Light(Red));
            palette.set_color("title_secondary", Dark(Yellow));
            palette.set_color("highlight", Dark(Red));
            palette.set_color("highlight_inactive", Dark(Blue));
            Theme { shadow: false, palette, ..Theme::default() }
        }
    }
}
