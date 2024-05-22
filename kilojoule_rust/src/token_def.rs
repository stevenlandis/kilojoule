use super::token::Token;

pub struct TokenDef<'a> {
    pub token: Token,
    pub pattern: &'a str,
}

pub static TOKEN_DEFS: &[TokenDef] = &[
    TokenDef {
        token: Token::END,
        pattern: "",
    },
    TokenDef {
        token: Token::TRUE,
        pattern: r"true",
    },
    TokenDef {
        token: Token::FALSE,
        pattern: r"false",
    },
    TokenDef {
        token: Token::IDENTIFIER,
        pattern: r"[_A-Za-z][_A-Za-z0-9]*",
    },
    TokenDef {
        token: Token::INTEGER,
        pattern: r"[0-9]+",
    },
    TokenDef {
        token: Token::FLOAT,
        pattern: r"[0-9]+(?:\.[0-9]+)?",
    },
    TokenDef {
        token: Token::DOT,
        pattern: r"\.",
    },
    TokenDef {
        token: Token::PIPE,
        pattern: r"\|",
    },
    TokenDef {
        token: Token::LEFT_PAREN,
        pattern: r"\(",
    },
    TokenDef {
        token: Token::RIGHT_PAREN,
        pattern: r"\)",
    },
    TokenDef {
        token: Token::LEFT_BRACE,
        pattern: r"\{",
    },
    TokenDef {
        token: Token::RIGHT_BRACE,
        pattern: r"\}",
    },
    TokenDef {
        token: Token::LEFT_BRACKET,
        pattern: r"\[",
    },
    TokenDef {
        token: Token::RIGHT_BRACKET,
        pattern: r"\]",
    },
    TokenDef {
        token: Token::COMMA,
        pattern: r"\,",
    },
    TokenDef {
        token: Token::COLON,
        pattern: r"\:",
    },
    TokenDef {
        token: Token::STRING_SINGLE_QUOTE,
        pattern: r#"'(?:[^'\\{}]|\\.)*'"#,
    },
    TokenDef {
        token: Token::F_STRING_SINGLE_QUOTE_LEFT,
        pattern: r#"'(?:[^'\\{}]|\\.)*\{"#,
    },
    TokenDef {
        token: Token::F_STRING_SINGLE_QUOTE_MIDDLE,
        pattern: r#"}(?:[^'\\{}]|\\.)*\{"#,
    },
    TokenDef {
        token: Token::F_STRING_SINGLE_QUOTE_RIGHT,
        pattern: r#"}(?:[^'\\{}]|\\.)*'"#,
    },
    TokenDef {
        token: Token::STRING_DOUBLE_QUOTE,
        pattern: r#""(?:[^"\\{}]|\\.)*""#,
    },
];
