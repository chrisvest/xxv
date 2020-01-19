use cursive::theme::{ColorStyle, ColorType, Effect, PaletteColor, Style};
use cursive::traits::Boxable;
use cursive::utils::markup::StyledString;
use cursive::views::{LinearLayout, PaddedView, TextView};

pub fn new_status_bar() -> PaddedView<LinearLayout> {
    let hints_style = ColorStyle::new(
        ColorType::Palette(PaletteColor::Tertiary),
        ColorType::Palette(PaletteColor::Background),
    );
    let hint_key_style = Style::none()
        .combine(hints_style)
        .combine(Effect::Underline);

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

    PaddedView::lrtb(
        1,
        1,
        0,
        0,
        LinearLayout::horizontal().child(hints_bar.full_width()),
    )
}
