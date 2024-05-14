use super::token::Token;

pub struct TokenDef<'a> {
    pub token: Token,
    pub pattern: &'a str,
}

pub static TOKEN_DEFS: &[TokenDef] = &[
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
        token: Token::STRING_DOUBLE_QUOTE,
        pattern: r#""(?:[^"\\{}]|\\.)*""#,
    },
];
