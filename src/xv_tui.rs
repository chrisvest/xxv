use cursive::Cursive;
use cursive::event::Key;
use cursive::theme::*;
use cursive::traits::*;
use cursive::utils::markup::*;
use cursive::views::*;
use std::fmt::Write;

use crate::hex_reader::HexReader;

pub fn run_tui(reader: &mut HexReader) {
    // +-----------------------------------+
    // |         |             |           |
    // | address | hex view    | data view |  }- data pane (scrollable)
    // |         |             |           |
    // +-----------------------------------+
    // | menu & hints         progress bar | }- status bar
    // +-----------------------------------+

    let mut tui = Cursive::default();
    tui.add_global_callback('q', quit);
    tui.add_global_callback(Key::Esc, quit);
    tui.add_global_callback('?', help);
    tui.add_global_callback(Key::F1, help);

    let hints_style = ColorStyle::new(
        ColorType::Palette(PaletteColor::Tertiary),
        ColorType::Palette(PaletteColor::Background));
    let hint_key_style = Style::none().combine(hints_style).combine(Effect::Underline);

//    let mut data_pane = HexView::new(reader);
    reader.capture().unwrap();
    let string = reader.get_hex();
    let mut data_pane = TextView::new(string.as_str());

    let mut hints_bar_string = StyledString::new();
    hints_bar_string.append_styled("Q", hint_key_style);
    hints_bar_string.append_styled("uit   ", hints_style);
    hints_bar_string.append_styled("G", hint_key_style);
    hints_bar_string.append_styled("o to   ", hints_style);
    hints_bar_string.append_styled("Navigate: ", hints_style);
    hints_bar_string.append_styled("hjkl", hint_key_style);
    hints_bar_string.append_styled("   ", hints_style);
    hints_bar_string.append_styled("F", hint_key_style);
    hints_bar_string.append_styled("ind   ", hints_style);
    hints_bar_string.append_styled("C", hint_key_style);
    hints_bar_string.append_styled("onfigure", hints_style);

    let mut hints_bar = TextView::new(hints_bar_string);
    let mut progress_bar = TextView::new(StyledString::styled("progress bar", hints_style));

    let mut work_area = StackView::new().fullscreen_layer(data_pane.full_screen());

    let mut status_bar = PaddedView::new((1, 1, 0, 0), LinearLayout::horizontal()
        .child(hints_bar.full_width())
        .child(progress_bar));

    tui.screen_mut().add_transparent_layer(LinearLayout::vertical()
        .child(work_area)
        .child(status_bar)
        .full_screen());

//    let select = SelectView::<String>::new()
//        .on_submit(on_submit)
//        .with_id("select")
//        .fixed_size((10, 5));
//
//    fn on_submit(s: &mut Cursive, name: &String) {
//        s.pop_layer();
//        s.add_layer(Dialog::text(format!("Name: {}\nAwesome: yes", name))
//            .title(format!("{}'s info", name))
//            .button("Quit", Cursive::quit));
//    }
//
//    let buttons = LinearLayout::vertical()
//        .child(Button::new("Add new", add_name))
//        .child(Button::new("Delete", delete_name))
//        .child(DummyView)
//        .child(Button::new("Quit", Cursive::quit));
//
//    fn add_name(s: &mut Cursive) {
//        fn ok(s: &mut Cursive, name: &str) {
//            s.call_on_id("select", |view: &mut SelectView<String>| {
//                view.add_item_str(name)
//            });
//            s.pop_layer();
//        }
//
//        s.add_layer(Dialog::around(EditView::new()
//            .on_submit(ok)
//            .with_id("name")
//            .fixed_width(10))
//            .title("Enter a new name")
//            .button("Ok", |s| {
//                let name = s.call_on_id("name", |v: &mut EditView| {
//                    v.get_content()
//                }).unwrap();
//                ok(s, &name);
//            })
//            .button("Cancel", |s| {
//                s.pop_layer();
//            }));
//    }
//
//    fn delete_name(s: &mut Cursive) {
//        let mut select = s.find_id::<SelectView<String>>("select").unwrap();
//        match select.selected_id() {
//            None => s.add_layer(Dialog::info("No name to remove!")),
//            Some(focus) => {
//                select.remove_item(focus);
//            }
//        }
//    }
//
//    siv.add_layer(Dialog::around(LinearLayout::horizontal()
//        .child(select)
//        .child(DummyView)
//        .child(buttons))
//        .title("Select a profile"));

    tui.run();
}

fn quit(s: &mut Cursive) {
    s.quit()
}

fn help(s: &mut Cursive) {
    s.add_layer(Dialog::info("Helpful text\n\nbla bla bla..."))
}
