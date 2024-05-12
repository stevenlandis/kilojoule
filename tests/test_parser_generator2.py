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


def test_dot_ambiguity():
    parser = Parser(
        [
            ("DOT", r"\."),
            ("IDENTIFIER", r"\w+"),
        ],
        None,
        "expr",
        [
            Rule("expr", ["DOT"], lambda elems: "."),
            Rule("expr", ["DOT", "IDENTIFIER"], lambda elems: (".", elems[1])),
        ],
    )

    assert parser.parse(".") == "."
    assert parser.parse(".stuff") == (".", "stuff")


def test_sort_tuple():
    def assert_lt(left, right):
        assert_eq(left, left)
        assert_eq(right, right)
        assert (SortTuple(left) < SortTuple(right)) is True
        assert (SortTuple(right) < SortTuple(left)) is False

    def assert_eq(left, right):
        assert (SortTuple(left) < SortTuple(right)) is False
        assert (SortTuple(right) < SortTuple(left)) is False

    assert_lt((), (1,))
    assert_lt((1,), (2,))
    assert_lt((100, 1), (100, 2))
    assert_lt((None,), (1,))
    assert_lt((1, 2, None), (1, 2, 3))
    assert_lt((1, 100), (2, 0))
