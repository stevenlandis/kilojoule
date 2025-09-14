pub trait ReaderTrait {
    fn peek(&mut self) -> Option<u8>;
    fn step(&mut self);
    fn get_idx(&mut self) -> usize;
}

pub struct StrReader<'a> {
    text: &'a str,
    idx: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(text: &'a str) -> Self {
        StrReader { text, idx: 0 }
    }
}

impl<'a> ReaderTrait for StrReader<'a> {
    fn peek(&mut self) -> Option<u8> {
        if self.idx >= self.text.len() {
            None
        } else {
            Some(self.text.as_bytes()[self.idx])
        }
    }

    fn step(&mut self) {
        self.idx += 1;
    }

    fn get_idx(&mut self) -> usize {
        self.idx
    }
}
