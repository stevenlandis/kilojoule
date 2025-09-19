use crate::byte_vec::ByteVec;

use super::byte_vec::ByteVecTrait;
use super::json_lexer::{JsonLexerTrait, JsonToken};

/*
Make a struct that collects JSON tokens from a lexer so it can answer queries.
*/

pub struct ObjectCollector {}

struct ObjectType {}
impl ObjectType {
    const List: u8 = 0;
    const Int64: u8 = 1;
    const String: u8 = 2;
    const Object: u8 = 3;
    const Bool: u8 = 4;
}

pub enum Object {
    Int64(i64),
    ListRef(ListRef),
    StringRef(StringRef),
}

pub struct ObjectRef {
    idx: u64,
}
impl ObjectRef {
    pub fn len<B: ByteVecTrait>(&self, byte_vec: &B) -> u64 {
        byte_vec.get_u64(self.idx + 1)
    }

    pub fn get_key<B: ByteVecTrait>(&self, byte_vec: &B, key: &str) -> Option<u64> {
        let mut idx = self.idx + 1 + 8 + 8;
        let end_idx = self.idx + ObjectCollector::get_obj_len_bytes(self.idx, byte_vec);

        while idx < end_idx {
            let obj_type = byte_vec.get_u8(idx);
            if obj_type == ObjectType::String {
                let search_key = StringRef { idx };
                if search_key.len(byte_vec) as usize == key.len() {
                    let mut matches = true;
                    let key_bytes = key.as_bytes();
                    let key_idx = idx + 1 + 8;
                    for search_idx in 0..key.len() {
                        if key_bytes[search_idx] != byte_vec.get_u8(key_idx + (search_idx as u64)) {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        idx += ObjectCollector::get_obj_len_bytes(idx, byte_vec);
                        return Some(idx);
                    }
                }
            }
            idx += ObjectCollector::get_obj_len_bytes(idx, byte_vec);
            idx += ObjectCollector::get_obj_len_bytes(idx, byte_vec);
        }

        None
    }
}

pub struct StringRef {
    idx: u64,
}

impl StringRef {
    pub fn len<B: ByteVecTrait>(&self, byte_vec: &B) -> u64 {
        byte_vec.get_u64(self.idx + 1)
    }

    pub fn to_string<B: ByteVecTrait>(&self, byte_vec: &B) -> String {
        let len = byte_vec.get_u64(self.idx + 1);
        let start_idx = self.idx + 1 + 8;
        byte_vec
            .get_slice_iterator(start_idx, len)
            .map(|slice| std::str::from_utf8(slice).unwrap())
            .collect::<String>()
    }
}

pub struct ListRef {
    idx: u64,
}

impl ListRef {
    pub fn len<B: ByteVecTrait>(&self, byte_vec: &B) -> u64 {
        byte_vec.get_u64(self.idx + 1)
    }

    pub fn get<B: ByteVecTrait>(&self, list_idx: usize, byte_vec: &B) -> u64 {
        let mut start_idx = self.idx + 1 + 8 + 8;
        let list_len = byte_vec.get_i64(self.idx + 1) as usize;
        assert!(list_idx < list_len);

        for _ in 0..list_idx {
            start_idx += ObjectCollector::get_obj_len_bytes(start_idx, byte_vec);
        }

        start_idx
    }
}

impl ObjectCollector {
    pub fn new() -> Self {
        ObjectCollector {}
    }

    pub fn populate_from_lexer<L: JsonLexerTrait, B: ByteVecTrait>(
        lexer: &mut L,
        byte_vec: &mut B,
    ) {
        let mut stack = Vec::<StackItem>::new();

        loop {
            let token = lexer.next();
            match token {
                JsonToken::Done => break,
                JsonToken::NumberStart { is_negative, digit } => {
                    let mut value: i64 = (digit - ('0' as u8)) as i64;
                    let mut decimal: u64 = 0;
                    let mut exponent: u64 = 0;
                    let mut exponent_is_negative: bool = false;

                    loop {
                        let token = lexer.next();
                        match token {
                            JsonToken::NumberDigit(digit) => {
                                value = 10 * value + (digit - ('0' as u8)) as i64
                            }
                            JsonToken::NumberEnd => {
                                break;
                            }
                            JsonToken::NumberDecimalPoint => {
                                loop {
                                    let token = lexer.next();
                                    match token {
                                        JsonToken::NumberDigit(digit) => {
                                            decimal = 10 * decimal + (digit - ('0' as u8)) as u64;
                                        }
                                        JsonToken::NumberEnd => break,
                                        JsonToken::NumberExponentStart => {
                                            loop {
                                                let token = lexer.next();
                                                match token {
                                                    JsonToken::NumberStart {
                                                        is_negative,
                                                        digit,
                                                    } => {
                                                        exponent_is_negative = is_negative;
                                                        exponent = (digit - ('0' as u8)) as u64;
                                                    }
                                                    JsonToken::NumberDigit(digit) => {
                                                        exponent = 10 * exponent
                                                            + (digit - ('0' as u8)) as u64;
                                                    }
                                                    JsonToken::NumberEnd => break,
                                                    _ => unreachable!("{:?}", token),
                                                }
                                            }
                                            break;
                                        }
                                        _ => unreachable!("{:?}", token),
                                    }
                                }
                                break;
                            }
                            JsonToken::NumberExponentStart => {
                                loop {
                                    let token = lexer.next();
                                    match token {
                                        JsonToken::NumberStart { is_negative, digit } => {
                                            exponent_is_negative = is_negative;
                                            exponent = (digit - ('0' as u8)) as u64;
                                        }
                                        JsonToken::NumberDigit(digit) => {
                                            exponent = 10 * exponent + (digit - ('0' as u8)) as u64;
                                        }
                                        JsonToken::NumberEnd => break,
                                        _ => unreachable!("{:?}", token),
                                    }
                                }
                                break;
                            }
                            _ => unreachable!("{:?}", token),
                        }
                    }

                    if is_negative {
                        value *= -1;
                    }

                    byte_vec.push_u8(ObjectType::Int64);
                    byte_vec.push_i64(value);
                }
                JsonToken::ListStart => {
                    stack.push(StackItem {
                        obj_idx: byte_vec.len(),
                    });
                    byte_vec.push_u8(ObjectType::List);
                    byte_vec.push_u64(u64::MAX); // reserve for length
                    byte_vec.push_u64(u64::MAX); // reserve for n_bytes

                    /*
                    Lists have the following layout:
                    - u8: ObjectType::List
                    - u64: LEN
                    - u64: N_BYTES
                    - ... serialized elements
                    // - ... u64 for each serialized element
                    */
                }
                JsonToken::ListEnd => {
                    let stack_entry = stack.pop().unwrap();
                    let obj_idx = stack_entry.obj_idx;
                    let end_idx = byte_vec.len();

                    let mut len: u64 = 0;
                    let mut n_bytes: u64 = 1 + 8 + 8;
                    let mut idx: u64 = obj_idx + 1 + 8 + 8;
                    while idx < end_idx {
                        // byte_vec.push_u64(idx); // push index of element
                        let elem_n_bytes = Self::get_obj_len_bytes(idx, byte_vec);
                        len += 1;
                        n_bytes += elem_n_bytes;
                        idx += elem_n_bytes;
                    }
                    // n_bytes += len * 8; // Add space for indexes

                    byte_vec.set_u64(obj_idx + 1, len);
                    byte_vec.set_u64(obj_idx + 1 + 8, n_bytes);
                }
                JsonToken::StringStart => {
                    stack.push(StackItem {
                        obj_idx: byte_vec.len(),
                    });
                    byte_vec.push_u8(ObjectType::String);
                    byte_vec.push_u64(u64::MAX); // reserve for n_bytes
                }
                JsonToken::StringUtf8CodePoint(code_point) => {
                    byte_vec.push_u8(code_point);
                }
                JsonToken::StringEnd => {
                    let stack_entry = stack.pop().unwrap();
                    let obj_idx = stack_entry.obj_idx;
                    let n_u8_chars = (byte_vec.len() - obj_idx - 1 - 8) as u64;
                    byte_vec.set_u64(obj_idx + 1, n_u8_chars);
                }
                JsonToken::ObjectStart => {
                    /*
                    - u8: ObjectType::Object
                    - u64: LEN
                    - u64: N_BYTES
                    - key0, val0
                    - key1, val1
                    - ...
                    */

                    stack.push(StackItem {
                        obj_idx: byte_vec.len(),
                    });
                    byte_vec.push_u8(ObjectType::Object);
                    byte_vec.push_u64(u64::MAX); // reserve for LEN
                    byte_vec.push_u64(u64::MAX); // reserve for N_BYTES
                }
                JsonToken::ObjectEnd => {
                    let stack_entry = stack.pop().unwrap();
                    let obj_idx = stack_entry.obj_idx;
                    let end_idx = byte_vec.len();

                    let mut len: u64 = 0;
                    let mut n_bytes: u64 = 1 + 8 + 8;
                    let mut idx: u64 = obj_idx + 1 + 8 + 8;
                    while idx < end_idx {
                        let elem_n_bytes = Self::get_obj_len_bytes(idx, byte_vec);
                        len += 1;
                        n_bytes += elem_n_bytes;
                        idx += elem_n_bytes;
                    }

                    // Make sure length is odd because one key and value for each entry
                    assert_eq!(len & 0b1, 0);
                    len >>= 1;

                    byte_vec.set_u64(obj_idx + 1, len);
                    byte_vec.set_u64(obj_idx + 1 + 8, n_bytes);
                }
                JsonToken::False => {
                    byte_vec.push_u8(ObjectType::Bool);
                    byte_vec.push_u8(0);
                }
                JsonToken::True => {
                    byte_vec.push_u8(ObjectType::Bool);
                    byte_vec.push_u8(1);
                }
                _ => unimplemented!("{:?}", token),
            }
        }
    }

    fn get_obj_len_bytes<B: ByteVecTrait>(idx: u64, byte_vec: &B) -> u64 {
        match byte_vec.get_u8(idx) {
            ObjectType::Int64 => 1 + 8,
            ObjectType::List => {
                let n_bytes = byte_vec.get_u64(idx + 1 + 8);
                assert_ne!(n_bytes, u64::MAX);
                n_bytes
            }
            ObjectType::String => {
                let n_chars = byte_vec.get_u64(idx + 1);
                assert_ne!(n_chars, u64::MAX);
                n_chars + 1 + 8
            }
            ObjectType::Object => {
                let n_bytes = byte_vec.get_u64(idx + 1 + 8);
                assert_ne!(n_bytes, u64::MAX);
                n_bytes
            }
            ObjectType::Bool => 2,
            _ => unimplemented!(),
        }
    }

    pub fn get_list<B: ByteVecTrait>(idx: u64, byte_vec: &B) -> ListRef {
        assert_eq!(byte_vec.get_u8(idx), ObjectType::List);
        ListRef { idx }
    }

    pub fn get_i64<B: ByteVecTrait>(idx: u64, byte_vec: &B) -> i64 {
        assert_eq!(byte_vec.get_u8(idx), ObjectType::Int64);
        byte_vec.get_i64(idx + 1)
    }

    pub fn get_string<B: ByteVecTrait>(idx: u64, byte_vec: &B) -> StringRef {
        assert_eq!(byte_vec.get_u8(idx), ObjectType::String);
        StringRef { idx }
    }

    pub fn get_object<B: ByteVecTrait>(idx: u64, byte_vec: &B) -> ObjectRef {
        assert_eq!(byte_vec.get_u8(idx), ObjectType::Object);
        ObjectRef { idx }
    }
}

struct StackItem {
    obj_idx: u64,
}

pub enum ObjectCollectorError {
    InvalidType,
}

#[cfg(test)]
mod test {
    use super::super::byte_vec::ByteVec;
    use super::super::json_lexer::JsonLexer;
    use super::super::reader::StrReader;
    use super::*;

    #[test]
    fn test_basic_int64() {
        let mut reader = StrReader::new("123");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);

        assert_eq!(ObjectCollector::get_i64(0, &byte_vec), 123);
    }

    #[test]
    fn test_basic_list_of_ints() {
        let mut reader = StrReader::new("[3, 1, 2]");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);

        let list = ObjectCollector::get_list(0, &byte_vec);
        assert_eq!(list.len(&byte_vec), 3);
        assert_eq!(
            ObjectCollector::get_i64(list.get(0, &byte_vec), &byte_vec),
            3
        );
        assert_eq!(
            ObjectCollector::get_i64(list.get(1, &byte_vec), &byte_vec),
            1
        );
        assert_eq!(
            ObjectCollector::get_i64(list.get(2, &byte_vec), &byte_vec),
            2
        );
    }

    #[test]
    fn test_empty_list() {
        let mut reader = StrReader::new("[]");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);

        let list = ObjectCollector::get_list(0, &byte_vec);
        assert_eq!(list.len(&byte_vec), 0);
    }

    #[test]
    fn test_simple_list_of_lists() {
        let mut reader = StrReader::new("[[]]");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);
        let byte_vec = &byte_vec;

        let list0 = ObjectCollector::get_list(0, byte_vec);
        assert_eq!(list0.len(byte_vec), 1);

        let list1 = ObjectCollector::get_list(list0.get(0, byte_vec), byte_vec);
        assert_eq!(list1.len(byte_vec), 0);
    }

    #[test]
    fn test_mixed_list_of_lists() {
        let mut reader = StrReader::new("[[1,2],3,[4,5,6],7]");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);
        let byte_vec = &byte_vec;

        let list0 = ObjectCollector::get_list(0, byte_vec);
        assert_eq!(list0.len(byte_vec), 4);

        let list1 = ObjectCollector::get_list(list0.get(0, byte_vec), byte_vec);
        assert_eq!(list1.len(byte_vec), 2);
        assert_eq!(
            ObjectCollector::get_i64(list1.get(0, byte_vec), byte_vec),
            1
        );
        assert_eq!(
            ObjectCollector::get_i64(list1.get(1, byte_vec), byte_vec),
            2
        );

        assert_eq!(
            ObjectCollector::get_i64(list0.get(1, byte_vec), byte_vec),
            3
        );

        let list2 = ObjectCollector::get_list(list0.get(2, byte_vec), byte_vec);
        assert_eq!(list2.len(byte_vec), 3);
        assert_eq!(
            ObjectCollector::get_i64(list2.get(0, byte_vec), byte_vec),
            4
        );
        assert_eq!(
            ObjectCollector::get_i64(list2.get(1, byte_vec), byte_vec),
            5
        );
    }

    #[test]
    fn test_basic_string() {
        let mut reader = StrReader::new("\"hello world\"");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);
        let byte_vec = &byte_vec;

        let str0 = ObjectCollector::get_string(0, byte_vec);
        assert_eq!(str0.to_string(byte_vec), "hello world");
        assert_eq!(str0.len(byte_vec), 11);
    }

    #[test]
    fn test_array_of_strings() {
        let mut reader = StrReader::new("[1, \"22\", 3, \"4\"]");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);
        let byte_vec = &byte_vec;

        let list = ObjectCollector::get_list(0, byte_vec);

        assert_eq!(ObjectCollector::get_i64(list.get(0, byte_vec), byte_vec), 1);

        let str0 = ObjectCollector::get_string(list.get(1, byte_vec), byte_vec);
        assert_eq!(str0.to_string(byte_vec), "22");
        assert_eq!(str0.len(byte_vec), 2);

        assert_eq!(ObjectCollector::get_i64(list.get(2, byte_vec), byte_vec), 3);

        let str1 = ObjectCollector::get_string(list.get(3, byte_vec), byte_vec);
        assert_eq!(str1.to_string(byte_vec), "4");
        assert_eq!(str1.len(byte_vec), 1);
    }

    #[test]
    fn test_basic_object() {
        let mut reader = StrReader::new("{\"a\": 100, \"b\": 200}");
        let mut lexer = JsonLexer::new(&mut reader);
        let mut byte_vec = ByteVec::new();
        ObjectCollector::populate_from_lexer(&mut lexer, &mut byte_vec);
        let byte_vec = &byte_vec;

        let obj = ObjectCollector::get_object(0, byte_vec);
        assert_eq!(
            ObjectCollector::get_i64(obj.get_key(byte_vec, "a").unwrap(), byte_vec),
            100
        );
        assert_eq!(
            ObjectCollector::get_i64(obj.get_key(byte_vec, "b").unwrap(), byte_vec),
            200
        );
    }
}
