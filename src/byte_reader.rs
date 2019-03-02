use std::path::Path;
use std::io::Result;
use std::fs::File;
use std::io::SeekFrom;
use std::io::Seek;
use std::thread::Builder;
use std::sync::mpsc::sync_channel;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use cursive::rect::Rect;
use std::io::Read;

pub struct TilingByteReader {
    file: File,
    length: u64
}

pub struct Segment {
    offset: u64,
    usage_counter: u32,
    data: [u8; 512]
}

struct IoRequest {
    offset: u64,
//    result: Promise<Box<Segment>>
}

pub type Window = (u64, u64, u16, u16);
//pub struct Window {
//    pub x: u64,
//    pub y: u64,
//    pub w: u16,
//    pub h: u16
//}
//
//impl<T> From<(T,T,T,T)> for Window {
//    fn from((x, y, w, h): (T,T,T,T)) -> Self {
//        Window { x, y, w, h }
//    }
//}

impl TilingByteReader {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<TilingByteReader> {
        let mut f = File::open(file)?;
        let file_len = f.metadata()?.len();

        let cache = Arc::new(Mutex::new(BTreeMap::new()));
        let (segments_in, segments_out) = sync_channel::<Box<Segment>>(10);
        let (ios_in, ios_out) = sync_channel::<IoRequest>(100);

        let fetcher_cache = Arc::clone(&cache);
        let io_fetcher = Builder::new()
            .name("XV IO Fetcher Thread".to_owned())
            .spawn(move || run_fetcher(fetcher_cache, ios_out, segments_out))?;

        let evicter_cache = Arc::clone(&cache);
        let cache_evicter = Builder::new()
            .name("XV Cache Evicter".to_owned())
            .spawn(move || run_evicter(evicter_cache, segments_in))?;

        Ok(TilingByteReader{file: f, length: file_len})
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
        let (x, y, w, h) = window.into();
        let mut read_buf = vec![0; w as usize];

        for i in y..(y + (h as u64)) {
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
}

fn run_fetcher(cache: Arc<Mutex<BTreeMap<u64,Box<Segment>>>>, ios_out: Receiver<IoRequest>, segments_out: Receiver<Box<Segment>>) {

}

fn run_evicter(cache: Arc<Mutex<BTreeMap<u64,Box<Segment>>>>, segments_in: SyncSender<Box<Segment>>) {

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile;
    
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
        reader.get_window((0,0,4,2), 8, &mut buf);
        assert_eq!(buf, b"012389ab")
    }
    
    #[test]
    fn getting_multi_line_string_top_right() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((4,0,4,2), 8, &mut buf);
        assert_eq!(buf, b"4567cdef")
    }
    
    #[test]
    fn getting_multi_line_string_bottom_left() {
        let mut tmpf = tempfile::NamedTempFile::new().unwrap();
        tmpf.write(b"0123456789abcdef").unwrap();
        
        let mut reader = TilingByteReader::new(tmpf.path()).unwrap();
        let mut buf = Vec::new();
        reader.get_window((0,1,4,2), 8, &mut buf);
        assert_eq!(buf, b"89ab")
    }
}
