use std::io::Error;
use std::path::PathBuf;

use cursive::Cursive;
use cursive::event::Key;
use cursive::traits::{Resizable, Nameable};
use cursive::views::{Dialog, LinearLayout, TextView};

use crate::search_dialog::search_dialog;
use crate::goto_dialog::open_goto_dialog;
use crate::help_text::show_help;
use crate::hex_reader::HexReader;
use crate::hex_view::HexView;
use crate::open_file_dialog::open_file_dialog;
use crate::panic_hook::archive_last_crash;
use crate::set_width_dialog::open_set_width_dialog;
use crate::status_bar::new_status_bar;
use crate::switch_file_dialog::switch_file_dialog;
use crate::utilities::{exit_reader_open_error, PKG_REPOSITORY};
use crate::xxv_state::XxvState;

pub const OBJ_HEX_VIEW: &str = "hex_view";
pub const OBJ_SWITCHER: &str = "file_switcher";
pub const OBJ_CURRENT_DIR: &str = "current_dir";
pub const OBJ_DIR_SELECTOR: &str = "dir_selector";
pub const OBJ_FILE_SELECTOR: &str = "file_selector";
pub const OBJ_LINE_WIDTH: &str = "line_width";
pub const OBJ_GROUP: &str = "group_width";
pub const OBJ_GOTO_OFFSET: &str = "goto_offset";
pub const OBJ_GOTO_MUL1: &str = "goto_mul1";
pub const OBJ_GOTO_MUL2: &str = "goto_mul2";
pub const OBJ_FIND_ASCII: &str = "find_ascii";
pub const OBJ_FIND_HEX: &str = "find_hex";

pub fn run_tui(reader: Option<HexReader>, mut state: XxvState) {
    let mut tui = Cursive::default();
    tui.set_theme(state.current_theme());

    tui.add_global_callback('q', quit);
    tui.add_global_callback(Key::Esc, quit);
    tui.add_global_callback('?', show_help);
    tui.add_global_callback(Key::F1, show_help);
    tui.add_global_callback('w', open_set_width_dialog);
    tui.add_global_callback('g', open_goto_dialog);
    tui.add_global_callback('t', change_theme);
    tui.add_global_callback('o', open_file_dialog);
    tui.add_global_callback('s', switch_file_dialog);
    tui.add_global_callback('f', |v| search_dialog(v, true));
    tui.add_global_callback('F', |v| search_dialog(v, false));

    let hex_view = match reader {
        Some(reader) => HexView::new(reader),
        None => {
            let recent = state.recent_files();
            let file_name = recent[0].path().to_path_buf();
            match state.open_reader(&file_name) {
                Ok(reader) => HexView::new(reader),
                Err(e) => exit_reader_open_error(e, file_name.as_os_str()),
            }
        }
    }
    .with_name(OBJ_HEX_VIEW);

    tui.set_user_data(state);

    let status_bar = new_status_bar();

    tui.screen_mut().add_transparent_layer(
        LinearLayout::vertical()
            .child(hex_view)
            .child(status_bar)
            .full_screen(),
    );

    if let Some(archived_crash_log) = archive_last_crash() {
        show_crash_dialog(&mut tui, archived_crash_log);
    }

    tui.run();
}

fn quit(s: &mut Cursive) {
    let reader_state = s
        .call_on_name(OBJ_HEX_VIEW, |view: &mut HexView| view.get_reader_state())
        .unwrap();
    s.with_user_data(|state: &mut XxvState| {
        state.close_reader(reader_state);
        state.store();
    });
    s.quit()
}

fn change_theme(s: &mut Cursive) {
    let new_theme = s.with_user_data(|state: &mut XxvState| {
        state.toggle_theme();
        state.current_theme()
    });
    if let Some(t) = new_theme {
        s.set_theme(t);
    }
}

fn show_crash_dialog(s: &mut Cursive, archived_crash_log: PathBuf) {
    let msg = format!(
        include_str!("crash_message.txt"),
        archived_crash_log, PKG_REPOSITORY
    );
    let text_view = TextView::new(msg);
    s.add_layer(Dialog::info("Oops!").content(text_view));
}

pub trait ShowError {
    fn show_error(&mut self, error: Error);
}

impl ShowError for Cursive {
    fn show_error(&mut self, error: Error) {
        self.add_layer(Dialog::info("Error").content(TextView::new(format!("{}", error))));
    }
}
