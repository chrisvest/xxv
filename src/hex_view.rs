use std::convert::TryFrom;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::event::{Event, Key, MouseEvent};
use cursive::theme::ColorStyle;
use cursive::traits::View;
use cursive::Printer;
use cursive::Vec2;
use unicode_width::UnicodeWidthStr;

use crate::hex_reader::{HexReader, Highlight, VisualMode};
use crate::hex_view_printers::{HexPrinter, OffsetPrinter, TableSet, VisualPrinter};
use crate::xxv_state::ReaderState;

pub struct HexView {
    reader: HexReader,
    invalidated_resize: bool,
    invalidated_data_changed: bool,
    show_visual_view: bool,
    offsets_column_pos: Vec2,
    offsets_column_size: Vec2,
    hex_column_pos: Vec2,
    hex_column_size: Vec2,
    visual_column_pos: Vec2,
    visual_column_size: Vec2,
    hex_tables: TableSet,
    visual_tables: TableSet,
}

impl HexView {
    pub fn new(reader: HexReader) -> HexView {
        HexView {
            reader,
            invalidated_resize: true,
            invalidated_data_changed: true,
            show_visual_view: true,
            offsets_column_pos: Vec2::new(0, 0),
            offsets_column_size: Vec2::new(0, 0),
            hex_column_pos: Vec2::new(0, 0),
            hex_column_size: Vec2::new(0, 0),
            visual_column_pos: Vec2::new(0, 0),
            visual_column_size: Vec2::new(0, 0),
            hex_tables: TableSet::new(),
            visual_tables: TableSet::new(),
        }
    }

    pub fn switch_reader(&mut self, reader: HexReader) {
        self.reader = reader;
        self.invalidated_data_changed = true;
        self.invalidated_resize = true;
    }

    pub fn get_reader_state(&self) -> ReaderState {
        ReaderState::new(&self.reader)
    }

    pub fn go_to_offset(&mut self, offset: u64) {
        self.reader.clear_highlights();

        let current_pos = self.reader.window_pos;
        let window_size = self.reader.window_size;
        let current_size = (u64::from(window_size.0), u64::from(window_size.1));
        let line_width = self.reader.line_width;

        let line = offset / line_width;
        let line_offset = offset % line_width;
        let lines_in_file = self.reader.get_lines_in_file();

        let target_pos = if line <= lines_in_file {
            self.reader.highlight(offset, 1, Highlight::Positive);
            (line_offset, line)
        } else {
            (0, lines_in_file)
        };

        // Only adjust the window position if the target position is not within the window bounds.
        if target_pos.0 < current_pos.0
            || target_pos.1 < current_pos.1
            || target_pos.0 > current_pos.0 + current_size.0 - 1
            || target_pos.1 > current_pos.1 + current_size.1 - 1
        {
            self.reader.window_pos = target_pos;
            let max_line_offset = line_width - current_size.0;
            self.reader.window_pos.0 = u64::min(max_line_offset, self.reader.window_pos.0);
        }

        self.invalidated_data_changed = true;
    }

    pub fn set_line_width(&mut self, length: u64) {
        self.reader.line_width = length;
        let lines_in_file = self.reader.get_lines_in_file();
        if self.reader.window_pos.1 > lines_in_file {
            self.reader.window_pos.1 = lines_in_file;
        }
        self.invalidated_resize = true;
        self.invalidated_data_changed = true;
    }

    pub fn get_line_width(&self) -> u64 {
        self.reader.line_width
    }

    pub fn set_group(&mut self, group: u16) {
        self.reader.group = group;
        self.invalidated_resize = true;
        self.invalidated_data_changed = true;
    }

    pub fn get_group(&self) -> u16 {
        self.reader.group
    }

    pub fn get_length(&self) -> u64 {
        self.reader.get_length()
    }

    pub fn search(&mut self, bytes: &[u8]) {
        self.reader.clear_highlights();
        self.reader.search(bytes);
    }

    fn toggle_visual(&mut self) -> EventResult {
        self.visual_tables.clear();
        match self.reader.get_visual_mode() {
            VisualMode::Unicode => {
                self.reader.set_visual_mode(VisualMode::Ascii);
            }
            VisualMode::Ascii => {
                self.reader.set_visual_mode(VisualMode::Off);
                self.show_visual_view = false;
                self.invalidated_resize = true;
            }
            VisualMode::Off => {
                self.reader.set_visual_mode(VisualMode::Unicode);
                self.show_visual_view = true;
                self.invalidated_resize = true;
            }
        }
        EventResult::Consumed(None)
    }

    fn reload_data(&mut self) -> EventResult {
        self.reader.clear_highlights();
        self.reader.capture_before_image();
        self.invalidated_data_changed = true;
        EventResult::Consumed(None)
    }

    fn reopen_and_reload_data(&mut self) -> EventResult {
        self.reader.reopen().unwrap();
        self.reload_data()
    }

    fn on_char_event(&mut self, c: char) -> EventResult {
        match c {
            'j' => self.on_key_event(Key::Down),
            'J' => self.on_key_event(Key::PageDown),
            'k' => self.on_key_event(Key::Up),
            'K' => self.on_key_event(Key::PageUp),
            'h' => self.on_key_event(Key::Left),
            'H' => self.on_key_event(Key::Home),
            'l' => self.on_key_event(Key::Right),
            'L' => self.on_key_event(Key::End),
            'v' => self.toggle_visual(),
            'r' => self.reload_data(),
            'R' => self.reopen_and_reload_data(),
            _ => EventResult::Ignored,
        }
    }

    fn on_mouse_event(&mut self, _offset: Vec2, _position: Vec2, event: MouseEvent) -> EventResult {
        match event {
            MouseEvent::WheelUp => self.on_key_event(Key::Up),
            MouseEvent::WheelDown => self.on_key_event(Key::Down),
            _ => EventResult::Ignored,
        }
    }

    fn on_key_event(&mut self, k: Key) -> EventResult {
        let inner_height = i64::try_from(self.offsets_column_size.y).unwrap();
        let line_width = i64::try_from(self.reader.line_width).unwrap();
        let pos_x = i64::try_from(self.reader.window_pos.0).unwrap();
        let size_x = i64::from(self.reader.window_size.0);
        let offset = match k {
            Key::Down => (0, 1),
            Key::Up => (0, -1),
            Key::Left => (-1, 0),
            Key::Right => (1, 0),
            Key::PageDown => (0, inner_height),
            Key::PageUp => (0, -inner_height),
            Key::Home => (-pos_x, 0),
            Key::End => (line_width - size_x - pos_x, 0),
            _ => (0, 0),
        };
        self.navigate(offset)
    }

    fn navigate(&mut self, offset: (i64, i64)) -> EventResult {
        if offset != (0, 0) {
            let (x, y) = offset;
            if x < 0 {
                let diff = u64::try_from(-x).unwrap();
                let curr = self.reader.window_pos.0;
                self.reader.window_pos.0 = if curr < diff { 0 } else { curr - diff };
            } else {
                let diff = u64::try_from(x).unwrap();
                let next = self.reader.window_pos.0 + diff;
                let line = self.reader.line_width;
                let width = u64::from(self.reader.window_size.0);
                let max = if width > line { 0 } else { line - width };
                self.reader.window_pos.0 = if next > max { max } else { next };
            }
            if y < 0 {
                let diff = u64::try_from(-y).unwrap();
                let curr = self.reader.window_pos.1;
                self.reader.window_pos.1 = if curr < diff { 0 } else { curr - diff };
            } else {
                let diff = u64::try_from(y).unwrap();
                let next = self.reader.window_pos.1 + diff;
                let lines = self.reader.get_lines_in_file();
                self.reader.window_pos.1 = if next > lines { lines } else { next };
            }
            self.invalidated_resize = true;
            self.invalidated_data_changed = true;
            EventResult::Consumed(None)
        } else {
            EventResult::Ignored
        }
    }

    fn draw_bg(&self, printer: &Printer) {
        for y in 0..printer.size.y {
            printer.print_hline((0, y), printer.size.x, " ");
        }
    }

    fn draw_title(&self, printer: &Printer) {
        let title = self.reader.file_name();
        let mut len = title.width();
        let container_width = printer.size.x;
        let spacing = 3;
        let spacing_both_ends = 2 * spacing;
        if len + spacing_both_ends > container_width {
            len = printer.size.x - spacing_both_ends;
        }
        let offset = spacing + HAlign::Center.get_offset(len, container_width - spacing_both_ends);
        printer.with_high_border(false, |p| {
            p.print((offset - 2, 0), "┤ ");
            p.print((offset + len, 0), " ├");
        });

        printer.with_color(ColorStyle::title_primary(), |p| {
            if len < title.len() {
                p.print((offset, 0), &title[0..len]);
                p.print((offset + len - 1, 0), "…");
            } else {
                p.print((offset, 0), title);
            }
        });
    }

    fn build_prestyled_hex_table(&mut self) {
        self.reader.generate_hex_tables(&mut self.hex_tables);
    }

    fn build_prestyled_visual_table(&mut self) {
        self.reader.generate_visual_tables(&mut self.visual_tables);
    }
}

impl View for HexView {
    fn draw(&self, printer: &Printer) {
        self.draw_bg(printer);
        printer.print_box((0, 0), printer.size, true);
        self.draw_title(printer);

        let mut offset_printer = OffsetPrinter {
            pos: Vec2::new(0, 0),
            printer: &printer
                .offset(self.offsets_column_pos)
                .cropped(self.offsets_column_size),
            spans: Vec::with_capacity(1),
        };
        self.reader.visit_row_offsets(&mut offset_printer);

        let inner_height = self.offsets_column_size.y;
        let border_offset = self.offsets_column_size.x + self.offsets_column_pos.x;
        printer.print_vline(Vec2::new(border_offset, 1), inner_height, "│");

        let mut hex_printer = HexPrinter {
            max_width: 0,
            pos: Vec2::new(0, 0),
            printer: &printer
                .offset(self.hex_column_pos)
                .cropped(self.hex_column_size),
            tables: &self.hex_tables,
        };
        self.reader.visit_hex(&mut hex_printer);

        if self.show_visual_view {
            let border_offset = self.hex_column_pos.x + self.hex_column_size.x;
            printer.print_vline(Vec2::new(border_offset, 1), inner_height, "│");

            let mut visual_printer = VisualPrinter {
                pos: Vec2::new(0, 0),
                printer: &printer
                    .offset(self.visual_column_pos)
                    .cropped(self.visual_column_size),
                tables: &self.visual_tables,
            };
            self.reader.visit_hex(&mut visual_printer);
        }
    }

    fn layout(&mut self, constraint: Vec2) {
        if self.hex_tables.is_empty() {
            self.build_prestyled_hex_table();
        }
        if self.visual_tables.is_empty() {
            self.build_prestyled_visual_table();
        }
        if self.invalidated_resize {
            // The viewing area changed size, or the visual column was toggled.

            // The available height inside the box border:
            let inner_height = constraint.y - 2;
            let inner_height_u16 = u16::try_from(inner_height).unwrap();
            if self.reader.window_size.1 != (inner_height_u16) {
                self.reader.window_size.1 = inner_height_u16;
                self.invalidated_data_changed = true;
            }

            let colw_offsets = self.reader.get_row_offsets_width();
            self.offsets_column_pos = Vec2::new(1, 1);
            self.offsets_column_size = Vec2::new(colw_offsets, inner_height);

            // Box-border, offsets column, separator line + space line:
            let hex_col_start = 1 + colw_offsets + 2;
            self.hex_column_pos = Vec2::new(hex_col_start, 1);
            self.hex_column_size = Vec2::new(constraint.x - hex_col_start - 1, inner_height);

            let group = u64::from(self.reader.group);
            let reader_pos_x = group - 1;
            let vis_group_spacer: isize = if self.show_visual_view { 1 } else { 0 };
            let vis_byte_width: isize = if self.show_visual_view { 1 } else { 0 };

            let avail_width = self.hex_column_size.x;
            let avail_width_isize = isize::try_from(avail_width).unwrap();
            let mut space_left = avail_width_isize;
            let mut hex_width: isize = 0;
            let mut vis_width: isize = 0;
            let mut bytes_consumed = 0;
            let bytes_left_in_line = self.reader.line_width;

            for i in 0..bytes_left_in_line {
                let byte_pair_spacer = if i == 0 { 0 } else { 1 };
                let consumed_by_byte = byte_pair_spacer + 2 + vis_byte_width;
                if space_left - consumed_by_byte >= 0 {
                    space_left -= consumed_by_byte;
                    hex_width += 2 + byte_pair_spacer;
                    vis_width += vis_byte_width;
                    bytes_consumed += 1;

                    if ((reader_pos_x + i) % group) == 0 && i != 0 {
                        // The hex column group spacer replaces the byte pair spacer automatically.
                        if space_left - vis_group_spacer > 0 {
                            space_left -= vis_group_spacer;
                            vis_width += vis_group_spacer;
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            if hex_width + vis_width + 1 < avail_width_isize {
                // Add right padding to hex column.
                hex_width += 1;
            } else if hex_width + vis_width == avail_width_isize {
                // Remove the left hex column padding to squeeze in the last byte.
                self.hex_column_pos.x -= 1;
            }

            let hex_uw = usize::try_from(hex_width).unwrap();
            let vis_uw = usize::try_from(vis_width).unwrap();
            self.hex_column_size = Vec2::new(hex_uw, inner_height);
            self.visual_column_pos = Vec2::new(self.hex_column_pos.x + hex_uw + 1, 1);
            self.visual_column_size = Vec2::new(vis_uw, inner_height);

            if bytes_consumed != self.reader.window_size.0 {
                self.reader.window_size.0 = bytes_consumed;
                self.invalidated_data_changed = true;
            }

            self.invalidated_resize = false;
        }

        if self.invalidated_data_changed {
            // The viewing area was moved or changed size.
            self.reader.capture().unwrap();
            self.invalidated_data_changed = false;
        }
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated_resize || self.invalidated_data_changed
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint // Always take up all the space we can get.
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::WindowResize => {
                self.invalidated_resize = true;
                EventResult::Consumed(None)
            }
            Event::Char(c) => self.on_char_event(c),
            Event::Key(k) => self.on_key_event(k),
            Event::Mouse {
                offset,
                position,
                event,
            } => self.on_mouse_event(offset, position, event),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::byte_reader::TilingByteReader;

    use super::*;

    #[test]
    fn layout_w80_h24_ll16() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        view.reader.line_width = 16;
        let constraint = Vec2::new(80, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(47, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(61, 1));
        assert_eq!(view.visual_column_size, Vec2::new(18, 21));
    }

    #[test]
    fn layout_w80_h24_ll32() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        view.reader.line_width = 32;
        let constraint = Vec2::new(80, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 32);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(47, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(61, 1));
        assert_eq!(view.visual_column_size, Vec2::new(18, 21));
    }

    #[test]
    fn layout_w79_h24_ll16() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        let constraint = Vec2::new(79, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(12, 1));
        assert_eq!(view.hex_column_size, Vec2::new(47, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(60, 1));
        assert_eq!(view.visual_column_size, Vec2::new(18, 21));
    }

    #[test]
    fn layout_w78_h24_ll16() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        let constraint = Vec2::new(78, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (15, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(45, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(59, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
    }

    #[test]
    fn layout_w77_h24_ll16() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        let constraint = Vec2::new(77, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (15, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(45, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(59, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
    }

    #[test]
    fn layout_w77_h24_ll16_offset1() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        let constraint = Vec2::new(77, 23);
        view.reader.window_pos = (1, 0);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 16);
        assert_eq!(view.reader.window_pos, (1, 0));
        assert_eq!(view.reader.window_size, (15, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(45, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(59, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
    }

    #[test]
    fn layout_w82_h24_ll32() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        view.reader.line_width = 32;
        let constraint = Vec2::new(82, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 32);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(48, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(62, 1));
        assert_eq!(view.visual_column_size, Vec2::new(18, 21));

        // Moving the window one byte to the right, should keep the rendering dimensions the same.
        view.navigate((1, 0));
        let constraint = Vec2::new(82, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_width, 32);
        assert_eq!(view.reader.window_pos, (1, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(48, 21));

        assert_eq!(view.visual_column_pos, Vec2::new(62, 1));
        assert_eq!(view.visual_column_size, Vec2::new(18, 21));
    }
}
