use std::fs::File;
use std::ops::Range;
use std::convert::TryFrom;
use std::io::{BufReader, Read, Bytes, BufRead};

pub fn search<F>(file: File, bytes: &[u8], mut consumer: F)
    where F: FnMut(u64) {
    let mut table = vec![-1; bytes.len() + 1];
    build_table(bytes, &mut table);

    let capacity = 16 * 1024 * 1024;
    let mut reader = BufReader::with_capacity(capacity, file);
    let mut buffer = reader.fill_buf().unwrap();
    let mut buffer_pos = 0;
    let mut file_pos = 0;
    let mut needle_pos = 0;

    while buffer.len() > 0 {
        while buffer_pos < buffer.len() {
            while needle_pos > -1 && bytes[to_usize(needle_pos)] != buffer[buffer_pos] {
                needle_pos = table[to_usize(needle_pos)];
            }
            needle_pos += 1;
            buffer_pos += 1;
            let np = to_usize(needle_pos);
            if np >= bytes.len() {
                let start = u64::try_from(file_pos + buffer_pos - np).unwrap();
                consumer(start);
                needle_pos = table[to_usize(needle_pos)];
            }
        }
        file_pos += buffer_pos;
        reader.consume(buffer_pos);
        buffer_pos = 0;
        buffer = reader.fill_buf().unwrap();
    }
}

fn build_table(bytes: &[u8], table: &mut [i32]) {
    let mut pos = 1;
    let mut next = 0;
    while pos < bytes.len() {
        if bytes[pos] == bytes[to_usize(next)] {
            table[pos] = table[to_usize(next)];
        } else {
            table[pos] = next;
            next = table[to_usize(next)];
            while next >= 0 && bytes[pos] != bytes[to_usize(next)] {
                next = table[to_usize(next)];
            }
        }
        pos += 1;
        next += 1;
    }
    table[pos] = next;
}

fn to_usize(n: i32) -> usize {
    usize::try_from(n).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Write, Seek, SeekFrom, BufRead};
    use std::fs::OpenOptions;

    #[test]
    fn searching_in_file() {
        let mut tmpf = tempfile::tempfile().unwrap();
        tmpf.write(b"ababa").unwrap();
        tmpf.seek(SeekFrom::Start(0)).unwrap();

        let mut output: Vec<u64> = Vec::new();
        search(tmpf, b"aba", |hit| output.push(hit));
        assert_eq!(output, vec![0, 2]);
    }
}
