use cursive::Cursive;
use cursive::views::{EditView, Dialog, OnEventView};
use cursive::traits::Identifiable;
use cursive::event::Key;
use crate::hex_view::HexView;
use std::num::ParseIntError;

pub fn open_set_width_dialog(s: &mut Cursive) {
    let current_length = get_current_width(s);
    let current_length_str = format!("{}", current_length);
    
    let edit_view = EditView::new()
        .content(current_length_str)
        .on_submit(set_line_length)
        .with_id("line_length");
    
    let dialog = Dialog::around(edit_view)
        .title("Line Width")
        .dismiss_button("Cancel")
        .button("Ok", |s| {
            let line_length = s.call_on_id(
                "line_length",
                |view: &mut EditView| view.get_content()).unwrap();
            set_line_length(s, &line_length);
        });
    
    let esc_view = OnEventView::new(dialog)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        });
    
    s.add_layer(esc_view)
}

fn get_current_width(s: &mut Cursive) -> u64 {
    s.call_on_id("hex_view", |v: &mut HexView| v.get_line_length()).unwrap()
}

fn set_line_length(s: &mut Cursive, line_length: &str) {
    s.pop_layer();
    if !line_length.is_empty() {
        let len_result = parse_number(line_length);
        match len_result {
            Ok(length) => s.call_on_id("hex_view", |v: &mut HexView| {
                if length > 0 {
                    v.set_line_length(length);
                }
            }),
            _ => None,
        };
    }
}

fn parse_number(number_str: &str) -> Result<u64, ParseIntError> {
    if number_str.starts_with("0x") {
        u64::from_str_radix(&number_str[2..], 16)
    } else if number_str.starts_with("0") {
        u64::from_str_radix(number_str, 8)
    } else {
        u64::from_str_radix(number_str, 10)
    }
}
