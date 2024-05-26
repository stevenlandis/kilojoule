import json
from src.parser_generator2 import get_lookup_tbl_rows, Rule
from pathlib import Path


def main():
    src = Path("kilojoule_rust/src")

    rules = [
        Rule("main", ["expr", "END"]),
        Rule("expr", ["assignExpr"]),
        Rule("assignExpr", ["opPipeExpr"]),
        Rule(
            "assignExpr",
            ["LET", "IDENTIFIER", "EQUAL", "opOrExpr", "PIPE", "assignExpr"],
        ),
        Rule(
            "assignExpr",
            [
                "opPipeExpr",
                "PIPE",
                "LET",
                "IDENTIFIER",
                "EQUAL",
                "opOrExpr",
                "PIPE",
                "assignExpr",
            ],
        ),
        Rule("opPipeExpr", ["opOrExpr"]),
        Rule("opPipeExpr", ["opPipeExpr", "PIPE", "opOrExpr"]),
        Rule("opOrExpr", ["opAndExpr"]),
        Rule("opOrExpr", ["opOrExpr", "OR", "opAndExpr"]),
        Rule("opAndExpr", ["opEqualityExpr"]),
        Rule("opAndExpr", ["opAndExpr", "AND", "opEqualityExpr"]),
        Rule("equalityOperator", ["DOUBLE_EQUALS"]),
        Rule("equalityOperator", ["NOT_EQUALS"]),
        Rule("equalityOperator", ["LESS_THAN"]),
        Rule("equalityOperator", ["LESS_THAN_OR_EQUAL"]),
        Rule("equalityOperator", ["GREATER_THAN"]),
        Rule("equalityOperator", ["GREATER_THAN_OR_EQUAL"]),
        Rule("opEqualityExpr", ["opAddExpr"]),
        Rule("opEqualityExpr", ["opAddExpr", "equalityOperator", "opAddExpr"]),
        Rule("opAddOperator", ["PLUS"]),
        Rule("opAddOperator", ["MINUS"]),
        Rule("opAddExpr", ["opMulExpr"]),
        Rule("opAddExpr", ["opAddExpr", "opAddOperator", "opMulExpr"]),
        Rule("opMulOperator", ["ASTERISK"]),
        Rule("opMulOperator", ["FORWARD_SLASH"]),
        Rule("opMulExpr", ["baseExpr"]),
        Rule("opMulExpr", ["opMulExpr", "opMulOperator", "baseExpr"]),
        Rule("baseExpr", ["baseDotExpr"]),
        Rule("baseExpr", ["baseDotAccess"]),
        Rule("baseExpr", ["baseDotBracketAccess"]),
        Rule("baseExpr", ["mapExpr"]),
        Rule("baseExpr", ["listExpr"]),
        Rule("baseExpr", ["LEFT_PAREN", "expr", "RIGHT_PAREN"]),
        Rule("baseExpr", ["INTEGER"]),
        Rule("baseExpr", ["FLOAT"]),
        Rule("baseExpr", ["stringLiteral"]),
        Rule("baseExpr", ["TRUE"]),
        Rule("baseExpr", ["FALSE"]),
        Rule("baseExpr", ["NULL"]),
        Rule("baseExpr", ["fcnCallExpr"]),
        Rule("baseExpr", ["IDENTIFIER"]),
        Rule("baseDotExpr", ["DOT"]),
        Rule("baseDotAccess", ["DOT", "IDENTIFIER"]),
        Rule(
            "baseDotBracketAccess",
            ["DOT", "LEFT_BRACKET", "listAccessExpr", "RIGHT_BRACKET"],
        ),
        Rule("listAccessIdx", ["expr"]),
        Rule("listAccessIdx", ["FORWARD_SLASH", "expr"]),
        Rule("listAccessExpr", ["listAccessIdx"]),
        Rule("listAccessExpr", ["listAccessIdx", "COLON"]),
        Rule("listAccessExpr", ["COLON", "listAccessIdx"]),
        Rule("listAccessExpr", ["listAccessIdx", "COLON", "listAccessIdx"]),
        Rule("listAccessExpr", ["COLON"]),
        Rule("mapExpr", ["LEFT_BRACE", "RIGHT_BRACE"]),
        Rule("mapExpr", ["LEFT_BRACE", "mapContents", "RIGHT_BRACE"]),
        Rule("mapExpr", ["LEFT_BRACE", "mapContents", "COMMA", "RIGHT_BRACE"]),
        Rule("mapContents", ["mapContentsElem"]),
        Rule("mapContents", ["mapContents", "COMMA", "mapContentsElem"]),
        Rule("mapContentsElem", ["IDENTIFIER", "COLON", "expr"]),
        Rule("listExpr", ["LEFT_BRACKET", "RIGHT_BRACKET"]),
        Rule("listExpr", ["LEFT_BRACKET", "listExprContents", "RIGHT_BRACKET"]),
        Rule(
            "listExpr", ["LEFT_BRACKET", "listExprContents", "COMMA", "RIGHT_BRACKET"]
        ),
        Rule("listExprContents", ["listElem"]),
        Rule("listExprContents", ["listExprContents", "COMMA", "listElem"]),
        Rule("listElem", ["expr"]),
        Rule("stringLiteral", ["STRING_DOUBLE_QUOTE"]),
        Rule("stringLiteral", ["STRING_SINGLE_QUOTE"]),
        Rule(
            "stringLiteral",
            [
                "F_STRING_SINGLE_QUOTE_LEFT",
                "innerFormatStringSingleQuote",
                "F_STRING_SINGLE_QUOTE_RIGHT",
            ],
        ),
        Rule("innerFormatStringSingleQuote", ["expr"]),
        Rule(
            "innerFormatStringSingleQuote",
            [
                "innerFormatStringSingleQuote",
                "F_STRING_SINGLE_QUOTE_MIDDLE",
                "expr",
            ],
        ),
        Rule("fcnCallExpr", ["IDENTIFIER", "LEFT_PAREN", "RIGHT_PAREN"]),
        Rule("fcnCallExpr", ["IDENTIFIER", "LEFT_PAREN", "fcnCallArgs", "RIGHT_PAREN"]),
        Rule(
            "fcnCallExpr",
            ["IDENTIFIER", "LEFT_PAREN", "fcnCallArgs", "COMMA", "RIGHT_PAREN"],
        ),
        Rule("fcnCallArgs", ["expr"]),
        Rule("fcnCallArgs", ["fcnCallArgs", "COMMA", "expr"]),
    ]

    # generate token.rs
    tokens = set()
    for rule in rules:
        tokens.add(rule.name)
        tokens.update(rule.steps)
    tokens = sorted(tokens)
    with open(src / "token.rs", "w") as fid:
        fid.write("#[allow(non_camel_case_types)]\n")
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
    pub token: Option<Token>,
    pub rule_name: Option<Token>,
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
            fid.write(
                f"        rule_name: {'None' if row.rule_name is None else f'Some(Token::{row.rule_name})'},\n"
            )
            fid.write(
                f"        token: {'None' if row.token is None else f'Some(Token::{row.token})'},\n"
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
