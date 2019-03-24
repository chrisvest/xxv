use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TilingByteReader {
    file: File,
    path: PathBuf,
    length: u64,
    use_large_addresses: bool,
    display_name: String
}

pub type Window = (u64, u64, u16, u16);

impl TilingByteReader {
    pub fn new<P: AsRef<Path>>(file_name: P) -> Result<TilingByteReader> {
        let path_ref = file_name.as_ref();
        let path_buf = PathBuf::from(path_ref);
        let display_name: String = path_ref.file_name().unwrap().to_string_lossy().into();
        let file = File::open(file_name)?;
        let file_len = file.metadata()?.len();

        Ok(TilingByteReader {
            file,
            path: path_buf,
            length: file_len,
            use_large_addresses: file_len > u64::from(std::u32::MAX),
            display_name
        })
    }
    
    pub fn file_name(&self) -> &str {
        &self.display_name
    }
    
    pub fn get_path_clone(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn get_window(&mut self, window: Window, line_length: u64, buf: &mut Vec<u8>) -> Result<()> {
        // The binary file is viewed in terms of lines.
        // The lines turn the linear byte sequence into a 2D byte grid.
        // Each line may be significantly longer than what can fit in the window.
        // The window is the rectangular viewport that is projected onto the grid.
        // We are only interested in the bytes that fall within the window.
        // The 'y' coordinate of the window directly corresponds to the first line in the window.
        // The 'x' coordinate is the offset into each line, where the left-most window edge starts.
        // The 'h' height is the number of lines in the window,
        // and 'w' is the width of each window line.
        let (x, y, w, h) = window;
        let mut read_buf = vec![0; w as usize];

        for i in y..(y + (u64::from(h))) {
            let offset = line_length * i + x;
            self.file.seek(SeekFrom::Start(offset))?;
            let bytes_read = self.file.read(&mut read_buf)?;
            buf.extend(&read_buf[0..bytes_read]);
        }
        Ok(())
    }
    
    pub fn get_length(&self) -> u64 {
        self.length
    }
    
    pub fn use_large_addresses(&self) -> bool {
        self.use_large_addresses
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile;

    use super::*;

    #[test]
    fn getting_top_left_window() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"01234567").unwrap();

        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((0,0,16,16).into(), 16, &mut buf).unwrap();
        assert_eq!(buf, b"01234567")
    }
    
    #[test]
    fn getting_multi_line_string_top_left() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((0,0,4,2), 8, &mut buf).unwrap();
        assert_eq!(buf, b"012389ab")
    }
    
    #[test]
    fn getting_multi_line_string_top_right() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((4,0,4,2), 8, &mut buf).unwrap();
        assert_eq!(buf, b"4567cdef")
    }
    
    #[test]
    fn getting_multi_line_string_bottom_left() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((0,1,4,2), 8, &mut buf).unwrap();
        assert_eq!(buf, b"89ab")
    }
}
