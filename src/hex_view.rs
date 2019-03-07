use crate::hex_reader::{HexReader, VisualVisitor};
use cursive::traits::View;
use cursive::Printer;
use cursive::event::{Event, Key};
use cursive::event::EventResult;
use cursive::Vec2;
use crate::byte_reader::Window;
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
    offsets_column_pos: Vec2,
    offsets_column_size: Vec2,
    hex_column_pos: Vec2,
    hex_column_size: Vec2,
    visual_column_pos: Vec2,
    visual_column_size: Vec2,
    last_drawn_window: Window
}

impl HexView {
    pub fn new(reader: HexReader) -> HexView {
        let title = reader.file_name().to_owned();
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
            last_drawn_window: (0, 0, 0, 0)
        }
    }
    
    pub fn toggle_visual(&mut self) {
        self.show_visual_view = !self.show_visual_view;
        self.invalidated_resize = true;
    }
    
    fn on_char_event(&mut self, c: char) -> EventResult {
        match c {
            'j' => self.on_key_event(Key::Down),
            'k' => self.on_key_event(Key::Up),
            _ => EventResult::Ignored
        }
    }
    
    fn on_key_event(&mut self, k: Key) -> EventResult {
        let offset = match k {
            Key::Down => (0, 1),
            Key::Up => if self.reader.window_pos.1 > 0 { (0, -1) } else { (0, 0) },
            _ => (0, 0)
        };
        self.navigate(offset)
    }
    
    fn navigate(&mut self, offset: (i16, i16)) -> EventResult {
        if offset != (0, 0) {
            let (x, y) = offset;
            if x < 0 {
                self.reader.window_pos.0 -= (x * -1) as u64;
            } else {
                self.reader.window_pos.0 += x as u64;
            }
            if y < 0 {
                self.reader.window_pos.1 -= (y * -1) as u64;
            } else {
                self.reader.window_pos.1 += y as u64;
            }
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
        self.pos.x += 1;
        self.printer.print(self.pos, "┊");
        self.pos.x += 1;
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
            
            if self.show_visual_view {
                // Split the hex column into a smaller hex column, and a visual view.
                // We also reserve 3 characters of width for the spacer line between the two
                // columns.
                // The number of bytes that we display in the hex column should ideally be
                // reflected in the visual column.
                // Every byte in the hex view takes up 2 characters. Every pair of bytes take up
                // an additional 1 character space. There is an additional 2 characters, for a
                // total of 3 characters, taken up for the group spacer between byte pairs where
                // each byte is in a different group.
                // In the visual view, each byte take up (probably) one character, and group
                // separators also take up one character.
                // We compute how much space each column needs by iterating the remaining bytes in
                // the HexReader line, reducing the avail_width at each step, until we run out of
                // space. Then we count how many bytes that took, and how much space ended up being
                // taken by each column.
                // Note that we can use get_bytes_left_in_line because even though the HexReader
                // window size might change, we will not change the position.
                let avail_width = self.hex_column_size.x - 2;
                let mut space_left = avail_width as isize;
                let mut hex_width = 0;
                let mut vis_width = 0;
                let mut first_byte = true;
                let group = self.reader.group as u64;
                let bytes_left_in_line = self.reader.get_bytes_left_in_line();
                let reader_window_pos_x = self.reader.window_pos.0;

                for i in 0..bytes_left_in_line {
                    let last_byte = i == bytes_left_in_line - 1;
                    
                    if !first_byte && !last_byte {
                        // Subtract 1 for the space between byte pairs.
                        space_left -= 1;
                        hex_width += 1;
                    }
                    first_byte = false;
                    
                    if ((reader_window_pos_x + i) % group) == 0 && !last_byte {
                        // There is a group separator here.
                        // Subtract another 3 for the group separators.
                        space_left -= 3;
                        hex_width += 2;
                        vis_width += 1;
                    }
                    
                    // Finally subtract space for actually displaying the byte.
                    space_left -= 3;
                    hex_width += 2;
                    vis_width += 1;
                    if space_left < 0 {
                        break;
                    }
                }
                
                // The sizes we computed might be slightly too large, so we truncate the views to
                // fit our constraints.
                if hex_width + vis_width > avail_width {
                    let oversize = (hex_width + vis_width) - avail_width;
                    hex_width -= oversize;
                }
                
                self.hex_column_size = Vec2::new(hex_width, inner_height);
                self.visual_column_pos = Vec2::new(hex_col_start + hex_width + 2, 1);
                self.visual_column_size = Vec2::new(vis_width, inner_height);
            }
            self.invalidated_resize = false;
        }
        if self.invalidated_data_changed {
            // The viewing area was moved.
            self.reader.capture().unwrap();
            self.invalidated_data_changed = false;
        }
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated_resize || self.invalidated_data_changed
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

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint // Always take up all the space we can get.
    }
}
