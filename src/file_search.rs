use std::collections::VecDeque;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{Result, Read, Seek, SeekFrom};

use bstr::Finder;

const BUFFER_SIZE: usize = 1024 * 1024;

#[cfg(target_os = "linux")]
pub fn search<F>(mut file: File, bytes: &[u8], mut consumer: F)
    where F: FnMut(u64) {
    if async_io_search(&mut file, &bytes, &mut consumer).is_err() {
        sync_io_search(&mut file, &bytes, &mut consumer);
    }
}

#[cfg(not(target_os = "linux"))]
pub fn search<F>(file: File, bytes: &[u8], mut consumer: F)
    where F: FnMut(u64) {
    sync_io_search(&mut file, &bytes, &mut consumer);
}

#[cfg(target_os = "linux")]
fn async_io_search<F>(file: &mut File, bytes: &[u8], consumer: &mut F) -> Result<()>
    where F: FnMut(u64) {
    let file_len = file.metadata()?.len();
    if file_len <= u64::try_from(BUFFER_SIZE).unwrap() {
        sync_io_search(file, bytes, consumer);
        return Ok(());
    }
    
    let finder = Finder::new(bytes);
    let needle_size = bytes.len();
    let queue_depth = 32;
    let mut read_pos = 0;
    let mut config = rio::Config::default();
    config.io_poll = true; // Use polled IO to make the kernel more proactive.
    let io = config.start()?;
    let buffers = vec![vec![0; BUFFER_SIZE]; queue_depth];
    
    let mut queue = VecDeque::with_capacity(queue_depth);
    for buf in &buffers {
        let cqe = io.read_at(file, buf, read_pos);
        queue.push_back((cqe, read_pos, buf));
        read_pos += u64::try_from(BUFFER_SIZE - needle_size + 1).unwrap();
    }

    while let Some((cqe, pos, buf)) = queue.pop_front() {
        let num_bytes = cqe.wait()?;
        if num_bytes > needle_size {
            let mut offset = 0;
            while let Some(p) = finder.find(&buf[offset..num_bytes]) {
                consumer(pos + u64::try_from(offset + p).unwrap());
                offset += p + 1;
            }
            let cqe = io.read_at(file, buf, read_pos);
            queue.push_back((cqe, read_pos, buf));
            read_pos += u64::try_from(BUFFER_SIZE - needle_size + 1).unwrap();
        }
    }
    
    Ok(())
}

fn sync_io_search<F>(file: &mut File, bytes: &[u8], consumer: &mut F)
    where F: FnMut(u64) {
    let finder = Finder::new(bytes);
    let needle_size = bytes.len();
    let mut buf = vec![0; BUFFER_SIZE];
    let mut num_bytes = file.read(&mut buf).unwrap();
    let mut pos = 0;
    
    while num_bytes > needle_size {
        let mut offset = 0;
        while let Some(p) = finder.find(&buf[offset..num_bytes]) {
            consumer(pos + u64::try_from(offset + p).unwrap());
            offset += p + 1;
        }
        pos += u64::try_from(num_bytes - needle_size + 1).unwrap();
        file.seek(SeekFrom::Start(pos)).unwrap();
        num_bytes = file.read(&mut buf).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Write};

    use super::*;

    #[test]
    fn searching_in_file() {
        let mut file = tempfile::tempfile().unwrap();
        file.write(b"ababa").unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let mut output: Vec<u64> = Vec::new();
        search(file, b"aba", |hit| output.push(hit));
        assert_eq!(output, vec![0, 2]);
    }
    
    #[test]
    fn sync_io_search_in_big_file() {
        let mut file = tempfile::tempfile().unwrap();
        prepare_big_file(&mut file);

        let mut counter: i64 = 0;
        sync_io_search(&mut file, b"Pokemon", &mut |_| counter += 1);

        assert_eq!(counter, 3);
    }
    
    #[cfg(target_os = "linux")]
    #[test]
    fn async_io_search_in_big_file() {
        let mut file = tempfile::tempfile().unwrap();
        prepare_big_file(&mut file);

        let mut counter: i64 = 0;
        if async_io_search(&mut file, b"Pokemon", &mut |_| counter += 1).is_ok() {
            // Only assert when no error happens.
            // We allow systems where io_uring is not available.
            assert_eq!(counter, 3);
        }
    }

    fn prepare_big_file(file: &mut File) {
        let file_len = u64::try_from(BUFFER_SIZE * 2 + (BUFFER_SIZE >> 1)).unwrap();
        file.set_len(file_len).unwrap();
        file.seek(SeekFrom::Start(u64::try_from(BUFFER_SIZE - 3).unwrap())).unwrap();
        file.write(b"Pokemon PokPokemon").unwrap();
        file.seek(SeekFrom::Start(file_len - 7)).unwrap();
        file.write(b"Pokemon").unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
    }
}
