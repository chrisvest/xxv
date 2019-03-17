
use cursive::theme::{Theme, Palette};
use cursive::theme::Color::*;
use cursive::theme::BaseColor::*;

pub struct XvState {
    theme: bool,
}

impl XvState {
    pub fn new() -> XvState {
        XvState {theme: true}
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
