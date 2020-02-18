use crate::hex_view::HexView;
use crate::xxv_state::XxvState;
use crate::xxv_tui::{ShowError, OBJ_CURRENT_DIR, OBJ_DIR_SELECTOR, OBJ_FILE_SELECTOR, OBJ_HEX_VIEW};
use cursive::event::Key;
use cursive::theme::Effect;
use cursive::traits::{Resizable, Nameable};
use cursive::views::{
    Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView,
};
use cursive::Cursive;
use std::ffi::{OsStr, OsString};
use std::io::Result;

pub fn open_file_dialog(s: &mut Cursive) {
    let dir_selector: SelectView<OsString> =
        SelectView::new().on_submit(select_directory).autojump();
    let file_selector: SelectView<OsString> = SelectView::new().autojump();

    let layout = LinearLayout::vertical()
        .child(
            TextView::new("")
                .center()
                .effect(Effect::Bold)
                .with_name(OBJ_CURRENT_DIR),
        )
        .child(DummyView)
        .child(
            LinearLayout::horizontal()
                .child(ScrollView::new(
                    dir_selector.with_name(OBJ_DIR_SELECTOR).full_width(),
                ))
                .child(ScrollView::new(
                    file_selector.with_name(OBJ_FILE_SELECTOR).full_width(),
                )),
        )
        .fixed_height(s.screen_size().y - 11)
        .fixed_width(s.screen_size().x - 20);

    let file_picker = Dialog::new()
        .title("Open file")
        .content(layout)
        .dismiss_button("Cancel")
        .button("Open", do_open_file);

    let event_view = OnEventView::new(file_picker)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_open_file);
    s.add_layer(event_view);
    select_directory(s, &OsString::new());
}

fn select_directory(s: &mut Cursive, dir: &OsStr) {
    let mut current_dir = s.find_name::<TextView>(OBJ_CURRENT_DIR).unwrap();
    let mut dir_selector = s.find_name::<SelectView<OsString>>(OBJ_DIR_SELECTOR).unwrap();
    let mut file_selector = s
        .find_name::<SelectView<OsString>>(OBJ_FILE_SELECTOR)
        .unwrap();
    let saved_current_dir = s
        .with_user_data(|state: &mut XxvState| state.current_directory().to_path_buf())
        .unwrap();

    let result = s.with_user_data(|state: &mut XxvState| {
        state.change_directory(dir);
        fill_selectors(
            &mut current_dir,
            &mut dir_selector,
            &mut file_selector,
            state,
        )
    });

    if let Some(Err(error)) = result {
        s.show_error(error);
        if let Some(Err(error)) = s.with_user_data(|state: &mut XxvState| {
            state.set_directory(saved_current_dir);
            fill_selectors(
                &mut current_dir,
                &mut dir_selector,
                &mut file_selector,
                state,
            )
        }) {
            s.show_error(error);
            if let Some(Err(error)) = s.with_user_data(|state: &mut XxvState| {
                state.reset_current_directory()?;
                fill_selectors(
                    &mut current_dir,
                    &mut dir_selector,
                    &mut file_selector,
                    state,
                )
            }) {
                s.show_error(error)
            }
        }
    }
}

fn fill_selectors(
    current_dir: &mut TextView,
    dir_selector: &mut SelectView<OsString>,
    file_selector: &mut SelectView<OsString>,
    state: &mut XxvState,
) -> Result<()> {
    dir_selector.clear();
    file_selector.clear();
    dir_selector.add_item("..", OsString::from(".."));

    match state.list_directory() {
        Ok(list) => {
            for entry in list {
                let dir_entry = entry.unwrap();
                let file_type = dir_entry.file_type().unwrap();
                let label: String = dir_entry.file_name().as_os_str().to_string_lossy().into();
                if file_type.is_dir() {
                    dir_selector.add_item(label, dir_entry.file_name());
                } else if file_type.is_file() {
                    file_selector.add_item(label, dir_entry.file_name());
                }
            }
            current_dir.set_content(state.current_directory().as_os_str().to_string_lossy());
            dir_selector.sort_by_label();
            file_selector.sort_by_label();
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn do_open_file(s: &mut Cursive) {
    let file_selector = s
        .find_name::<SelectView<OsString>>(OBJ_FILE_SELECTOR)
        .unwrap();
    s.pop_layer();
    if let Some(rc_file) = file_selector.selection() {
        let file_name = rc_file.as_ref();
        let current_file = s
            .call_on_name(OBJ_HEX_VIEW, |view: &mut HexView| view.get_reader_state())
            .unwrap();
        if let Some(reader_result) = s.with_user_data(|state: &mut XxvState| {
            let path = state.resolve_path(file_name);
            state.close_reader(current_file);
            state.open_reader(path)
        }) {
            match reader_result {
                Ok(reader) => s.call_on_name(OBJ_HEX_VIEW, |view: &mut HexView| {
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
