use crate::hex_reader::{HexReader, VisualVisitor, VisualColumnWidth};
use cursive::traits::View;
use cursive::Printer;
use cursive::event::Event;
use cursive::event::Key;
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
    offsets_column_width: usize,
    hex_column_width: usize,
    data_column_width: usize,
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
            offsets_column_width: 0,
            hex_column_width: 0,
            data_column_width: 0,
            last_drawn_window: (0, 0, 0, 0)
        }
    }
    
    pub fn toggle_visual(&mut self) {
        self.show_visual_view = !self.show_visual_view;
    }

    fn on_key_event(&mut self, key: Key) -> EventResult {
        EventResult::Ignored
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
        
        let offset_column_width = self.reader.get_row_offsets_width();
        let mut offset_printer = OffsetPrinter {
            pos: Vec2::new(1, 1),
            printer
        };
        self.reader.visit_row_offsets(&mut offset_printer);
        
        let border_offset = offset_column_width + 1;
        printer.print_vline(Vec2::new(border_offset, 1), printer.size.y - 2, "│");
        
        let hex_column_offset = border_offset + 2;
        let mut hex_printer = HexPrinter {
            max_width: 0,
            pos: Vec2::new(0, 0),
            printer: &printer.offset((hex_column_offset,1)).shrinked((0,1))
        };
        self.reader.visit_hex(&mut hex_printer);

        if self.show_visual_view {
            let border_offset = hex_printer.max_width + 1 + hex_column_offset;
            printer.print_vline(Vec2::new(border_offset, 1), printer.size.y - 2, "│");
            
            let visual_column_offset = border_offset + 2;
            let p = printer.offset((visual_column_offset, 1)).shrinked((1, 1));

            let mut visual_printer = VisualPrinter {
                pos: Vec2::new(0,0),
                printer: &p
            };
            self.reader.visit_visual(&mut visual_printer);
        }
    }

    fn layout(&mut self, constraint: Vec2) {
        if self.invalidated_resize {
            // The viewing area changed size, or the visual column was toggled.
            let offsets_colunm = self.reader.get_row_offsets_width();
            // Borders: 1) left edge, 2) offset column to hex, 3) hex to data, 4) right edge.
            let borders = 4;
            // Spaces: 2 on the inside of the hex column. If we show the visual column, 2 on the
            // inside of the visual column as well. 
            let spaces = 2 + (if self.show_visual_view { 2 } else { 0 });
            // Area available for data, after we've taking all the visual elements into account.
            let available_width = (constraint.x - offsets_colunm - borders - spaces) as isize;
            
            if self.show_visual_view {
                // todo
            } else {
                // We are not showing the visual column, so the available data can be dedicated to
                // the hex column.
                // In the hex column, every group takes up 3 characters. Every byte takes up 2
                // characters. And between every pair of bytes not separated by a group, there is
                // a space taking up one character.
                let group = self.reader.group as u64;
                let bytes_left_in_line = self.reader.get_bytes_left_in_line();
                let (wx,wy) = self.reader.window_pos;
                let mut space_left = available_width;
                let mut bytes_fitted = 0;
                for i in 0..bytes_left_in_line {
                    if space_left != available_width {
                        // We are not the first byte, so we subtract 1 for the space between bytes.
                        space_left -= 1;
                    }
                    if ((wx + i) % group) == 0 {
                        // There is a group separator here. Subtract another 2.
                        space_left -= 2;
                    }
                    // Finally subtract 2 for the byte itself.
                    space_left -= 2;
                    if space_left > 0 {
                        bytes_fitted += 1;
                    } else {
                        break;
                    }
                }
                // Now bytes fitted is the number of bytes we can display.
                let new_window_size = (bytes_fitted, constraint.y as u16);
                self.invalidated_data_changed = new_window_size != self.reader.window_size;
                self.reader.window_size = new_window_size;
            }

//            let visual_data_width = if self.show_data_view {
//                self.reader.get_visual_data_width()
//            } else {
//                VisualColumnWidth::Fixed(0)
//            };
//
//            // The hex column takes up 3 characters for every 1 character in the data column.
//            let hex_colunm = (available_width / 4) * 3;
//            let data_column = (available_width / 4) * 1;
//            let width = borders + offsets_colunm + hex_colunm + data_column;
//            self.offsets_column_width = offsets_colunm;
//            self.hex_column_width = hex_colunm;
//            self.data_column_width = data_column;
            self.invalidated_resize = false;
        }
        if self.invalidated_data_changed {
            // The viewing area was moved.
            self.reader.capture();
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
            Event::Key(k) => self.on_key_event(k),
            _ => EventResult::Ignored
        }
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint // Always take up all the space we can get.
    }
}
