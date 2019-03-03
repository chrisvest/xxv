use crate::hex_reader::HexReader;
use cursive::traits::View;
use cursive::Printer;
use cursive::event::Event;
use cursive::event::Key;
use cursive::event::EventResult;
use cursive::Vec2;
use cursive::views::TextView;
use crate::byte_reader::Window;
use cursive::views::Panel;
use cursive::align::HAlign;
use cursive::theme::ColorStyle;
use unicode_width::UnicodeWidthStr;
use crate::hex_reader::OffsetsVisitor;
use crate::hex_reader::HexVisitor;

pub struct HexView {
    reader: HexReader,
    invalidated: bool,
    show_data_view: bool,
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
            invalidated: true,
            show_data_view: true,
            offsets_column_width: 0,
            hex_column_width: 0,
            data_column_width: 0,
            last_drawn_window: (0, 0, 0, 0)
        }
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
    initial_offset: usize,
    max_width: usize,
    pos: Vec2,
    printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> HexVisitor for HexPrinter<'a, 'b, 'x> {
    fn byte(&mut self, byte: &str) {
        if self.pos.x != self.initial_offset {
            self.pos.x += 1;
        }
        self.printer.print(self.pos, byte);
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
        self.pos.x = self.initial_offset;
    }

    fn end(&mut self) {
        self.max_width = self.max_width.max(self.pos.x);
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
            initial_offset: hex_column_offset,
            max_width: 0,
            pos: Vec2::new(hex_column_offset, 1),
            printer
        };
        self.reader.visit_hex(&mut hex_printer);

        if self.show_data_view {
            let border_offset = hex_printer.max_width + 1;
            printer.print_vline(Vec2::new(border_offset, 1), printer.size.y - 2, "│");
        }
        // render data colunm
    }

    fn layout(&mut self, constraint: Vec2) {
        self.reader.capture();
        let offsets_colunm = self.reader.get_row_offsets_width();
        let borders = 4; // 1) left edge, 2) offset column to hex, 3) hex to data, 4) right edge.
        let available_data_width = constraint.x - offsets_colunm - borders;
        // The hex column takes up 3 characters for every 1 character in the data column.
        let hex_colunm = (available_data_width / 4) * 3;
        let data_column = (available_data_width / 4) * 1;
        let width = borders + offsets_colunm + hex_colunm + data_column;
        self.offsets_column_width = offsets_colunm;
        self.hex_column_width = hex_colunm;
        self.data_column_width = data_column;
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated
    }
    
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::WindowResize => {
                self.invalidated = true;
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
