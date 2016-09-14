use std::io::{BufReader, Read, Write, copy, BufWriter};
use std::fs::{File, metadata};
use std::io::Result as IoResult;
use std::path::Path;

struct TruncRead<R: Read> {
    inner: R,
    count: u64,
    max: u64,
}

impl <R> Read for TruncRead<R> where R: Read {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if self.count >= self.max {
            Ok(0)
        } else if buf.len() as u64 + self.count > self.max {
            let actual_len = self.max - self.count;
            let r = self.inner.read(&mut buf[..actual_len as usize]);

            if let Ok(bytes_read) = r {
                self.count += bytes_read as u64;
            }

            r
        } else {
            let r = self.inner.read(buf);

            if let Ok(bytes_read) = r {
                self.count += bytes_read as u64;
            }

            r
        }
    }
}

impl <R: Read> TruncRead<R> {
    fn new(inner: R, max_size: u64) -> TruncRead<R> {
        TruncRead {
            inner: inner,
            count: 0,
            max: max_size,
        }
    }

    fn unwrap(self) -> R {
        self.inner
    }
}

fn split_file<P1, P2, P3>(input: P1, out1: P2, out2: P3) -> IoResult<()>
where P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path> {
    let input_size = try!(metadata(&input)).len();
    let first_file_size = input_size / 2;
    println!("first of size {}", first_file_size);
    let mut input_buffer = BufReader::new(try!(File::open(input)));
    let mut first_out = BufWriter::new(try!(File::create(out1)));
    let mut second_out = BufWriter::new(try!(File::create(out2)));

    let mut first_reader = TruncRead::new(input_buffer, first_file_size);
    try!(copy(&mut first_reader, &mut first_out));
    let mut second_reader = first_reader.unwrap();
    try!(copy(&mut second_reader, &mut second_out));

    Ok(())
}

fn main() {
    let file = ::std::env::args().nth(1);
    let file = match file {
        Some(f) => f,
        None => {
            println!("no file passed");
            return;
        }
    };

    split_file(&file, format!("{}.1", file), format!("{}.2", file)).unwrap()
}
