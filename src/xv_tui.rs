use cursive::Cursive;
use cursive::event::Key;
use cursive::traits::{Boxable, Identifiable};
use cursive::views::{Dialog, LinearLayout, StackView};

use crate::goto_dialog::open_goto_dialog;
use crate::hex_reader::HexReader;
use crate::hex_view::HexView;
use crate::set_width_dialog::open_set_width_dialog;
use crate::status_bar::new_status_bar;

pub fn run_tui(reader: HexReader) {
    let mut tui = Cursive::default();
    tui.add_global_callback('q', quit);
    tui.add_global_callback(Key::Esc, quit);
    tui.add_global_callback('?', help);
    tui.add_global_callback(Key::F1, help);
    tui.add_global_callback('w', open_set_width_dialog);
    tui.add_global_callback('g', open_goto_dialog);

    let hex_view = HexView::new(reader).with_id("hex_view");

    let work_area = StackView::new().fullscreen_layer(hex_view.full_screen());
    
    let status_bar = new_status_bar();

    tui.screen_mut().add_transparent_layer(LinearLayout::vertical()
        .child(work_area)
        .child(status_bar)
        .full_screen());

    tui.run();
}

fn quit(s: &mut Cursive) {
    s.quit()
}

fn help(s: &mut Cursive) {
    s.add_layer(Dialog::info("Helpful text\n\nbla bla bla..."))
}
