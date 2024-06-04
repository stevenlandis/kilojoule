use super::token::Token;

#[derive(Clone)]
pub struct LookupRow {
    pub state: u64,
    pub token: Option<Token>,
    pub rule_name: Option<Token>,
    pub token_group: Option<u64>,
    pub next_state: Option<u64>,
    pub reduce_rule: Option<u64>,
}

pub static LOOKUP_ROWS: &[LookupRow] = &[];
