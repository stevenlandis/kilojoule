use super::token::Token;

pub struct TokenDef<'a> {
    pub token: Token,
    pub pattern: &'a str,
}

pub static TOKEN_DEFS: &[TokenDef] = &[
    TokenDef {
        token: Token::IDENTIFIER,
        pattern: r"\w+",
    },
    TokenDef {
        token: Token::DOT,
        pattern: r"\.",
    },
];
