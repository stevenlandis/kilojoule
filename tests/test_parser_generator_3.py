from src.parser_generator3 import *


def test_parse_expr():
    rules = [
        Rule("main", ["expr", "END"]),
        Rule("expr", ["addExpr"]),
        Rule("addExpr", ["mulExpr"]),
        Rule("addExpr", ["addExpr", "PLUS", "mulExpr"]),
        Rule("mulExpr", ["INTEGER"]),
        Rule("mulExpr", ["mulExpr", "ASTERISK", "INTEGER"]),
    ]

    ps = ParserState(rules, 0)

    ps.step("INTEGER", "123")
    ps.step("ASTERISK", "*")
    ps.step("INTEGER", "456")
    ps.step("PLUS", "+")
    ps.step("INTEGER", "7")
    ps.step("ASTERISK", "*")
    ps.step("INTEGER", "8")
    ps.step("PLUS", "+")
    ps.step("INTEGER", "9")
    ps.step("END", "")
    print(ps.val_stack)
    print(ps)
