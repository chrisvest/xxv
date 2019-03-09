use crate::hex_reader::{HexReader, VisualVisitor, VisualMode};
use cursive::traits::View;
use cursive::Printer;
use cursive::event::{Event, Key};
use cursive::event::EventResult;
use cursive::Vec2;
use cursive::align::HAlign;
use cursive::theme::ColorStyle;
use unicode_width::UnicodeWidthStr;
use crate::hex_reader::OffsetsVisitor;
use crate::hex_reader::HexVisitor;
use crate::hex_reader::ByteCategory;

pub struct HexView {
    reader: HexReader,
    invalidated_resize: bool,
    invalidated_data_changed: bool,
    show_visual_view: bool,
    visual: Visual,
    offsets_column_pos: Vec2,
    offsets_column_size: Vec2,
    hex_column_pos: Vec2,
    hex_column_size: Vec2,
    visual_column_pos: Vec2,
    visual_column_size: Vec2,
}

impl HexView {
    pub fn new(reader: HexReader) -> HexView {
        let title = reader.file_name().to_owned();
        HexView {
            reader,
            invalidated_resize: true,
            invalidated_data_changed: true,
            show_visual_view: true,
            visual: Visual::Unicode,
            offsets_column_pos: Vec2::new(0, 0),
            offsets_column_size: Vec2::new(0, 0),
            hex_column_pos: Vec2::new(0, 0),
            hex_column_size: Vec2::new(0, 0),
            visual_column_pos: Vec2::new(0, 0),
            visual_column_size: Vec2::new(0, 0),
        }
    }
    
    fn toggle_visual(&mut self) -> EventResult {
        match self.visual {
            Visual::Unicode => {
                self.visual = Visual::Ascii;
                self.reader.set_visual_mode(VisualMode::Ascii);
            },
            Visual::Ascii => {
                self.visual = Visual::Off;
                self.show_visual_view = false;
                self.invalidated_resize = true;
            },
            Visual::Off => {
                self.visual = Visual::Unicode;
                self.reader.set_visual_mode(VisualMode::Unicode);
                self.show_visual_view = true;
                self.invalidated_resize = true;
            }
        }
        EventResult::Consumed(None)
    }
    
    fn on_char_event(&mut self, c: char) -> EventResult {
        match c {
            'j' => self.on_key_event(Key::Down),
            'k' => self.on_key_event(Key::Up),
            'h' => self.on_key_event(Key::Left),
            'l' => self.on_key_event(Key::Right),
            'v' => self.toggle_visual(),
            _ => EventResult::Ignored
        }
    }
    
    fn on_key_event(&mut self, k: Key) -> EventResult {
        let inner_height = self.offsets_column_size.y as i16;
        let (pos_x, pos_y) = self.reader.window_pos;
        let (size_x, _size_y) = self.reader.window_size;
        let offset = match k {
            Key::Down => (0, 1),
            Key::Up => if pos_y > 0 { (0, -1) } else { (0, 0) },
            Key::Left => if pos_x > 0 { (-1, 0) } else { (0, 0) }
            Key::Right => if pos_x + (size_x as u64) < self.reader.line_length { (1, 0) } else { (0, 0) },
            Key::PageDown => (0, inner_height),
            Key::PageUp => if pos_y > inner_height as u64 {
                (0, -inner_height)
            } else if pos_y > 0 {
                (0, -(pos_y as i16))
            } else {
                (0, 0)
            }
            _ => (0, 0)
        };
        self.navigate(offset)
    }
    
    fn navigate(&mut self, offset: (i16, i16)) -> EventResult {
        if offset != (0, 0) {
            let (x, y) = offset;
            if x < 0 {
                self.reader.window_pos.0 -= (-x) as u64;
            } else {
                self.reader.window_pos.0 += x as u64;
            }
            if y < 0 {
                self.reader.window_pos.1 -= (-y) as u64;
            } else {
                self.reader.window_pos.1 += y as u64;
            }
            self.invalidated_resize = true;
            self.invalidated_data_changed = true;
            EventResult::Consumed(None)
        } else {
            EventResult::Ignored
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
}

impl View for HexView {
    fn draw(&self, printer: &Printer) {
        printer.print_box((0, 0), printer.size, true);
        self.draw_title(printer);
        
        let mut offset_printer = OffsetPrinter {
            pos: Vec2::new(0, 0),
            printer: &printer.offset(self.offsets_column_pos).cropped(self.offsets_column_size)
        };
        self.reader.visit_row_offsets(&mut offset_printer);
        
        let inner_height = self.offsets_column_size.y;
        let border_offset = self.offsets_column_size.x + self.offsets_column_pos.x;
        printer.print_vline(Vec2::new(border_offset, 1), inner_height, "│");
        
        let mut hex_printer = HexPrinter {
            max_width: 0,
            pos: Vec2::new(0, 0),
            printer: &printer.offset(self.hex_column_pos).cropped(self.hex_column_size)
        };
        self.reader.visit_hex(&mut hex_printer);

        if self.show_visual_view {
            let border_offset = self.hex_column_pos.x + self.hex_column_size.x;
            printer.print_vline(Vec2::new(border_offset, 1), inner_height, "│");
            
            let mut visual_printer = VisualPrinter {
                pos: Vec2::new(0,0),
                printer: &printer.offset(self.visual_column_pos).cropped(self.visual_column_size)
            };
            self.reader.visit_visual(&mut visual_printer);
        }
    }

    fn layout(&mut self, constraint: Vec2) {
        if self.invalidated_resize {
            // The viewing area changed size, or the visual column was toggled.

            // The available height inside the box border:
            let inner_height = constraint.y - 2;
            if self.reader.window_size.1 != (inner_height as u16) {
                self.reader.window_size.1 = inner_height as u16;
                self.invalidated_data_changed = true;
            }

            let colw_offsets = self.reader.get_row_offsets_width();
            self.offsets_column_pos = Vec2::new(1, 1);
            self.offsets_column_size = Vec2::new(colw_offsets, inner_height);
            
            // Box-border, offsets column, separator line + space line:
            let hex_col_start = 1 + colw_offsets + 2;
            self.hex_column_pos = Vec2::new(hex_col_start, 1);
            self.hex_column_size = Vec2::new(constraint.x - hex_col_start - 1, inner_height);

            let reader_pos_x = self.reader.window_pos.0;
            let group = self.reader.group as u64;
            let vis_group_spacer: usize = if self.show_visual_view { 1 } else { 0 };
            let vis_byte_width: usize = if self.show_visual_view { 1 } else { 0 };
            
            let avail_width = self.hex_column_size.x;
            let mut space_left = avail_width as isize;
            let mut hex_width: usize = 0;
            let mut vis_width: usize = 0;
            let mut bytes_consumed = 0;
            let bytes_left_in_line = self.reader.get_bytes_left_in_line();

            for i in 0..bytes_left_in_line {
                let byte_pair_spacer = if i == 0 { 0 } else { 1 };
                let consumed_by_byte = byte_pair_spacer + 2 + vis_byte_width as isize;
                if space_left - consumed_by_byte >= 0 {
                    space_left -= consumed_by_byte;
                    hex_width += 2 + byte_pair_spacer as usize;
                    vis_width += vis_byte_width;
                    bytes_consumed += 1;
                    
                    if ((reader_pos_x + i) % group) == 0 && i != 0 {
                        // The hex column group spacer replaces the byte pair spacer automatically.
                        let consumed_by_group = vis_group_spacer as isize;
                        if space_left - consumed_by_group > 0 {
                            space_left -= consumed_by_group;
                            vis_width += vis_group_spacer;
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            
            if hex_width + vis_width + 1 < avail_width {
                // Add right padding to hex column.
                hex_width += 1;
            } else if hex_width + vis_width == avail_width {
                // Remove the left hex column padding to squeeze in the last byte.
                self.hex_column_pos.x -= 1;
            }
            
            self.hex_column_size = Vec2::new(hex_width, inner_height);
            self.visual_column_pos = Vec2::new(self.hex_column_pos.x + hex_width + 1, 1);
            self.visual_column_size = Vec2::new(vis_width, inner_height);
            
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
            },
            Event::Char(c) => self.on_char_event(c),
            Event::Key(k) => self.on_key_event(k),
            _ => EventResult::Ignored
        }
    }
}

enum Visual {
    Unicode,
    Ascii,
    Off
}

struct OffsetPrinter<'a, 'b, 'x> {
    pos: Vec2,
    printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> OffsetsVisitor for OffsetPrinter<'a, 'b, 'x> {
    fn offset(&mut self, offset: &str) {
        self.printer.with_color(ColorStyle::secondary(), |p| {
            p.print(self.pos, offset);
        });
        self.pos.y += 1;
    }

    fn end(&mut self) {
        // Nothing to do.
    }
}

struct HexPrinter<'a, 'b, 'x> {
    max_width: usize,
    pos: Vec2,
    printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> HexVisitor for HexPrinter<'a, 'b, 'x> {
    fn byte(&mut self, byte: &str, category: &ByteCategory) {
        if self.pos.x != 0 {
            self.pos.x += 1;
        }
        let color = category_to_color(category);
        self.printer.with_color(color, |p| {
            p.print(self.pos, byte);
        });
        self.pos.x += 2;
    }

    fn group(&mut self) {
        self.printer.print(self.pos, "┊");
    }

    fn next_line(&mut self) {
        self.pos.y += 1;
        self.max_width = self.max_width.max(self.pos.x);
        self.pos.x = 0;
    }

    fn end(&mut self) {
        self.max_width = self.max_width.max(self.pos.x);
    }
}

struct VisualPrinter<'a, 'b, 'x> {
    pos: Vec2,
    printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> VisualVisitor for VisualPrinter<'a, 'b, 'x> {
    fn visual_element(&mut self, element: &str, category: &ByteCategory) {
        let color = category_to_color(category);
        self.printer.with_color(color, |p| {
            p.print(self.pos, element);
        });
        self.pos.x += element.width();
    }

    fn group(&mut self) {
        self.printer.print(self.pos, "┊");
        self.pos.x += 1;
    }

    fn next_line(&mut self) {
        self.pos.y += 1;
        self.pos.x = 0;
    }

    fn end(&mut self) {
        // Nothing to do.
    }
}

fn category_to_color(category: &ByteCategory) -> ColorStyle {
    match category {
        ByteCategory::AsciiControl => ColorStyle::title_primary(),
        ByteCategory::AsciiPrintable => ColorStyle::primary(),
        ByteCategory::AsciiWhitespace => ColorStyle::secondary(),
        ByteCategory::Other => ColorStyle::title_secondary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use crate::byte_reader::TilingByteReader;

    #[test]
    fn layout_w80_h24_ll16() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        let constraint = Vec2::new(80, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_length, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(48, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(62, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
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

        assert_eq!(view.reader.line_length, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(47, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(61, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
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

        assert_eq!(view.reader.line_length, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(12, 1));
        assert_eq!(view.hex_column_size, Vec2::new(47, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(60, 1));
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

        assert_eq!(view.reader.line_length, 16);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (15, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(45, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(59, 1));
        assert_eq!(view.visual_column_size, Vec2::new(16, 21));
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

        assert_eq!(view.reader.line_length, 16);
        assert_eq!(view.reader.window_pos, (1, 0));
        assert_eq!(view.reader.window_size, (15, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(45, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(59, 1));
        assert_eq!(view.visual_column_size, Vec2::new(16, 21));
    }

    #[test]
    fn layout_w80_h24_ll32() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef0123456789abcdef").unwrap();

        let byte_reader = TilingByteReader::new(tmpf.path()).unwrap();
        let hex_reader = HexReader::new(byte_reader).unwrap();
        let mut view = HexView::new(hex_reader);

        view.reader.line_length = 32;
        let constraint = Vec2::new(80, 23);
        view.layout(constraint);

        assert_eq!(view.reader.line_length, 32);
        assert_eq!(view.reader.window_pos, (0, 0));
        assert_eq!(view.reader.window_size, (16, 21));

        assert_eq!(view.offsets_column_pos, Vec2::new(1, 1));
        assert_eq!(view.offsets_column_size, Vec2::new(10, 21));

        assert_eq!(view.hex_column_pos, Vec2::new(13, 1));
        assert_eq!(view.hex_column_size, Vec2::new(48, 21));
        
        assert_eq!(view.visual_column_pos, Vec2::new(62, 1));
        assert_eq!(view.visual_column_size, Vec2::new(17, 21));
    }
}