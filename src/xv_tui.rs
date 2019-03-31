use cursive::Cursive;
use cursive::event::Key;
use cursive::traits::{Boxable, Identifiable};
use cursive::views::{Dialog, LinearLayout, TextView};

use crate::goto_dialog::open_goto_dialog;
use crate::hex_reader::HexReader;
use crate::hex_view::HexView;
use crate::open_file_dialog::open_file_dialog;
use crate::set_width_dialog::open_set_width_dialog;
use crate::status_bar::new_status_bar;
use crate::switch_file_dialog::switch_file_dialog;
use crate::xv_state::XvState;
use std::io::Error;
use crate::help_text::show_help;

pub fn run_tui(reader: HexReader, state: XvState) {
    let mut tui = Cursive::default();
    tui.set_theme(state.current_theme());
    tui.set_user_data(state);
    
    tui.add_global_callback('q', quit);
    tui.add_global_callback(Key::Esc, quit);
    tui.add_global_callback('?', show_help);
    tui.add_global_callback(Key::F1, show_help);
    tui.add_global_callback('w', open_set_width_dialog);
    tui.add_global_callback('g', open_goto_dialog);
    tui.add_global_callback('t', change_theme);
    tui.add_global_callback('o', open_file_dialog);
    tui.add_global_callback('s', switch_file_dialog);

    let hex_view = HexView::new(reader).with_id("hex_view");
    let status_bar = new_status_bar();

    tui.screen_mut().add_transparent_layer(LinearLayout::vertical()
        .child(hex_view)
        .child(status_bar)
        .full_screen());

    tui.run();
}

fn quit(s: &mut Cursive) {
    let reader_state = s.call_on_id("hex_view", |view: &mut HexView| {
        view.get_reader_state()
    }).unwrap();
    s.with_user_data(|state: &mut XvState| {
        state.close_reader(reader_state);
        state.store();
    });
    s.quit()
}

fn change_theme(s: &mut Cursive) {
    let new_theme = s.with_user_data(|state: &mut XvState| {
        state.toggle_theme();
        state.current_theme()
    });
    if let Some(t) = new_theme {
        s.set_theme(t);
    }
}

pub trait ShowError {
    fn show_error(&mut self, error: Error);
}

impl ShowError for Cursive {
    fn show_error(&mut self, error: Error) {
        self.add_layer(Dialog::info("Error").content(
            TextView::new(format!("{}", error))));
    }
}
