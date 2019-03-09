use std::io::Result;

use crate::byte_reader::TilingByteReader;

const UNICODE_TEXT_TABLE: &'static [&'static str; 256] = &[
    "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}", "\u{2406}", "\u{2407}",
    "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}", "\u{240C}", "\u{240D}", "\u{240E}", "\u{240F}",
    "\u{2410}", "\u{2411}", "\u{2412}", "\u{2413}", "\u{2414}", "\u{2415}", "\u{2416}", "\u{2417}",
    "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}", "\u{241C}", "\u{241D}", "\u{241E}", "\u{241F}",
    "\u{2423}", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?",
    "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_",
    "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~", "\u{2421}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}",
    "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}", "\u{FFFD}"];

const ASCII_TEXT_TABLE: &str = concat!(
    "................",
    "................",
    " !\"#$%&'()*+,-./",
    "0123456789:;<=>?",
    "@ABCDEFGHIJKLMNO",
    "PQRSTUVWXYZ[\\]^_",
    "`abcdefghijklmno",
    "pqrstuvwxyz{|}~.",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",);

const BYTE_RENDER: &'static [&'static str; 256] = &[
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d", "0e", "0f",
    "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "1e", "1f",
    "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f",
    "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f",
    "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f",
    "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f",
    "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "6e", "6f",
    "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "7c", "7d", "7e", "7f",
    "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f",
    "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f",
    "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af",
    "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf",
    "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc", "cd", "ce", "cf",
    "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd", "de", "df",
    "e0", "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "ea", "eb", "ec", "ed", "ee", "ef",
    "f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "fa", "fb", "fc", "fd", "fe", "ff"];

const BYTE_CATEGORY: &'static [ByteCategory; 256] = &[
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiControl,
    ByteCategory::AsciiControl, ByteCategory::AsciiControl, ByteCategory::AsciiWhitespace,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable, ByteCategory::AsciiPrintable,
    ByteCategory::AsciiPrintable, ByteCategory::AsciiControl, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other, ByteCategory::Other,
    ByteCategory::Other, ByteCategory::Other, ByteCategory::Other];

pub enum ByteCategory {
    AsciiControl,
    AsciiPrintable,
    AsciiWhitespace,
    Other,
}

pub enum VisualColumnWidth {
    Fixed(usize),
    PerByte(usize)
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
    offset: u64,
    pub line_length: u64,
    pub group: u16,
    pub window_pos: (u64,u64),
    pub window_size: (u16,u16),
    capture: Box<Vec<u8>>,
}

impl HexReader {
    pub fn new(reader: TilingByteReader) -> Result<HexReader> {
        Ok(HexReader {
            reader,
            offset: 0,
            line_length: 16,
            group: 8,
            window_pos: (0,0),
            window_size: (16,32),
            capture: Box::new(Vec::new())
        })
    }
    
    pub fn file_name(&self) -> &str {
        self.reader.file_name()
    }
    
    pub fn capture(&mut self) -> Result<()> {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        self.capture.clear();
        self.reader.get_window((x, y, w, h), self.line_length, &mut self.capture)
    }
    
    pub fn get_row_offsets_width(&self) -> usize {
        if self.reader.use_large_addresses() { 16 + 2 } else { 8 + 2 }
    }
    
    pub fn get_visual_data_width(&self) -> VisualColumnWidth {
        // Hard-coded for now, until we get more advanced visualisations.
        VisualColumnWidth::PerByte(1)
    }
    
    pub fn get_bytes_left_in_line(&self) -> u64 {
        let (x, y) = self.window_pos;
        self.line_length - x
    }
    
    pub fn visit_row_offsets(&self, visitor: &mut OffsetsVisitor) {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        let base_offset = y * self.line_length;
        let mut capture_height = self.capture.len() / w as usize;
        if capture_height * (w as usize) < self.capture.len() {
            capture_height += 1;
        }
        let height = (h as usize).min(capture_height);
        let mut bufout = String::with_capacity(self.get_row_offsets_width() * height);
        
        if self.reader.use_large_addresses() {
            for i in 0..height as u64 {
                let offset = base_offset + i * self.line_length;
                visitor.offset(&format!("0x{:016X}", offset));
            }
        } else {
            for i in 0..height as u64 {
                let offset = base_offset + i * self.line_length;
                visitor.offset(&format!("0x{:08X}", offset));
            }
        }
        visitor.end();
    }
    
    pub fn get_column_offsets(&mut self) -> String {
        unimplemented!() // todo
    }

    pub fn visit_hex(&self, visitor: &mut HexVisitor) {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        let cap = self.capture.as_slice();
        
        let mut i = 0;
        for b in cap {
            i += 1;
            let r = b.clone() as usize;
            visitor.byte(BYTE_RENDER[r], &BYTE_CATEGORY[r]);
            
            if i == w {
                visitor.next_line();
                i = 0;
            } else if (x + i as u64) % self.group as u64 == 0 {
                visitor.group();
            }
        }
        visitor.end();
    }
    
    pub fn visit_visual(&self, visitor: &mut VisualVisitor) {
        let (x, y) = self.window_pos;
        let (w, h) = self.window_size;
        let cap = self.capture.as_slice();

        let mut i = 0;
        for b in cap {
            i += 1;
            let r = b.clone() as usize;
            visitor.visual_element(UNICODE_TEXT_TABLE[r], &BYTE_CATEGORY[r]);

            if i == w {
                visitor.next_line();
                i = 0;
            } else if (x + i as u64) % self.group as u64 == 0 {
                visitor.group();
            }
        }
        visitor.end();
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
        fn byte(&mut self, byte: &str, category: &ByteCategory) {
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
        reader.line_length = 4;
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
        reader.line_length = 4;
        reader.capture();
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
        reader.line_length = 4;
        reader.capture();
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
