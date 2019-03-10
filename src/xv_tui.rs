use cursive::Cursive;
use cursive::event::Key;
use cursive::theme::*;
use cursive::traits::*;
use cursive::utils::markup::*;
use cursive::views::*;

use crate::hex_reader::HexReader;
use crate::hex_view::HexView;

pub fn run_tui(reader: HexReader) {
    let mut tui = Cursive::default();
    tui.add_global_callback('q', quit);
    tui.add_global_callback(Key::Esc, quit);
    tui.add_global_callback('?', help);
    tui.add_global_callback(Key::F1, help);
    tui.add_global_callback('w', adjust_line_width);

    let hints_style = ColorStyle::new(
        ColorType::Palette(PaletteColor::Tertiary),
        ColorType::Palette(PaletteColor::Background));
    let hint_key_style = Style::none().combine(hints_style).combine(Effect::Underline);

    let data_pane = HexView::new(reader).with_id("hex_view");

    let mut hints_bar_string = StyledString::new();
    hints_bar_string.append_styled("Q", hint_key_style);
    hints_bar_string.append_styled("uit   ", hints_style);
    hints_bar_string.append_styled("G", hint_key_style);
    hints_bar_string.append_styled("o to   ", hints_style);
    hints_bar_string.append_styled("O", hint_key_style);
    hints_bar_string.append_styled("pen   ", hints_style);
    hints_bar_string.append_styled("S", hint_key_style);
    hints_bar_string.append_styled("witch   ", hints_style);
    hints_bar_string.append_styled("V", hint_key_style);
    hints_bar_string.append_styled("isual   ", hints_style);
    hints_bar_string.append_styled("W", hint_key_style);
    hints_bar_string.append_styled("idth   ", hints_style);

    let hints_bar = TextView::new(hints_bar_string);
    let progress_bar = TextView::new(StyledString::styled("progress bar", hints_style));

    let work_area = StackView::new().fullscreen_layer(data_pane.full_screen());

    let status_bar = PaddedView::new((1, 1, 0, 0), LinearLayout::horizontal()
        .child(hints_bar.full_width())
        .child(progress_bar));

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

fn adjust_line_width(s: &mut Cursive) {
    fn set_line_length(s: &mut Cursive, line_length: &str) {
        s.pop_layer();
        if !line_length.is_empty() {
            let len_result = if line_length.starts_with("0x") {
                u64::from_str_radix(&line_length[2..], 16)
            } else if line_length.starts_with("0") {
                u64::from_str_radix(line_length, 8)
            } else {
                u64::from_str_radix(line_length, 10)
            };
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

    let current_length = s.call_on_id("hex_view", |v: &mut HexView| v.get_line_length()).unwrap();
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
