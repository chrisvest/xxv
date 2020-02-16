use std::convert::TryFrom;
use std::io::Result;
use std::path::PathBuf;

use crate::byte_reader::TilingByteReader;
use crate::hex_tables::*;
use std::collections::btree_map::{BTreeMap, Range};
use crate::hex_view_printers::TableSet;
use crate::file_search;

#[derive(Copy, Clone, Debug)]
pub enum VisualMode {
    Unicode,
    Ascii,
    Off,
}

#[derive(Copy, Clone, Debug)]
pub enum Highlight {
    Neutral,
    Positive,
    Negative,
}

pub trait OffsetsVisitor {
    fn offset(&mut self, offset: &str);
    
    fn end(&mut self);
}

pub trait HexVisitor {
    fn byte(&mut self, index: usize, highlight: Highlight);
    
    fn group(&mut self);
    
    fn next_line(&mut self);
    
    fn end(&mut self);
}

#[derive(Debug)]
struct Highlights {
    highlight: BTreeMap<u64,(u64,Highlight)>,
    highlight_width: u64,
}

impl Highlights {
    fn new() -> Highlights {
        Highlights {
            highlight: BTreeMap::new(),
            highlight_width: 0,
        }
    }
    
    fn clear(&mut self) {
        self.highlight.clear();
        self.highlight_width = 0;
    }
    
    fn insert(&mut self, offset: u64, width: u64, highlight: Highlight) {
        self.highlight.insert(offset, (width, highlight));
        if self.highlight_width < width {
            self.highlight_width = width;
        }
    }
    
    fn iter_from(&self, offset: u64) -> Range<u64, (u64, Highlight)> {
        let start = if offset > self.highlight_width {
            offset - self.highlight_width
        } else {
            0
        };
        self.highlight.range(start..)
    }
}

#[derive(Debug)]
pub struct HexReader {
    reader: TilingByteReader,
    pub line_width: u64,
    pub group: u16,
    pub window_pos: (u64,u64),
    pub window_size: (u16,u16),
    capture: Vec<u8>,
    before_image: Vec<u8>,
    highlight: Highlights,
    pub vis_mode: VisualMode,
}

impl HexReader {
    pub fn new(reader: TilingByteReader) -> Result<HexReader> {
        Ok(HexReader {
            reader,
            line_width: 16,
            group: 8,
            window_pos: (0,0),
            window_size: (16,32),
            capture: Vec::new(),
            before_image: Vec::new(),
            highlight: Highlights::new(),
            vis_mode: VisualMode::Unicode
        })
    }
    
    pub fn reopen(&mut self) -> Result<()> {
        self.reader.reopen()
    }
    
    pub fn file_name(&self) -> &str {
        self.reader.file_name()
    }
    
    pub fn get_path(&self) -> PathBuf {
        self.reader.get_path_clone()
    }
    
    pub fn get_length(&self) -> u64 {
        self.reader.get_length()
    }

    pub fn get_row_offsets_width(&self) -> usize {
        if self.reader.use_large_addresses() { 16 + 2 } else { 8 + 2 }
    }

    pub fn get_lines_in_file(&self) -> u64 {
        self.reader.get_length() / self.line_width
    }
    
    pub fn capture(&mut self) -> Result<()> {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        self.capture.clear();
        self.reader.get_window((x, y, w, h), self.line_width, &mut self.capture)?;

        if !self.before_image.is_empty() {
            self.compute_window_diff();
            self.before_image.clear();
        }

        Ok(())
    }
    
    fn compute_window_diff(&mut self) {
        let line_cap = u64::from(self.window_size.0);
        let line_width = self.line_width;
        let mut line_offset = line_width * self.window_pos.1 + self.window_pos.0;
        
        let mut before_itr = self.before_image.iter();
        let mut after_itr = self.capture.iter();
        let mut i = 0;
        let mut begin_offset: Option<u64> = None;
        while let Some(a) = after_itr.next() {
            let offset = line_offset + i;
            
            if let Some(b) = before_itr.next() {
                if a != b && begin_offset.is_none() {
                    begin_offset = Some(offset);
                } else if let Some(begin) = begin_offset {
                    self.highlight.insert(begin, offset - begin, Highlight::Negative);
                }
            } else {
                if let Some(begin) = begin_offset {
                    self.highlight.insert(begin, offset - begin, Highlight::Negative);
                }
                break;
            }
            
            i += 1;
            if i == line_cap {
                i = 0;
                line_offset += line_width;
            }
        }
    }
    
    pub fn capture_before_image(&mut self) {
        self.before_image.clear();
        self.before_image.clone_from(&self.capture);
    }
    
    pub fn clear_highlights(&mut self) {
        self.highlight.clear();
    }
    
    pub fn highlight(&mut self, offset: u64, width: u64, highlight: Highlight) {
        self.highlight.insert(offset, width, highlight);
    }
    
    pub fn visit_row_offsets(&self, visitor: &mut dyn OffsetsVisitor) {
        let w = usize::from(self.window_size.0);
        let h = usize::from(self.window_size.1);
        let base_offset = self.window_pos.1 * self.line_width;
        let mut capture_height = self.capture.len() / w;
        if capture_height * w < self.capture.len() {
            capture_height += 1;
        }
        let height = u64::try_from(h.min(capture_height)).unwrap();
        
        if self.reader.use_large_addresses() {
            for i in 0..height {
                let offset = base_offset + i * self.line_width;
                visitor.offset(&format!("0x{:016X}", offset));
            }
        } else {
            for i in 0..height {
                let offset = base_offset + i * self.line_width;
                visitor.offset(&format!("0x{:08X}", offset));
            }
        }
        visitor.end();
    }
    
    #[inline]
    pub fn visit_hex(&self, visitor: &mut dyn HexVisitor) {
        let capture = self.capture.as_slice();
        let line_cap = u64::from(self.window_size.0);
        let line_width = self.line_width;
        let group = u64::from(self.group);

        let mut line_offset = line_width * self.window_pos.1 + self.window_pos.0;
        let mut hl_iter = self.highlight.iter_from(line_offset);
        let mut hl = hl_iter.next();

        let mut i = 0;
        let mut highlight = Highlight::Neutral;
        let mut hl_end = 0;
        for b in capture {
            let offset = line_offset + i;

            while let Some((&start, &(w,h))) = hl {
                if start + w < offset {
                    hl = hl_iter.next();
                } else if start <= offset && offset < start + w {
                    hl = hl_iter.next();
                    hl_end = w + i - (offset - start);
                    highlight = h;
                    break;
                } else {
                    break;
                }
            }

            let r = usize::from(*b);
            visitor.byte(r, highlight);

            i += 1;
            if i == hl_end {
                highlight = Highlight::Neutral;
            }
            if i == line_cap {
                visitor.next_line();
                i = 0;
                line_offset += line_width;
                highlight = Highlight::Neutral;
            } else if (self.window_pos.0 + i) % group == 0 {
                visitor.group();
            }
        }

        visitor.end();
    }
    
    fn vis_table(&self) -> &'static [&'static str; 256] {
        match self.vis_mode {
            VisualMode::Unicode => UNICODE_TEXT_TABLE,
            VisualMode::Ascii => ASCII_TEXT_TABLE,
            VisualMode::Off => ASCII_TEXT_TABLE
        }
    }

    pub fn generate_hex_tables(&self, table_set: &mut TableSet) {
        for i in 0..BYTE_RENDER.len() {
            table_set.push_byte(&BYTE_CATEGORY[i], BYTE_RENDER[i]);
        }
    }

    pub fn generate_visual_tables(&self, table_set: &mut TableSet) {
        let table = self.vis_table();
        for i in 0..BYTE_RENDER.len() {
            table_set.push_byte(&BYTE_CATEGORY[i], table[i]);
        }
    }
    
    pub fn set_visual_mode(&mut self, mode: VisualMode) {
        self.vis_mode = mode;
    }
    
    pub fn get_visual_mode(&self) -> &VisualMode {
        &self.vis_mode
    }
    
    pub fn search(&mut self, bytes: &[u8]) {
        let file = self.reader.open_file().unwrap();
        let len = u64::try_from(bytes.len()).unwrap();
        file_search::search(file, bytes, |start| {
            self.highlight(start, len, Highlight::Positive);
        });
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile;

    use super::*;

    impl OffsetsVisitor for String {
        fn offset(&mut self, offset: &str) {
            self.push_str(offset);
            self.push('\n');
        }

        fn end(&mut self) {
            self.pop();
        }
    }
    
    impl HexVisitor for String {
        fn byte(&mut self, index: usize, highlight: Highlight) {
            self.push_str(BYTE_RENDER[index]);
            match highlight {
                Highlight::Positive => self.push( '+' ),
                Highlight::Negative => self.push( '-' ),
                _ => (),
            }
            self.push(' ');
        }

        fn group(&mut self) {
            // Nothing to do.
        }

        fn next_line(&mut self) {
            let ch = self.pop();
            if let Some(c) = ch {
                if c == '+' || c == '-' {
                    self.push(c);
                }
            }
            self.push('\n');
        }

        fn end(&mut self) {
            self.pop();
        }
    }
    
    #[test]
    fn getting_hex_of_file_top_left_window() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = HexReader::new(TilingByteReader::new(tmpf.path()).unwrap()).unwrap();
        reader.window_pos = (0,0);
        reader.window_size = (2,2);
        reader.line_width = 4;
        reader.capture().unwrap();
        let mut hex = String::new();
        reader.visit_hex(&mut hex);
        // Bytes:  Hex:
        //  01      30 31
        //  45      34 35
        assert_eq!(hex, "30 31\n34 35")
    }
    
    #[test]
    fn getting_hex_of_file_with_highlight() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = HexReader::new(TilingByteReader::new(tmpf.path()).unwrap()).unwrap();
        reader.highlight(1, 1, Highlight::Positive);
        reader.highlight(3, 1, Highlight::Positive);
        reader.highlight(4, 1, Highlight::Negative);
        reader.highlight(5, 1, Highlight::Negative);
        reader.window_pos = (0,0);
        reader.window_size = (2,2);
        reader.line_width = 4;
        reader.capture().unwrap();
        let mut hex = String::new();
        reader.visit_hex(&mut hex);
        // Bytes:  Hex:
        //  01      30 31
        //  45      34 35
        assert_eq!(hex, "30 31+\n34- 35-")
    }
    
    #[test]
    fn hex_view_bigger_than_file() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();

        let mut reader = HexReader::new(TilingByteReader::new(tmpf.path()).unwrap()).unwrap();
        reader.window_pos = (0,0);
        reader.window_size = (4,16);
        reader.line_width = 4;
        reader.capture().unwrap();
        let mut hex = String::new();
        reader.visit_hex(&mut hex);
        // Bytes:  Hex:
        //  0123    30 31 32 33
        //  4567    34 35 36 37
        //  89ab    38 39 61 62
        //  cdef    63 64 65 66
        assert_eq!(hex, "30 31 32 33\n34 35 36 37\n38 39 61 62\n63 64 65 66");
        let mut offsets = String::new();
        reader.visit_row_offsets(&mut offsets);
        assert_eq!(offsets, "0x00000000\n0x00000004\n0x00000008\n0x0000000C");
    }
    
    #[test]
    fn hex_view_bigger_than_unaligned_file() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcde").unwrap();

        let mut reader = HexReader::new(TilingByteReader::new(tmpf.path()).unwrap()).unwrap();
        reader.window_pos = (0,0);
        reader.window_size = (4,16);
        reader.line_width = 4;
        reader.capture().unwrap();
        let mut hex = String::new();
        reader.visit_hex(&mut hex);
        // Bytes:  Hex:
        //  0123    30 31 32 33
        //  4567    34 35 36 37
        //  89ab    38 39 61 62
        //  cdef    63 64 65 66
        assert_eq!(hex, "30 31 32 33\n34 35 36 37\n38 39 61 62\n63 64 65");
        let mut offsets = String::new();
        reader.visit_row_offsets(&mut offsets);
        assert_eq!(offsets, "0x00000000\n0x00000004\n0x00000008\n0x0000000C");
    }
}
