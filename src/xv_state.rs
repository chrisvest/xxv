use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Result;
use std::path::{Path, PathBuf};

use cursive::theme::{Palette, Theme};
use cursive::theme::BaseColor::*;
use cursive::theme::Color::*;
use directories::BaseDirs;
use rmp_serde::Serializer;
use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};

use crate::byte_reader::TilingByteReader;
use crate::hex_reader::{HexReader, VisualMode};
use crate::utilities;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReaderState {
    path: PathBuf,
    line_width: u64,
    group: u16,
    window_pos: (u64,u64),
    window_size: (u16,u16),
    vis_mode: String
}

impl ReaderState {
    pub fn new(reader: &HexReader) -> ReaderState {
        ReaderState {
            path: reader.get_path(),
            line_width: reader.line_width,
            group: reader.group,
            window_pos: reader.window_pos,
            window_size: reader.window_size,
            vis_mode: reader.vis_mode.into(),
        }
    }
    
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl PartialEq for ReaderState {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl From<VisualMode> for String {
    fn from(mode: VisualMode) -> Self {
        match mode {
            VisualMode::Unicode => String::from("Unicode"),
            VisualMode::Ascii => String::from("Ascii"),
            VisualMode::Off => String::from("Off"),
        }
    }
}

impl From<String> for VisualMode {
    fn from(mode: String) -> Self {
        match mode.as_str() {
            "Unicode" => VisualMode::Unicode,
            "Ascii" => VisualMode::Ascii,
            "Off" => VisualMode::Off,
            _ => VisualMode::Unicode
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XvState {
    theme: bool,
    current_dir: PathBuf,
    max_recent_files: usize,
    recent_files: Vec<ReaderState>,
}

impl XvState {
    pub fn load() -> XvState {
        if let Some(project_dirs) = utilities::project_dirs() {
            let mut state_path = project_dirs.config_dir().to_owned();
            state_path.push("xv.state");
            
            if let Ok(state_file) = File::open(state_path) {
                let result = rmp_serde::from_read(state_file);
                if let Ok(state) = result {
                    return state;
                }
            }
        }
        
        let current_dir = env::current_dir().unwrap_or_else(|_e| {
            let user_dirs = BaseDirs::new();
            if let Some(ud) = user_dirs {
                ud.home_dir().to_path_buf()
            } else {
                PathBuf::default()
            }
        });
        
        XvState {
            theme: true,
            current_dir,
            max_recent_files: 50,
            recent_files: Vec::new()
        }
    }
    
    pub fn store(&mut self) {
        if let Some(project_dirs) = utilities::project_dirs() {
            let mut state_path = project_dirs.config_dir().to_owned();
            create_dir_all(&state_path).unwrap();
            state_path.push("xv.state");

            let mut open_options = OpenOptions::new();
            open_options.create(true).write(true).truncate(true);
            if let Ok(state_file) = open_options.open(state_path) {
                let mut serializer = Serializer::new(state_file);
                self.serialize(&mut serializer).unwrap();
            }
        }
    }
    
    pub fn open_reader<P: AsRef<Path>>(&mut self, file_name: P) -> Result<HexReader> {
        let b_reader = TilingByteReader::new(file_name)?;
        match HexReader::new(b_reader) {
            Ok(mut reader) => {
                let lookup_state = ReaderState::new(&reader);
                if let Some(index) = self.index_of(&lookup_state) {
                    let state = &self.recent_files[index];
                    reader.line_width = state.line_width;
                    reader.group = state.group;
                    reader.window_pos = state.window_pos;
                    reader.window_size = state.window_size;
                    self.recent_files.remove(index);
                };
                Ok(reader)
            },
            err => err
        }
    }
    
    pub fn close_reader(&mut self, reader: ReaderState) {
        if let Some(index) = self.index_of(&reader) {
            self.recent_files.remove(index);
            self.recent_files.insert(0, reader);
        } else if self.recent_files.len() >= self.max_recent_files {
            self.recent_files.remove(0);
            self.recent_files.insert(0, reader);
        } else {
            self.recent_files.insert(0, reader);
        }
    }
    
    fn index_of(&self, reader: &ReaderState) -> Option<usize> {
        for i in 0..self.recent_files.len() {
            if self.recent_files[i].eq(reader) {
                return Some(i);
            }
        }
        None
    }
    
    pub fn remove_recent_file(&mut self, index: usize) {
        self.recent_files.remove(index);
    }
    
    pub fn recent_files(&self) -> &[ReaderState] {
        &self.recent_files
    }
    
    pub fn change_directory(&mut self, cd: &OsStr) {
        if cd == ".." {
            self.current_dir.pop();
        } else {
            self.current_dir.push(cd);
        }
    }
    
    pub fn set_directory(&mut self, path: PathBuf) {
        self.current_dir = path;
    }
    
    pub fn list_directory(&mut self) -> Result<fs::ReadDir> {
        fs::read_dir(&self.current_dir)
    }
    
    pub fn reset_current_directory(&mut self) -> Result<()> {
        self.current_dir = env::current_dir()?;
        Ok(())
    }
    
    pub fn current_directory(&self) -> &Path {
        &self.current_dir
    }
    
    pub fn resolve_path(&self, file_name: &OsStr) -> PathBuf {
        let mut cloned_buf = self.current_dir.clone();
        cloned_buf.push(file_name);
        cloned_buf
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
