from src.parser_generator2 import *


def test_basic_expr():
    parser = Parser(
        [
            ("INTEGER", r"\d+"),
            ("PLUS", r"\+"),
            ("ASTERISK", r"\*"),
        ],
        r"[ \t\n\r]*",
        "expr",
        [
            Rule("expr", ["addExpr"], lambda elems: ("expr", *elems)),
            Rule("addExpr", ["mulExpr"], lambda elems: ("add", *elems)),
            Rule(
                "addExpr", ["addExpr", "PLUS", "mulExpr"], lambda elems: ("add", *elems)
            ),
            Rule("mulExpr", ["INTEGER"], lambda elems: ("mull", *elems)),
            Rule(
                "mulExpr",
                ["mulExpr", "ASTERISK", "INTEGER"],
                lambda elems: ("mull", *elems),
            ),
        ],
    )

    assert parser.parse("123 * 456") == (
        "expr",
        ("add", ("mull", ("mull", "123"), "*", "456")),
    )
    assert parser.parse("123 * 456*789") == (
        "expr",
        ("add", ("mull", ("mull", ("mull", "123"), "*", "456"), "*", "789")),
    )
    assert parser.parse("1*2 + 3") == (
        "expr",
        ("add", ("add", ("mull", ("mull", "1"), "*", "2")), "+", ("mull", "3")),
    )
    assert parser.parse("1*2 + 3*4") == (
        "expr",
        (
            "add",
            ("add", ("mull", ("mull", "1"), "*", "2")),
            "+",
            ("mull", ("mull", "3"), "*", "4"),
        ),
    )


def test_token_precedence():
    parser = Parser(
        [("FOO", r"foo"), ("IDENTIFIER", r"\w+")],
        None,
        "expr",
        [
            Rule("expr", ["FOO"], lambda elems: ("FOO", elems[0])),
            Rule("expr", ["IDENTIFIER"], lambda elems: ("identifier", elems[0])),
        ],
    )

    assert parser.parse("foo") == ("FOO", "foo")
    assert parser.parse("fo") == ("identifier", "fo")
    assert parser.parse("fooo") == ("identifier", "fooo")
