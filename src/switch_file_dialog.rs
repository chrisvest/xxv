use cursive::Cursive;
use cursive::views::{SelectView, Dialog, LinearLayout, ScrollView, OnEventView};
use cursive::traits::{Boxable, Identifiable};
use cursive::event::Key;
use crate::xv_state::XvState;
use crate::hex_view::HexView;
use std::ffi::OsString;

pub fn switch_file_dialog(s: &mut Cursive) {
    let mut file_selector: SelectView<OsString> = SelectView::new().autojump();
    
    s.with_user_data(|state: &mut XvState| {
        let recent_files = state.recent_files();
        for recent_file in recent_files {
            let path = recent_file.path();
            file_selector.add_item(path.file_name().unwrap().to_string_lossy(), path.into());
        }
    }).unwrap();
    
    let layout = LinearLayout::vertical()
        .child(ScrollView::new(file_selector.with_id("file_selector")).full_screen())
        .fixed_height((s.screen_size().y - 11).min(50))
        .fixed_width((s.screen_size().x - 20).min(80));
    
    let file_switcher = Dialog::new()
        .title("Switch file")
        .content(layout)
        .dismiss_button("Cancel")
        .button("Ok", do_switch_file);
    
    let event_view = OnEventView::new(file_switcher)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_switch_file);
    s.add_layer(event_view);
    
}

fn do_switch_file(s: &mut Cursive) {
    let file_selector = s.find_id::<SelectView<OsString>>("file_selector").unwrap();
    if let Some(rc_file) = file_selector.selection() {
        let file_name = rc_file.as_ref();
        let current_file = s.call_on_id("hex_view", |view: &mut HexView| {
            view.get_reader_state()
        }).unwrap();
        if let Some(reader) = s.with_user_data(|state: &mut XvState| {
            let path = state.resolve_path(file_name);
            state.close_reader(current_file);
            state.open_reader(path).unwrap()
        }) {
            s.call_on_id("hex_view", |view: &mut HexView| {
                view.switch_reader(reader);
            });
        }
    }
    s.pop_layer();
}
