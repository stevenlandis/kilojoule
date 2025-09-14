use super::reader::ReaderTrait;

pub trait JsonLexerTrait {
    fn next(&mut self) -> JsonToken;
}

pub struct JsonLexer<'a, R: ReaderTrait> {
    reader: &'a mut R,
    stack: Vec<ParseState>,
}

impl<'a, R: ReaderTrait> JsonLexer<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        JsonLexer {
            reader,
            stack: vec![ParseState::JsonStart],
        }
    }

    fn set_top_state(&mut self, state: ParseState) {
        let idx = self.stack.len() - 1;
        self.stack[idx] = state;
    }

    fn start_next_object(&mut self) -> JsonToken {
        match self.reader.peek() {
            None => self.report_error(ParseError::UnexpectedReaderEnd),
            Some(char) => {
                if char == '{' as u8 {
                    self.reader.step();
                    self.stack.push(ParseState::ObjectParseKey);
                    return JsonToken::ObjectStart;
                }

                if char == '[' as u8 {
                    self.reader.step();
                    self.stack.push(ParseState::ListParseElement);
                    return JsonToken::ListStart;
                }

                if char == '"' as u8 {
                    self.reader.step();
                    self.stack.push(ParseState::StringInside);
                    return JsonToken::StringStart;
                }

                if is_numeric(char) {
                    self.reader.step();
                    self.stack.push(ParseState::NumberBeforeDecimalPoint);
                    return JsonToken::NumberStart {
                        is_negative: false,
                        digit: char,
                    };
                }

                if char == '-' as u8 {
                    self.reader.step();
                    self.parse_ws();
                    match self.reader.peek() {
                        None => {
                            return self
                                .report_error(ParseError::NoCharacterAfterNegativeNumberStart)
                        }
                        Some(first_digit) => {
                            if is_numeric(first_digit) {
                                self.reader.step();
                                self.stack.push(ParseState::NumberBeforeDecimalPoint);
                                return JsonToken::NumberStart {
                                    is_negative: true,
                                    digit: first_digit,
                                };
                            }

                            return self
                                .report_error(ParseError::InvalidDigitAtNegativeNumberStart);
                        }
                    }
                }

                if match self.parse_string_literal(char, "true") {
                    Err(err) => return err,
                    Ok(matches) => matches,
                } {
                    return JsonToken::True;
                }

                if match self.parse_string_literal(char, "false") {
                    Err(err) => return err,
                    Ok(matches) => matches,
                } {
                    return JsonToken::False;
                }

                return self.report_error(ParseError::UnexpectedCharacter);
            }
        }
    }

    fn parse_string_literal(&mut self, first_char: u8, text: &str) -> Result<bool, JsonToken> {
        let bytes = text.as_bytes();
        if first_char == bytes[0] {
            self.reader.step();
            for idx in 1..bytes.len() {
                let match_char = bytes[idx];
                match self.reader.peek() {
                    None => return Err(self.report_error(ParseError::UnexpectedReaderEnd)),
                    Some(char) => {
                        if char == match_char {
                            self.reader.step();
                        } else {
                            return Err(self.report_error(ParseError::UnexpectedCharacter));
                        }
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn report_error(&mut self, error: ParseError) -> JsonToken {
        self.stack.clear();
        self.stack.push(ParseState::ParseError);

        JsonToken::Error(ErrorInfo {
            idx: self.reader.get_idx(),
            error,
        })
    }

    fn try_parse_number_exponent(&mut self, first_char: u8) -> Option<JsonToken> {
        if first_char == 'e' as u8 || first_char == 'E' as u8 {
            self.reader.step();
            self.set_top_state(ParseState::NumberExponentStart);
            return Some(JsonToken::NumberExponentStart);
        }

        None
    }

    fn parse_ws(&mut self) {
        loop {
            match self.reader.peek() {
                None => {
                    break;
                }
                Some(ch) => {
                    if !is_whitespace(ch) {
                        break;
                    }
                }
            }
            self.reader.step();
        }
    }

    fn parse_hex_u16(&mut self) -> Result<u16, ParseError> {
        let mut result: u16 = 0;

        match self.reader.peek() {
            None => return Err(ParseError::ReaderFinishedInMiddleOfCodepoint),
            Some(char) => match get_hex_value_from_char(char) {
                None => return Err(ParseError::InvalidHexCodePoint),
                Some(val) => {
                    result += (val as u16) << 12;
                    self.reader.step();
                }
            },
        }
        match self.reader.peek() {
            None => return Err(ParseError::ReaderFinishedInMiddleOfCodepoint),
            Some(char) => match get_hex_value_from_char(char) {
                None => return Err(ParseError::InvalidHexCodePoint),
                Some(val) => {
                    result += (val as u16) << 8;
                    self.reader.step();
                }
            },
        }
        match self.reader.peek() {
            None => return Err(ParseError::ReaderFinishedInMiddleOfCodepoint),
            Some(char) => match get_hex_value_from_char(char) {
                None => return Err(ParseError::InvalidHexCodePoint),
                Some(val) => {
                    result += (val as u16) << 4;
                    self.reader.step();
                }
            },
        }
        match self.reader.peek() {
            None => return Err(ParseError::ReaderFinishedInMiddleOfCodepoint),
            Some(char) => match get_hex_value_from_char(char) {
                None => return Err(ParseError::InvalidHexCodePoint),
                Some(val) => {
                    result += val as u16;
                    self.reader.step();
                }
            },
        }

        return Ok(result);
    }
}

impl<'a, R: ReaderTrait> JsonLexerTrait for JsonLexer<'a, R> {
    fn next(&mut self) -> JsonToken {
        if self.stack.len() == 0 {
            return JsonToken::Done;
        }

        let state = self.stack[self.stack.len() - 1];

        match state {
            ParseState::ParseError => return JsonToken::InErrorState,
            ParseState::JsonStart => {
                self.parse_ws();
                self.set_top_state(ParseState::JsonInside);
                return self.start_next_object();
            }
            ParseState::JsonInside => {
                self.stack.pop();
                return JsonToken::Done;
            }
            ParseState::ObjectParseKey => {
                self.parse_ws();

                match self.reader.peek() {
                    None => return self.report_error(ParseError::UnexpectedReaderEnd),
                    Some(char) => {
                        if char == '"' as u8 {
                            self.reader.step();
                            self.set_top_state(ParseState::ObjectParseValue);
                            self.stack.push(ParseState::StringInside);
                            return JsonToken::StringStart;
                        }

                        if char == '}' as u8 {
                            self.reader.step();
                            self.stack.pop();
                            return JsonToken::ObjectEnd;
                        }

                        return self.report_error(ParseError::UnexpectedCharacter);
                    }
                }
            }
            ParseState::ObjectParseCommaThenKey => {
                self.parse_ws();

                match self.reader.peek() {
                    None => return self.report_error(ParseError::UnexpectedReaderEnd),
                    Some(char) => {
                        if char == '}' as u8 {
                            self.reader.step();
                            self.stack.pop();
                            return JsonToken::ObjectEnd;
                        }

                        if char == ',' as u8 {
                            self.reader.step();
                            self.set_top_state(ParseState::ObjectParseValue);
                            self.parse_ws();
                            return self.start_next_object();
                        }

                        return self.report_error(ParseError::UnexpectedCharacter);
                    }
                }
            }
            ParseState::ObjectParseValue => {
                self.parse_ws();

                match self.reader.peek() {
                    None => return self.report_error(ParseError::UnexpectedReaderEnd),
                    Some(char) => {
                        if char == ':' as u8 {
                            self.reader.step();
                            self.parse_ws();

                            self.set_top_state(ParseState::ObjectParseCommaThenKey);

                            return self.start_next_object();
                        }

                        return self.report_error(ParseError::UnexpectedCharacter);
                    }
                }
            }
            ParseState::ListParseElement => {
                self.parse_ws();
                match self.reader.peek() {
                    None => return self.report_error(ParseError::MissingFirstCharacter),
                    Some(char) => {
                        if char == ']' as u8 {
                            self.reader.step();
                            self.stack.pop();
                            return JsonToken::ListEnd;
                        }

                        self.set_top_state(ParseState::ListParseCommaThenElement);

                        return self.start_next_object();
                    }
                }
            }
            ParseState::ListParseCommaThenElement => {
                self.parse_ws();
                match self.reader.peek() {
                    None => return self.report_error(ParseError::MissingFirstCharacter),
                    Some(char) => {
                        if char == ']' as u8 {
                            self.reader.step();
                            self.stack.pop();
                            return JsonToken::ListEnd;
                        }

                        if char == ',' as u8 {
                            self.reader.step();
                            self.parse_ws();
                            return self.start_next_object();
                        }

                        return self.report_error(ParseError::UnexpectedCharacter);
                    }
                }
            }
            ParseState::NumberBeforeDecimalPoint => match self.reader.peek() {
                None => {
                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
                Some(char) => {
                    if is_numeric(char) {
                        self.reader.step();
                        return JsonToken::NumberDigit(char);
                    }

                    if char == '.' as u8 {
                        self.reader.step();
                        self.set_top_state(ParseState::NumberAfterDecimalPoint);
                        return JsonToken::NumberDecimalPoint;
                    }

                    match self.try_parse_number_exponent(char) {
                        None => {}
                        Some(token) => return token,
                    }

                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
            },
            ParseState::NumberAfterDecimalPoint => match self.reader.peek() {
                None => {
                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
                Some(char) => {
                    if is_numeric(char) {
                        self.reader.step();
                        return JsonToken::NumberDigit(char);
                    }

                    match self.try_parse_number_exponent(char) {
                        None => {}
                        Some(token) => return token,
                    }

                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
            },
            ParseState::NumberExponentStart => match self.reader.peek() {
                None => return self.report_error(ParseError::UnexpectedReaderEnd),
                Some(char) => {
                    let (is_negative, digit) = if char == '-' as u8 {
                        self.reader.step();

                        let digit = match self.reader.peek() {
                            None => return self.report_error(ParseError::UnexpectedReaderEnd),
                            Some(char) => char,
                        };

                        (true, digit)
                    } else if is_numeric(char) {
                        (false, char)
                    } else {
                        return self.report_error(ParseError::UnexpectedCharacter);
                    };

                    self.reader.step();
                    self.set_top_state(ParseState::NumberExponentDigit);
                    return JsonToken::NumberStart { is_negative, digit };
                }
            },
            ParseState::NumberExponentDigit => match self.reader.peek() {
                None => {
                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
                Some(char) => {
                    if is_numeric(char) {
                        self.reader.step();
                        return JsonToken::NumberDigit(char);
                    }

                    self.stack.pop();
                    return JsonToken::NumberEnd;
                }
            },
            ParseState::StringInside => match self.reader.peek() {
                None => return self.report_error(ParseError::ReaderFinishedInMiddleOfString),
                Some(char) => {
                    if char == '"' as u8 {
                        self.reader.step();
                        self.stack.pop();
                        return JsonToken::StringEnd;
                    }

                    if char == '\\' as u8 {
                        self.reader.step();
                        match self.reader.peek() {
                            None => {
                                return self
                                    .report_error(ParseError::ReaderFinishedInMiddleOfString)
                            }
                            Some(char) => {
                                if char == 'u' as u8 {
                                    self.reader.step();
                                    let code_point = match self.parse_hex_u16() {
                                        Err(err) => return self.report_error(err),
                                        Ok(code_point) => code_point,
                                    };
                                    todo!()
                                }

                                // https://www.json.org/json-en.html
                                let escaped_char = if char == '"' as u8 {
                                    '"' as u8
                                } else if char == '\\' as u8 {
                                    '\\' as u8
                                } else if char == '/' as u8 {
                                    '/' as u8
                                } else if char == 'b' as u8 {
                                    8 // backspace
                                } else if char == 'f' as u8 {
                                    12 // form feed
                                } else if char == 'n' as u8 {
                                    '\n' as u8
                                } else if char == 'r' as u8 {
                                    '\r' as u8
                                } else if char == 't' as u8 {
                                    '\t' as u8
                                } else {
                                    return self
                                        .report_error(ParseError::StringInvalidEscapeCharacter);
                                };
                                self.reader.step();
                                return JsonToken::StringUtf8CodePoint(escaped_char);
                            }
                        }
                    }

                    self.reader.step();
                    return JsonToken::StringUtf8CodePoint(char as u8);
                }
            },
        }
    }
}

#[derive(Clone, Copy)]
enum ParseState {
    ParseError,
    JsonStart,
    JsonInside,
    ObjectParseKey,
    ObjectParseCommaThenKey,
    ObjectParseValue,
    ListParseElement,
    ListParseCommaThenElement,
    NumberBeforeDecimalPoint,
    NumberAfterDecimalPoint,
    NumberExponentStart,
    NumberExponentDigit,
    StringInside,
}

#[derive(PartialEq, Eq, Debug)]
pub enum JsonToken {
    Error(ErrorInfo),
    Done,
    InErrorState,
    ObjectStart,
    ObjectEnd,
    ListStart,
    ListEnd,
    NumberStart { is_negative: bool, digit: u8 },
    NumberDigit(u8),
    NumberDecimalPoint,
    NumberExponentStart,
    NumberEnd,
    StringStart,
    StringUtf8CodePoint(u8),
    StringEnd,
    True,
    False,
}

#[derive(PartialEq, Eq, Debug)]
pub struct ErrorInfo {
    pub idx: usize,
    pub error: ParseError,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ParseError {
    UnexpectedReaderEnd,
    UnexpectedCharacterAfterParseDone,
    MissingFirstCharacter,
    UnexpectedCharacter,
    NoCharacterAfterNegativeNumberStart,
    InvalidDigitAtNegativeNumberStart,
    ReaderFinishedInMiddleOfString,
    StringInvalidEscapeCharacter,
    ReaderFinishedInMiddleOfCodepoint,
    InvalidHexCodePoint,
}

fn is_whitespace(val: u8) -> bool {
    val == (' ' as u8) || val == ('\n' as u8) || val == ('\t' as u8) || val == ('\r' as u8)
}

fn is_alpha(val: u8) -> bool {
    ('A' as u8) <= val && val <= ('Z' as u8) || ('a' as u8) <= val && val <= ('z' as u8)
}

fn is_alpha_underscore(val: u8) -> bool {
    is_alpha(val) || val == ('_' as u8)
}

fn is_numeric(val: u8) -> bool {
    ('0' as u8) <= val && val <= ('9' as u8)
}

fn is_alpha_underscore_numeric(val: u8) -> bool {
    is_alpha_underscore(val) || is_numeric(val)
}

fn get_hex_value_from_char(val: u8) -> Option<u8> {
    if ('0' as u8) <= val && val <= ('9' as u8) {
        return Some(val - ('0' as u8));
    }

    if ('A' as u8) <= val && val <= ('F' as u8) {
        return Some(val - ('A' as u8) + 10);
    }

    if ('a' as u8) <= val && val <= ('f' as u8) {
        return Some(val - ('a' as u8) + 10);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::reader::StrReader;
    use super::*;

    fn get_all_tokens_from_lexer<R: ReaderTrait>(lexer: &mut JsonLexer<R>) -> Vec<JsonToken> {
        let mut tokens = Vec::<JsonToken>::new();

        loop {
            let next_token = lexer.next();
            match next_token {
                JsonToken::Done => {
                    // Be extra sure that calling .next() on a finished lexer
                    // still returns Done.
                    assert_eq!(lexer.next(), JsonToken::Done);
                    break;
                }
                JsonToken::Error(_) => {
                    // Be extra sure that calling .next() on a finished lexer
                    // still returns Error.
                    let next_error = lexer.next();
                    assert_eq!(next_error, JsonToken::InErrorState);

                    tokens.push(next_token);
                    break;
                }
                _ => {
                    tokens.push(next_token);
                }
            }
        }

        return tokens;
    }

    fn parse_into_vec(text: &str) -> Vec<JsonToken> {
        let mut reader = StrReader::new(text);
        let mut lexer = JsonLexer::new(&mut reader);

        return get_all_tokens_from_lexer(&mut lexer);
    }

    #[test]
    fn test_number() {
        assert_eq!(
            parse_into_vec(" 1"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("132"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberDigit('3' as u8),
                JsonToken::NumberDigit('2' as u8),
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("-7"),
            vec![
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '7' as u8
                },
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("-72"),
            vec![
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '7' as u8
                },
                JsonToken::NumberDigit('2' as u8),
                JsonToken::NumberEnd
            ]
        );

        assert_eq!(
            parse_into_vec("-  7"),
            vec![
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '7' as u8
                },
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("-  72"),
            vec![
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '7' as u8
                },
                JsonToken::NumberDigit('2' as u8),
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("12.34"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberDigit('2' as u8),
                JsonToken::NumberDecimalPoint,
                JsonToken::NumberDigit('3' as u8),
                JsonToken::NumberDigit('4' as u8),
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("1e2"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberExponentStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("1e-2"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberExponentStart,
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd
            ]
        );
        assert_eq!(
            parse_into_vec("1e-23"),
            vec![
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberExponentStart,
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '2' as u8
                },
                JsonToken::NumberDigit('3' as u8),
                JsonToken::NumberEnd
            ]
        );
    }

    #[test]
    fn test_list() {
        assert_eq!(
            parse_into_vec("[]"),
            vec![JsonToken::ListStart, JsonToken::ListEnd,]
        );
        assert_eq!(
            parse_into_vec("[ ]"),
            vec![JsonToken::ListStart, JsonToken::ListEnd,]
        );
        assert_eq!(
            parse_into_vec("[1]"),
            vec![
                JsonToken::ListStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListEnd,
            ]
        );
        assert_eq!(
            parse_into_vec("[ 1 ]"),
            vec![
                JsonToken::ListStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListEnd,
            ]
        );
        assert_eq!(
            parse_into_vec("[1,2]"),
            vec![
                JsonToken::ListStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListEnd,
            ]
        );
        assert_eq!(
            parse_into_vec("[ 1 , 2 ]"),
            vec![
                JsonToken::ListStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListEnd,
            ]
        );
        assert_eq!(
            parse_into_vec("[[]]"),
            vec![
                JsonToken::ListStart,
                JsonToken::ListStart,
                JsonToken::ListEnd,
                JsonToken::ListEnd,
            ]
        );
        assert_eq!(
            parse_into_vec("[[],9, []]"),
            vec![
                JsonToken::ListStart,
                JsonToken::ListStart,
                JsonToken::ListEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '9' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListStart,
                JsonToken::ListEnd,
                JsonToken::ListEnd,
            ]
        );
    }

    #[test]
    fn test_parse_multiple_objects() {
        // This is a cool example of three valid JSON objects together in a
        // single string. This test creates three separate lexers to parse each
        // object.
        // In real use cases, each object would probably be on a separate line.
        let text = "[]-12[2]";
        let mut reader = StrReader::new(text);

        let mut lexer0 = JsonLexer::new(&mut reader);
        assert_eq!(
            get_all_tokens_from_lexer(&mut lexer0),
            vec![JsonToken::ListStart, JsonToken::ListEnd,]
        );

        let mut lexer1 = JsonLexer::new(&mut reader);
        assert_eq!(
            get_all_tokens_from_lexer(&mut lexer1),
            vec![
                JsonToken::NumberStart {
                    is_negative: true,
                    digit: '1' as u8
                },
                JsonToken::NumberDigit('2' as u8),
                JsonToken::NumberEnd,
            ]
        );

        let mut lexer2 = JsonLexer::new(&mut reader);
        assert_eq!(
            get_all_tokens_from_lexer(&mut lexer2),
            vec![
                JsonToken::ListStart,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ListEnd,
            ]
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_into_vec(r#""""#),
            vec![JsonToken::StringStart, JsonToken::StringEnd,]
        );
        assert_eq!(
            parse_into_vec(r#""a""#),
            vec![
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('a' as u8),
                JsonToken::StringEnd,
            ]
        );
        assert_eq!(
            parse_into_vec(r#""asdf""#),
            vec![
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('a' as u8),
                JsonToken::StringUtf8CodePoint('s' as u8),
                JsonToken::StringUtf8CodePoint('d' as u8),
                JsonToken::StringUtf8CodePoint('f' as u8),
                JsonToken::StringEnd,
            ]
        );
        assert_eq!(
            parse_into_vec(r#""\"\\\/\b\f\n\r\t""#),
            vec![
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('"' as u8),
                JsonToken::StringUtf8CodePoint('\\' as u8),
                JsonToken::StringUtf8CodePoint('/' as u8),
                JsonToken::StringUtf8CodePoint(8),
                JsonToken::StringUtf8CodePoint(12),
                JsonToken::StringUtf8CodePoint('\n' as u8),
                JsonToken::StringUtf8CodePoint('\r' as u8),
                JsonToken::StringUtf8CodePoint('\t' as u8),
                JsonToken::StringEnd,
            ]
        );
        assert_eq!(
            parse_into_vec(r#""\q""#),
            vec![
                JsonToken::StringStart,
                JsonToken::Error(ErrorInfo {
                    idx: 2,
                    error: ParseError::StringInvalidEscapeCharacter
                })
            ]
        );
    }

    #[test]
    fn test_parse_object() {
        assert_eq!(
            parse_into_vec(r#"{}"#),
            vec![JsonToken::ObjectStart, JsonToken::ObjectEnd,]
        );
        assert_eq!(
            parse_into_vec(r#" { "a" : 1 } "#),
            vec![
                JsonToken::ObjectStart,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('a' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ObjectEnd,
            ]
        );
        assert_eq!(
            parse_into_vec(r#" { "a" : 1 , "b" : 2 } "#),
            vec![
                JsonToken::ObjectStart,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('a' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('b' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ObjectEnd,
            ]
        );
        assert_eq!(
            parse_into_vec(r#" { "a" : 1 , "b" : 2, "c": 3 } "#),
            vec![
                JsonToken::ObjectStart,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('a' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '1' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('b' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '2' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::StringStart,
                JsonToken::StringUtf8CodePoint('c' as u8),
                JsonToken::StringEnd,
                JsonToken::NumberStart {
                    is_negative: false,
                    digit: '3' as u8
                },
                JsonToken::NumberEnd,
                JsonToken::ObjectEnd,
            ]
        );
    }

    #[test]
    fn test_bools() {
        assert_eq!(
            parse_into_vec(r#"[false,true]"#),
            vec![
                JsonToken::ListStart,
                JsonToken::False,
                JsonToken::True,
                JsonToken::ListEnd,
            ]
        );
    }
}
