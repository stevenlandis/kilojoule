import json
from src.parser_generator2 import get_lookup_tbl_rows, Rule


def main():
    rules = [
        Rule("Main", ["Expr", "END"]),
        Rule("Expr", ["AddExpr"]),
        Rule("AddExpr", ["MulExpr"]),
        Rule("AddExpr", ["AddExpr", "PLUS", "MulExpr"]),
        Rule("MulExpr", ["INTEGER"]),
        Rule("MulExpr", ["MulExpr", "ASTERISK", "INTEGER"]),
    ]

    lookup_rows, token_groups = get_lookup_tbl_rows(rules, 0)

    rule_enums = []

    for rule in rules:
        rule_enum_name = f"{rule.name}__{'_'.join(rule.steps)}"
        rule_enums.append(rule_enum_name)
        txt = ""
        txt += f"rule_type: RuleType::{rule_enum_name},"
        txt += f"token: Token::{rule.name},"
        txt += f"steps: &[{','.join(f'Token::{step}' for step in rule.steps)}],"
        txt = f"Rule {{{txt}}},"
        print(txt)
    print("-----------------------")

    print("enum RuleType {")
    for enum in rule_enums:
        print(f"    {enum},")
    print("}")

    print("-----------------------")

    for row in lookup_rows:
        txt = ""
        txt += f"state: {row.state},"
        txt += f"token: Token::{row.token},"
        txt += f"follow_token: {'None' if row.follow_token is None else f'Some(Token::{row.follow_token})'},"
        txt += f"token_group: {'None' if row.token_group is None else f'Some({row.token_group})'},"
        txt += f"next_state: {'None' if row.next_state is None else f'Some({row.next_state})'},"
        txt += f"reduce_rule: {'None' if row.reduce_rule is None else f'Some({row.reduce_rule})'},"
        txt = f"LookupRow{{{txt}}},"
        print(txt)

    print("-----------------")
    for token_group, tokens in token_groups.items():
        print(token_group, tokens)


"""
LookupRow(state=0, token='INTEGER', follow_token=None, token_group=2, next_state=None, reduce_rule=4)

LookupRow(state=0, token='addExpr', follow_token='END', token_group=None, next_state=None, reduce_rule=1)
LookupRow(state=0, token='addExpr', follow_token='PLUS', token_group=None, next_state=2, reduce_rule=None)
LookupRow(state=0, token='expr', follow_token=None, token_group=None, next_state=1, reduce_rule=None)
LookupRow(state=0, token='mulExpr', follow_token='PLUS', token_group=None, next_state=None, reduce_rule=2)
LookupRow(state=0, token='mulExpr', follow_token='END', token_group=None, next_state=None, reduce_rule=2)
LookupRow(state=0, token='mulExpr', follow_token='ASTERISK', token_group=None, next_state=4, reduce_rule=None)
LookupRow(state=1, token='END', follow_token=None, token_group=1, next_state=None, reduce_rule=0)
LookupRow(state=2, token='PLUS', follow_token=None, token_group=0, next_state=3, reduce_rule=None)
LookupRow(state=3, token='INTEGER', follow_token=None, token_group=2, next_state=None, reduce_rule=4)
LookupRow(state=3, token='mulExpr', follow_token='PLUS', token_group=None, next_state=None, reduce_rule=3)
LookupRow(state=3, token='mulExpr', follow_token='END', token_group=None, next_state=None, reduce_rule=3)
LookupRow(state=3, token='mulExpr', follow_token='ASTERISK', token_group=None, next_state=4, reduce_rule=None)
LookupRow(state=4, token='ASTERISK', follow_token=None, token_group=0, next_state=5, reduce_rule=None)
LookupRow(state=5, token='INTEGER', follow_token=None, token_group=2, next_state=None, reduce_rule=5)
-----------------
0 ('INTEGER',)
1 ()
2 ('ASTERISK', 'END', 'PLUS')
"""
