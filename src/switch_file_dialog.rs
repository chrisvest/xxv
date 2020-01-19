use std::ffi::OsString;
use std::path::PathBuf;

use cursive::event::Key;
use cursive::traits::{Boxable, Identifiable};
use cursive::views::{Dialog, LinearLayout, OnEventView, ScrollView, SelectView};
use cursive::Cursive;

use crate::hex_view::HexView;
use crate::xv_state::XvState;
use crate::xv_tui::ShowError;

pub fn switch_file_dialog(s: &mut Cursive) {
    let mut file_selector: SelectView<OsString> = SelectView::new().autojump();

    s.with_user_data(|state: &mut XvState| {
        let recent_files = state.recent_files();
        for recent_file in recent_files {
            let path = recent_file.path();
            file_selector.add_item(format!("{}", path.display()), path.into());
        }
    })
    .unwrap();

    let layout = LinearLayout::vertical()
        .child(ScrollView::new(file_selector.with_name("file_selector")).scroll_x(true))
        .max_height((s.screen_size().y - 11).min(50))
        .max_width((s.screen_size().x - 20).min(80));

    let file_switcher = Dialog::new()
        .title("Switch file")
        .content(layout)
        .dismiss_button("Cancel")
        .button("Ok", do_switch_file);

    let event_view = OnEventView::new(file_switcher)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_switch_file)
        .on_event(Key::Del, remove_selected_file);
    s.add_layer(event_view);
}

fn do_switch_file(s: &mut Cursive) {
    let file_selector = s
        .find_name::<SelectView<OsString>>("file_selector")
        .unwrap();
    s.pop_layer();
    if let Some(rc_file) = file_selector.selection() {
        let file_name = rc_file.as_ref();
        let current_file = s
            .call_on_name("hex_view", |view: &mut HexView| view.get_reader_state())
            .unwrap();
        if let Some(reader_result) = s.with_user_data(|state: &mut XvState| {
            let path = PathBuf::from(file_name);
            let result = state.open_reader(path);
            if result.is_ok() {
                state.close_reader(current_file);
            }
            result
        }) {
            match reader_result {
                Ok(reader) => s.call_on_name("hex_view", |view: &mut HexView| {
                    view.switch_reader(reader);
                }),
                Err(error) => {
                    s.show_error(error);
                    None
                }
            };
        }
    }
}

fn remove_selected_file(s: &mut Cursive) {
    let mut file_selector = s
        .find_name::<SelectView<OsString>>("file_selector")
        .unwrap();
    if let Some(id) = file_selector.selected_id() {
        file_selector.remove_item(id)(s);
        s.with_user_data(|state: &mut XvState| {
            state.remove_recent_file(id);
        });
    }
}
