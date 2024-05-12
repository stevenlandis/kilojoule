use super::token::Token;

pub struct TokenDef<'a> {
    pub token: Token,
    pub pattern: &'a str,
}

pub static TOKEN_DEFS: &[TokenDef] = &[
    TokenDef {
        token: Token::INTEGER,
        pattern: r"\d+",
    },
    TokenDef {
        token: Token::PLUS,
        pattern: r"\+",
    },
    TokenDef {
        token: Token::ASTERISK,
        pattern: r"\*",
    },
];
