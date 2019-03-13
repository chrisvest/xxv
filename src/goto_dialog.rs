use cursive::Cursive;
use cursive::views::{LinearLayout, EditView, TextView, Dialog, OnEventView, DummyView};
use cursive::traits::{Identifiable, Boxable};
use cursive::direction::Orientation;
use cursive::event::Key;
use crate::hex_view::HexView;
use crate::utilities::parse_number_or_zero;

pub fn open_goto_dialog(s: &mut Cursive) {
    let (line_width, length) = s.call_on_id(
        "hex_view", |v: &mut HexView| (v.get_line_width(), v.get_length())).unwrap();
    let last_line_idx = length / line_width;
    
    let edit_boxes = LinearLayout::new(Orientation::Horizontal)
        .child(EditView::new().content("0").with_id("offset").min_width(18))
        .child(TextView::new(" + "))
        .child(EditView::new().content("0").with_id("mul1").min_width(18))
        .child(TextView::new(" * "))
        .child(EditView::new().content(format!("{}", line_width)).with_id("mul2").min_width(18));
    
    let info_boxes = LinearLayout::new(Orientation::Horizontal)
        .child(TextView::new("Line width:  \nFile size:  \nLast line index:  "))
        .child(TextView::new(format!("{}  \n{}  \n{}", line_width, length, last_line_idx)))
        .child(TextView::new(format!("0x{:X}\n0x{:X}\n0x{:X}", line_width, length, last_line_idx)));
    
    let layout = LinearLayout::new(Orientation::Vertical)
        .child(edit_boxes)
        .child(DummyView)
        .child(info_boxes);
    
    let dialog = Dialog::around(layout)
        .dismiss_button("Cancel")
        .button("Go", do_goto)
        .title("Go to");
    
    let esc_view = OnEventView::new(dialog)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event(Key::Enter, do_goto);
    
    s.add_layer(esc_view);
}

fn do_goto(s: &mut Cursive) {
    let offset_str = s.call_on_id("offset", |v: &mut EditView| v.get_content()).unwrap();
    let mul1_str = s.call_on_id("mul1", |v: &mut EditView| v.get_content()).unwrap();
    let mul2_str = s.call_on_id("mul2", |v: &mut EditView| v.get_content()).unwrap();
    
    s.pop_layer();

    let offset = parse_number_or_zero(&offset_str);
    let mul1 = parse_number_or_zero(&mul1_str);
    let mul2 = parse_number_or_zero(&mul2_str);
    
    let target = offset + mul1 * mul2;
    
    s.call_on_id("hex_view", |view: &mut HexView| {
        view.go_to_offset(target);
    });
}
