use super::json_lexer::{JsonLexerTrait, JsonToken};

/*
Make a struct that collects JSON tokens from a lexer so it can answer queries.
*/

pub trait ByteVec {
    fn get_u8(&self, idx: u64) -> u8;
    fn get_u64(&self, idx: u64) -> u64;
    fn get_i64(&self, idx: u64) -> i64;
    fn set_u64(&mut self, idx: u64, value: u64);
    fn push_u8(&mut self, value: u8);
    fn push_u64(&mut self, value: u64);
    fn push_i64(&mut self, value: i64);
    fn get_next_idx(&self) -> u64;
}

pub struct ObjectCollector<'a, L: JsonLexerTrait, B: ByteVec> {
    lexer: &'a mut L,
    byte_vec: &'a mut B,
}

struct ObjectType {}
impl ObjectType {
    const List: u8 = 0;
    const Int64: u8 = 1;
}

impl<'a, L: JsonLexerTrait, B: ByteVec> ObjectCollector<'a, L, B> {
    pub fn create_from_lexer(&mut self, lexer: &mut L) {
        let mut stack = Vec::<StackItem>::new();

        loop {
            let token = lexer.next();
            match token {
                JsonToken::Done => break,
                JsonToken::NumberStart { is_negative, digit } => {
                    let mut value: i64 = (digit - ('0' as u8)) as i64;
                    if is_negative {
                        value *= -1;
                    }
                    loop {
                        let token = lexer.next();
                        match token {
                            JsonToken::NumberDigit(digit) => {
                                value = 10 * value + (digit - ('0' as u8)) as i64
                            }
                            JsonToken::NumberEnd => {
                                break;
                            }
                            _ => unreachable!(),
                        }
                    }
                    self.byte_vec.push_u8(ObjectType::Int64);
                    self.byte_vec.push_i64(value);
                }
                JsonToken::ListStart => {
                    stack.push(StackItem {
                        obj_idx: self.byte_vec.get_next_idx(),
                    });
                    self.byte_vec.push_u8(ObjectType::List);
                    self.byte_vec.push_u64(u64::MAX); // reserve for length
                    self.byte_vec.push_u64(u64::MAX); // reserve for n_bytes

                    /*
                    Lists have the following layout:
                    - u8: ObjectType::List
                    - u64: LEN
                    - u64: N_BYTES
                    - ... serialized elements
                    - ... u64 for each serialized element
                    */
                }
                JsonToken::ListEnd => {
                    let stack_entry = stack.pop().unwrap();
                    let obj_idx = stack_entry.obj_idx;
                    let end_idx = self.byte_vec.get_next_idx();

                    let mut len: u64 = 0;
                    let mut n_bytes: u64 = 0;
                    let mut idx: u64 = obj_idx + 1 + 8 + 8;
                    while idx < end_idx {
                        self.byte_vec.push_u64(idx); // push index of element
                        let elem_n_bytes = self.get_obj_len_bytes(idx);
                        len += 1;
                        n_bytes += elem_n_bytes;
                        idx += elem_n_bytes;
                    }
                    n_bytes += len * 8; // Add space for indexes

                    self.byte_vec.set_u64(obj_idx + 1, len);
                    self.byte_vec.set_u64(obj_idx + 1 + 8, n_bytes);
                }
                _ => unimplemented!(),
            }
        }
    }

    fn get_obj_len_bytes(&self, idx: u64) -> u64 {
        match self.byte_vec.get_u8(idx) {
            ObjectType::Int64 => 1 + 8,
            ObjectType::List => {
                let n_bytes = self.byte_vec.get_u64(idx + 1 + 8);
                assert_ne!(n_bytes, u64::MAX);
                n_bytes
            }
            _ => unimplemented!(),
        }
    }
}

struct StackItem {
    obj_idx: u64,
}

pub enum ObjectCollectorError {
    InvalidType,
}
