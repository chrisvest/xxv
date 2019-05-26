use cursive::{Vec2, Printer};
use cursive::utils::span::{IndexedSpan, IndexedCow, SpannedStr};
use cursive::theme::{Style, ColorStyle};
use crate::hex_reader::{OffsetsVisitor, HexVisitor, Highlight};
use cursive::utils::markup::StyledString;
use unicode_width::UnicodeWidthStr;

const GROUP_SEP: &str = "\u{00A6}";

pub struct OffsetPrinter<'a, 'b, 'x> {
    pub pos: Vec2,
    pub printer: &'x Printer<'a, 'b>,
    pub spans: Vec<IndexedSpan<Style>>
}

impl<'a, 'b, 'x> OffsetsVisitor for OffsetPrinter<'a, 'b, 'x> {
    fn offset(&mut self, offset: &str) {
        if self.spans.is_empty() {
            self.spans.push(IndexedSpan {
                content: IndexedCow::Borrowed {start: 0, end: offset.len()},
                attr: Style::from(ColorStyle::secondary()),
                width: offset.width()
            });
        }
        let styled_offset = SpannedStr::new(offset, &self.spans);
        self.printer.print_styled(self.pos, styled_offset);
        self.pos.y += 1;
    }

    fn end(&mut self) {
        // Nothing to do.
    }
}

pub struct HexPrinter<'a, 'b, 'x> {
    pub max_width: usize,
    pub pos: Vec2,
    pub table_neu: &'x [StyledString],
    pub table_pos: &'x [StyledString],
    pub table_neg: &'x [StyledString],
    pub printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> HexVisitor for HexPrinter<'a, 'b, 'x> {
    fn byte(&mut self, index: usize, highlight: Highlight) {
        if self.pos.x != 0 {
            self.pos.x += 1;
        }
        let table = match highlight {
            Highlight::Neutral => self.table_neu,
            Highlight::Positive => self.table_pos,
            Highlight::Negative => self.table_neg,
        };
        let hex_element = &table[index];
        self.printer.print_styled(self.pos, hex_element.into());
        self.pos.x += 2;
    }

    fn group(&mut self) {
        self.printer.print(self.pos, GROUP_SEP);
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

pub struct VisualPrinter<'a, 'b, 'x> {
    pub pos: Vec2,
    pub table_neu: &'x [StyledString],
    pub table_pos: &'x [StyledString],
    pub table_neg: &'x [StyledString],
    pub printer: &'x Printer<'a, 'b>
}

impl<'a, 'b, 'x> HexVisitor for VisualPrinter<'a, 'b, 'x> {
    fn byte(&mut self, index: usize, highlight: Highlight) {
        let table = match highlight {
            Highlight::Neutral => self.table_neu,
            Highlight::Positive => self.table_pos,
            Highlight::Negative => self.table_neg,
        };
        let vis_element = &table[index];
        self.printer.print_styled(self.pos, vis_element.into());
        self.pos.x += vis_element.width();
    }

    fn group(&mut self) {
        self.printer.print(self.pos, GROUP_SEP);
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
