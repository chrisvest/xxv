use std::convert::TryFrom;

use cursive::event::Key;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, OnEventView, TextView};
use cursive::Cursive;

use crate::hex_view::HexView;
use crate::utilities::{get_content, parse_number};
use crate::xxv_tui::{OBJ_GROUP, OBJ_HEX_VIEW, OBJ_LINE_WIDTH};

pub fn open_set_width_dialog(s: &mut Cursive) {
    let (current_width, current_group) = get_current_width_and_group(s);
    let current_width_str = format!("{}", current_width);
    let current_group_str = format!("{}", current_group);

    let line_width_edit = EditView::new()
        .content(current_width_str)
        .with_name(OBJ_LINE_WIDTH)
        .min_width(8);

    let group_edit = EditView::new()
        .content(current_group_str)
        .with_name(OBJ_GROUP)
        .min_width(8);

    let editors = LinearLayout::vertical()
        .child(line_width_edit)
        .child(group_edit);

    let layout = LinearLayout::horizontal()
        .child(TextView::new("Line width:  \nGroup:  "))
        .child(editors);

    let dialog = Dialog::around(layout)
        .title("Line Width")
        .dismiss_button("Cancel")
        .button("Ok", do_set_widths);

    let event_view = OnEventView::new(dialog)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_set_widths);

    s.add_layer(event_view)
}

fn get_current_width_and_group(s: &mut Cursive) -> (u64, u16) {
    s.call_on_name(OBJ_HEX_VIEW, |v: &mut HexView| {
        (v.get_line_width(), v.get_group())
    })
    .unwrap()
}

fn do_set_widths(s: &mut Cursive) {
    let line_width = s.call_on_name(OBJ_LINE_WIDTH, get_content).unwrap();
    let group = s.call_on_name(OBJ_GROUP, get_content).unwrap();

    s.pop_layer();

    if !line_width.is_empty() {
        match parse_number(&line_width) {
            Ok(width) => s.call_on_name(OBJ_HEX_VIEW, |v: &mut HexView| {
                if width > 0 {
                    v.set_line_width(width);
                }
            }),
            _ => None,
        };
    }

    if !group.is_empty() {
        match parse_number(&group) {
            Ok(group) => s.call_on_name(OBJ_HEX_VIEW, |v: &mut HexView| {
                if group > 0 && group < u64::from(std::u16::MAX) {
                    v.set_group(u16::try_from(group).unwrap());
                }
            }),
            _ => None,
        };
    }
}
