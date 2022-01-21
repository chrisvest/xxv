use crate::hex_tables::{BYTE_RENDER, UNICODE_TEXT_TABLE};
use crate::hex_view::HexView;
use crate::xxv_tui::{OBJ_FIND_ASCII, OBJ_FIND_HEX, OBJ_HEX_VIEW};
use cursive::event::Key;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, OnEventView, TextView};
use cursive::Cursive;

pub fn search_dialog(s: &mut Cursive) {
    let ascii_field = EditView::new()
        .content("")
        .on_edit(edit_ascii)
        .on_submit(on_find)
        .with_name(OBJ_FIND_ASCII)
        .min_width(48);

    let hex_field = EditView::new()
        .content("")
        .on_edit(edit_hex)
        .on_submit(on_find)
        .with_name(OBJ_FIND_HEX)
        .min_width(48);

    let layout = LinearLayout::vertical()
        .child(
            LinearLayout::horizontal()
                .child(TextView::new("ASCII: "))
                .child(ascii_field),
        )
        .child(
            LinearLayout::horizontal()
                .child(TextView::new("HEX:   "))
                .child(hex_field),
        );

    let dialog = Dialog::around(layout)
        .dismiss_button("Cancel")
        .button("Search", do_find)
        .title("Search");

    let esc_view = OnEventView::new(dialog)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_find);

    s.add_layer(esc_view);
}

fn edit_ascii(s: &mut Cursive, text: &str, _cursor: usize) {
    s.call_on_name(OBJ_FIND_HEX, |v: &mut EditView| {
        let bytes = text.as_bytes();
        let mut hex_text = String::new();
        for b in bytes {
            hex_text.push_str(BYTE_RENDER[usize::from(*b)]);
        }
        v.set_content(hex_text);
        update_companion_accessibility(text, v)
    });
}

fn edit_hex(s: &mut Cursive, text: &str, cursor: usize) {
    let mut ascii = String::new();
    s.call_on_name(OBJ_FIND_HEX, |v: &mut EditView| {
        let mut hex_text = v.get_content().to_string();
        let mut bytes: Vec<u8> = Vec::new();
        let removed_illegal_digits = hex_to_bytes(&mut hex_text, &mut bytes);
        for b in bytes {
            ascii.push_str(UNICODE_TEXT_TABLE[usize::from(b)]);
        }
        if removed_illegal_digits {
            v.set_content(hex_text);
            v.set_cursor(cursor - 1);
        };
    });
    s.call_on_name(OBJ_FIND_ASCII, |v: &mut EditView| {
        v.set_content(ascii);
        update_companion_accessibility(text, v)
    });
}

fn hex_to_bytes(hex_text: &mut String, bytes: &mut Vec<u8>) -> bool {
    let mut modified = false;
    let mut i = 0;
    while i < hex_text.len() {
        let byte = if hex_text.len() > i + 1 {
            &hex_text[i..=i + 1]
        } else {
            &hex_text[i..=i]
        };
        if let Ok(mut byte_value) = u8::from_str_radix(byte, 16) {
            if byte.len() == 1 {
                byte_value <<= 4;
            }
            bytes.push(byte_value);
            i += 2;
        } else {
            // Remove characters that cannot be parsed as hexadecimal.
            if u8::from_str_radix(&byte[0..=0], 16).is_ok() {
                hex_text.remove(i + 1);
            } else {
                hex_text.remove(i);
            }
            modified = true;
        }
    }
    modified
}

fn update_companion_accessibility(text: &str, v: &mut EditView) {
    if !text.is_empty() && v.is_enabled() {
        v.disable();
    } else if text.is_empty() && !v.is_enabled() {
        v.enable();
    }
}

fn on_find(s: &mut Cursive, _text: &str) {
    do_find(s);
}

fn do_find(s: &mut Cursive) {
    let mut bytes: Vec<u8> = Vec::new();
    s.call_on_name(OBJ_FIND_HEX, |v: &mut EditView| {
        let mut contents = v.get_content().to_string();
        hex_to_bytes(&mut contents, &mut bytes);
    });
    s.pop_layer();
    s.call_on_name(OBJ_HEX_VIEW, |v: &mut HexView| {
        v.search(&bytes);
    });
}
