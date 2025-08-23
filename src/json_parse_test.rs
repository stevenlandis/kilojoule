use kilojoule::{JsonLexer, JsonLexerTrait, JsonToken, Reader};
use std::io::Read;

struct FileReader {
    file: std::fs::File,
    buffer: [u8; 2048],
    buf_len: usize,
    idx: usize,
    total_idx: usize,
}

impl FileReader {
    pub fn new(path: &str) -> Self {
        FileReader {
            file: std::fs::File::open(path).unwrap(),
            buffer: [0u8; 2048],
            buf_len: 0,
            idx: 0,
            total_idx: 0,
        }
    }
}

impl Reader for FileReader {
    fn peek(&mut self) -> Option<u8> {
        if self.idx < self.buf_len {
            // println!("Read {}", self.buffer[self.idx] as char);
            return Some(self.buffer[self.idx]);
        }

        // Else read bytes into buffer
        self.buf_len = match self.file.read(&mut self.buffer) {
            Err(err) => {
                println!("Reader error: {}", err);
                return None;
            }
            Ok(buf_len) => buf_len,
        };
        // println!("Read n_bytes={}", self.buf_len);
        self.idx = 0;

        if self.idx < self.buf_len {
            // println!("Read {}", self.buffer[self.idx] as char);
            return Some(self.buffer[self.idx]);
        }

        // println!("returning None");
        None
    }

    fn step(&mut self) {
        self.idx += 1;
        self.total_idx += 1;
    }

    fn get_idx(&mut self) -> usize {
        self.total_idx
    }
}

fn main() {
    let mut reader = FileReader::new("/Users/steven/Downloads/space_age.2.0.60.json");

    let mut lexer = JsonLexer::new(&mut reader);

    let mut n_loops: u64 = 0;
    loop {
        n_loops += 1;
        match lexer.next() {
            JsonToken::Done => {
                println!("got done token");
                break;
            }
            JsonToken::Error(err) => {
                println!("Got error {:?}", err);
                break;
            }
            _ => {}
        }
    }
    println!("Done in n_loops={}", n_loops);
}
