import json
from src.parser_generator2 import get_lookup_tbl_rows, Rule
from pathlib import Path


def main():
    src = Path("kilojoule_rust/src")

    rules = [
        Rule("Main", ["Expr", "END"]),
        Rule("Expr", ["AddExpr"]),
        Rule("AddExpr", ["MulExpr"]),
        Rule("AddExpr", ["AddExpr", "PLUS", "MulExpr"]),
        Rule("MulExpr", ["INTEGER"]),
        Rule("MulExpr", ["MulExpr", "ASTERISK", "INTEGER"]),
    ]

    # generate token.rs
    tokens = set()
    for rule in rules:
        tokens.add(rule.name)
        tokens.update(rule.steps)
    tokens = sorted(tokens)
    with open(src / "token.rs", "w") as fid:
        fid.write("#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]\n")
        fid.write("pub enum Token {\n")
        for token in tokens:
            fid.write(f"    {token},\n")
        fid.write("}\n")

    lookup_rows, token_groups = get_lookup_tbl_rows(rules, 0)

    rule_types = [f"{rule.name}__{'_'.join(rule.steps)}" for rule in rules]

    # generate rule_type.rs
    with open(src / "rule_type.rs", "w") as fid:
        fid.write("#[allow(non_camel_case_types)]\n")
        fid.write("#[derive(Debug, Clone, Copy)]\n")
        fid.write("pub enum RuleType {\n")
        for rule_type in rule_types:
            fid.write(f"    {rule_type},\n")
        fid.write("}\n")

    # generate rules.rs
    with open(src / "rules.rs", "w") as fid:
        fid.write(
            """
use super::rule_type::RuleType;
use super::token::Token;

pub struct Rule<'a> {
    pub rule_type: RuleType,
    pub token: Token,
    pub steps: &'a [Token],
}

pub static RULES: &[Rule] = &[
""".lstrip()
        )
        for rule, rule_type in zip(rules, rule_types):
            fid.write("    Rule {\n")
            fid.write(f"        rule_type: RuleType::{rule_type},\n")
            fid.write(f"        token: Token::{rule.name},\n")
            fid.write(
                f"        steps: &[{','.join(f'Token::{step}' for step in rule.steps)}],\n"
            )
            fid.write("    },\n")
        fid.write("];\n")

    # generate lookup_rows.rs
    with open(src / "lookup_rows.rs", "w") as fid:
        fid.write(
            """
use super::token::Token;

#[derive(Clone)]
pub struct LookupRow {
    pub state: u64,
    pub token: Token,
    pub follow_token: Option<Token>,
    pub token_group: Option<u64>,
    pub next_state: Option<u64>,
    pub reduce_rule: Option<u64>,
}

pub static LOOKUP_ROWS: &[LookupRow] = &[
""".lstrip()
        )
        for row in lookup_rows:
            fid.write("    LookupRow {\n")
            fid.write(f"        state: {row.state},\n")
            fid.write(f"        token: Token::{row.token},\n")
            fid.write(
                f"        follow_token: {'None' if row.follow_token is None else f'Some(Token::{row.follow_token})'},\n"
            )
            fid.write(
                f"        token_group: {'None' if row.token_group is None else f'Some({row.token_group})'},\n"
            )
            fid.write(
                f"        next_state: {'None' if row.next_state is None else f'Some({row.next_state})'},\n"
            )
            fid.write(
                f"        reduce_rule: {'None' if row.reduce_rule is None else f'Some({row.reduce_rule})'},\n"
            )
            fid.write("    },\n")
        fid.write("];\n")

    # generate token_groups.rs
    with open(src / "token_groups.rs", "w") as fid:
        fid.write(
            """
use super::token::Token;

pub struct TokenGroup<'a> {
    pub tokens: &'a [Token],
}

pub static TOKEN_GROUPS: &[TokenGroup] = &[
""".lstrip()
        )
        sorted_token_groups = [
            tokens
            for _, tokens in sorted(token_groups.items(), key=lambda pair: pair[0])
        ]
        for tokens in sorted_token_groups:
            fid.write("    TokenGroup {\n")
            fid.write(
                f"        tokens: &[{', '.join(f'Token::{token}' for token in tokens)}],\n"
            )
            fid.write("    },\n")
        fid.write("];\n")
