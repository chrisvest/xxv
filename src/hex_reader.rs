use std::io::Result;

use crate::byte_reader::TilingByteReader;
use crate::hex_tables::*;

pub enum VisualMode {
    Unicode,
    Ascii
}

pub trait OffsetsVisitor {
    fn offset(&mut self, offset: &str);
    
    fn end(&mut self);
}

pub trait HexVisitor {
    fn byte(&mut self, byte: &str, category: &ByteCategory);
    
    fn group(&mut self);
    
    fn next_line(&mut self);
    
    fn end(&mut self);
}

pub trait VisualVisitor {
    fn visual_element(&mut self, element: &str, category: &ByteCategory);
    
    fn group(&mut self);
    
    fn next_line(&mut self);
    
    fn end(&mut self);
}

pub struct HexReader {
    reader: TilingByteReader,
    pub line_width: u64,
    pub group: u16,
    pub window_pos: (u64,u64),
    pub window_size: (u16,u16),
    capture: Box<Vec<u8>>,
    vis_mode: VisualMode
}

impl HexReader {
    pub fn new(reader: TilingByteReader) -> Result<HexReader> {
        Ok(HexReader {
            reader,
            line_width: 16,
            group: 8,
            window_pos: (0,0),
            window_size: (16,32),
            capture: Box::new(Vec::new()),
            vis_mode: VisualMode::Unicode
        })
    }
    
    pub fn file_name(&self) -> &str {
        self.reader.file_name()
    }
    
    pub fn get_length(&self) -> u64 {
        self.reader.get_length()
    }
    
    pub fn capture(&mut self) -> Result<()> {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        self.capture.clear();
        // xxx Possible optimisation, since 'capture' is a Vec of u8 where drop is a no-op.
//        unsafe { self.capture.set_len(0) };
        self.reader.get_window((x, y, w, h), self.line_width, &mut self.capture)
    }
    
    pub fn get_row_offsets_width(&self) -> usize {
        if self.reader.use_large_addresses() { 16 + 2 } else { 8 + 2 }
    }
    
    pub fn get_bytes_left_in_line(&self) -> u64 {
        self.line_width - self.window_pos.0
    }
    
    pub fn visit_row_offsets(&self, visitor: &mut OffsetsVisitor) {
        let (w, h) = self.window_size;
        let base_offset = self.window_pos.1 * self.line_width;
        let mut capture_height = self.capture.len() / w as usize;
        if capture_height * (w as usize) < self.capture.len() {
            capture_height += 1;
        }
        let height = (h as usize).min(capture_height);
        
        if self.reader.use_large_addresses() {
            for i in 0..height as u64 {
                let offset = base_offset + i * self.line_width;
                visitor.offset(&format!("0x{:016X}", offset));
            }
        } else {
            for i in 0..height as u64 {
                let offset = base_offset + i * self.line_width;
                visitor.offset(&format!("0x{:08X}", offset));
            }
        }
        visitor.end();
    }
    
    pub fn visit_hex(&self, visitor: &mut HexVisitor) {
        let cap = self.capture.as_slice();
        
        let mut i = 0;
        for b in cap {
            i += 1;
            let r = b.clone() as usize;
            visitor.byte(BYTE_RENDER[r], &BYTE_CATEGORY[r]);
            
            if i == self.window_size.0 {
                visitor.next_line();
                i = 0;
            } else if (self.window_pos.0 + i as u64) % self.group as u64 == 0 {
                visitor.group();
            }
        }

        visitor.end();
    }
    
    pub fn visit_visual(&self, visitor: &mut VisualVisitor) {
        let cap = self.capture.as_slice();
        let table = self.vis_table();

        let mut i = 0;
        for b in cap {
            i += 1;
            let r = b.clone() as usize;
            visitor.visual_element(table[r], &BYTE_CATEGORY[r]);

            if i == self.window_size.0 {
                visitor.next_line();
                i = 0;
            } else if (self.window_pos.0 + i as u64) % self.group as u64 == 0 {
                visitor.group();
            }
        }

        visitor.end();
    }
    
    fn vis_table(&self) -> &'static [&'static str; 256] {
        match self.vis_mode {
            VisualMode::Unicode => UNICODE_TEXT_TABLE,
            VisualMode::Ascii => ASCII_TEXT_TABLE
        }
    }
    
    pub fn set_visual_mode(&mut self, mode: VisualMode) {
        self.vis_mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile;

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
        fn byte(&mut self, byte: &str, _category: &ByteCategory) {
            self.push_str(byte);
            self.push(' ');
        }

        fn group(&mut self) {
            // Nothing to do.
        }

        fn next_line(&mut self) {
            self.pop();
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
