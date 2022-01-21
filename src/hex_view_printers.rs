use cursive::theme::{ColorStyle, Style};
use cursive::utils::markup::StyledString;
use cursive::utils::span::{IndexedCow, IndexedSpan, SpannedStr};
use cursive::{Printer, Vec2};
use unicode_width::UnicodeWidthStr;

use crate::hex_reader::{HexVisitor, Highlight, OffsetsVisitor};
use crate::hex_tables::ByteCategory;

const GROUP_SEP: &str = "\u{00A6}";

pub struct OffsetPrinter<'a, 'b, 'x> {
    pub pos: Vec2,
    pub printer: &'x Printer<'a, 'b>,
    pub spans: Vec<IndexedSpan<Style>>,
}

impl<'a, 'b, 'x> OffsetsVisitor for OffsetPrinter<'a, 'b, 'x> {
    fn offset(&mut self, offset: &str) {
        if self.spans.is_empty() {
            self.spans.push(IndexedSpan {
                content: IndexedCow::Borrowed {
                    start: 0,
                    end: offset.len(),
                },
                attr: Style::from(ColorStyle::secondary()),
                width: offset.width(),
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

pub struct TableSet {
    pub neu: Vec<StyledString>,
    pub pos: Vec<StyledString>,
    pub neg: Vec<StyledString>,
}

impl TableSet {
    pub fn new() -> TableSet {
        TableSet {
            neu: Vec::new(),
            pos: Vec::new(),
            neg: Vec::new(),
        }
    }

    pub fn push_byte(&mut self, category: &ByteCategory, s: &'static str) {
        self.neu
            .push(StyledString::styled(s, category_to_color(category)));
        self.pos
            .push(StyledString::styled(s, ColorStyle::highlight_inactive()));
        self.neg
            .push(StyledString::styled(s, ColorStyle::highlight()));
    }

    pub fn clear(&mut self) {
        self.neu.clear();
        self.pos.clear();
        self.neg.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.neu.is_empty()
    }
}

fn category_to_color(category: &ByteCategory) -> ColorStyle {
    match category {
        ByteCategory::AsciiControl => ColorStyle::title_primary(),
        ByteCategory::AsciiPrintable => ColorStyle::primary(),
        ByteCategory::AsciiWhitespace => ColorStyle::secondary(),
        ByteCategory::Other => ColorStyle::title_secondary(),
    }
}

pub struct HexPrinter<'a, 'b, 'x> {
    pub max_width: usize,
    pub pos: Vec2,
    pub printer: &'x Printer<'a, 'b>,
    pub tables: &'x TableSet,
}

impl<'a, 'b, 'x> HexVisitor for HexPrinter<'a, 'b, 'x> {
    fn byte(&mut self, index: usize, highlight: Highlight) {
        if self.pos.x != 0 {
            self.pos.x += 1;
        }
        let table = match highlight {
            Highlight::Neutral => &self.tables.neu,
            Highlight::Positive => &self.tables.pos,
            Highlight::Negative => &self.tables.neg,
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
    pub printer: &'x Printer<'a, 'b>,
    pub tables: &'x TableSet,
}

impl<'a, 'b, 'x> HexVisitor for VisualPrinter<'a, 'b, 'x> {
    fn byte(&mut self, index: usize, highlight: Highlight) {
        let table = match highlight {
            Highlight::Neutral => &self.tables.neu,
            Highlight::Positive => &self.tables.pos,
            Highlight::Negative => &self.tables.neg,
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
