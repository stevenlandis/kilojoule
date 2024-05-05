from src.parser_generator import *

MOCK_RULES2 = [
    CfgRule("main", [CfgProd(["list", "END"])]),
    CfgRule("list", [CfgProd(["elem"]), CfgProd(["list", "elem"])]),
    CfgRule("elem", [CfgProd(["e0"]), CfgProd(["e1"])]),
    CfgRule("e0", [CfgProd(["X", "Y"])]),
    CfgRule("e1", [CfgProd(["X", "Z"])]),
]


def test_basic_grammar():
    parser = Parser(
        [
            TokenGroup(["X"], r"X"),
            TokenGroup(["Y"], r"Y"),
            TokenGroup(["Z"], r"Z"),
        ],
        "list",
        [
            ParserRule(
                "list",
                [["X", "elem"], ["list", "X", "elem"]],
                lambda elems: ("list", elems),
            ),
            ParserRule("elem", [["Y"], ["Z"]], lambda elems: ("elem", elems)),
        ],
    )

    assert parser.parse("XYXZXY") == (
        "list",
        [
            ("list", [("list", ["X", ("elem", ["Y"])]), "X", ("elem", ["Z"])]),
            "X",
            ("elem", ["Y"]),
        ],
    )


def test_other_grammar():
    lookup_map = get_lookup_table_map(MOCK_RULES2, "main")
    parser = Parser(
        [
            TokenGroup(["X"], r"X"),
            TokenGroup(["Y"], r"Y"),
            TokenGroup(["Z"], r"Z"),
        ],
        "list",
        [
            ParserRule(
                "list", [["elem"], ["list", "elem"]], lambda elems: ("list", elems)
            ),
            ParserRule("elem", [["e0"], ["e1"]], lambda elems: ("elem", elems)),
            ParserRule(
                "e0",
                [["X", "Y"]],
                lambda elems: ("e0", elems),
            ),
            ParserRule(
                "e1",
                [["X", "Z"]],
                lambda elems: ("e1", elems),
            ),
        ],
    )

    assert parser.parse("XYXZXY") == (
        "list",
        [
            (
                "list",
                [
                    ("list", [("elem", [("e0", ["X", "Y"])])]),
                    ("elem", [("e1", ["X", "Z"])]),
                ],
            ),
            ("elem", [("e0", ["X", "Y"])]),
        ],
    )


def test_parentheses_grammar():
    rules = [
        CfgRule("main", [CfgProd(["paren", "END"])]),
        CfgRule(
            "paren",
            [
                CfgProd(["(", ")"]),
                CfgProd(["(", "paren", ")"]),
            ],
        ),
    ]
    lookup_map = get_lookup_table_map(rules, "main")
    parser = Parser(
        [
            TokenGroup(["("], r"\("),
            TokenGroup([")"], r"\)"),
        ],
        "paren",
        [
            ParserRule(
                "paren",
                [["(", ")"], ["(", "paren", ")"]],
                lambda elems: ("paren", elems),
            ),
        ],
    )

    assert parser.parse("((()))") == (
        "paren",
        ["(", ("paren", ["(", ("paren", ["(", ")"]), ")"]), ")"],
    )


def test_repeated_rules():
    rules = [
        CfgRule("main", [CfgProd(["S", "END"])]),
        CfgRule("S", [CfgProd(["C", "C"])]),
        CfgRule(
            "C",
            [
                CfgProd(["c", "C"]),
                CfgProd(["d"]),
            ],
        ),
    ]
    lookup_map = get_lookup_table_map(rules, "main")
    parser = Parser(
        [
            TokenGroup(["c"], r"c"),
            TokenGroup(["d"], r"d"),
        ],
        "S",
        [
            ParserRule("S", [["C", "C"]], lambda elems: ("S", elems)),
            ParserRule("C", [["c", "C"], ["d"]], lambda elems: ("C", elems)),
        ],
    )

    assert parser.parse("cdccd") == (
        "S",
        [("C", ["c", ("C", ["d"])]), ("C", ["c", ("C", ["c", ("C", ["d"])])])],
    )


def test_association_rules():
    parser = Parser(
        [
            TokenGroup(["1"], r"1"),
            TokenGroup(["2"], r"2"),
            TokenGroup(["*"], r"\*"),
            TokenGroup(["+"], r"\+"),
        ],
        "expr",
        [
            ParserRule(
                "expr",
                [["addExpr"]],
                lambda elems: ("expr", elems),
            ),
            ParserRule(
                "addExpr",
                [["mulExpr"], ["addExpr", "+", "mulExpr"]],
                lambda elems: ("addExpr", elems),
            ),
            ParserRule(
                "mulExpr",
                [["baseExpr"], ["mulExpr", "*", "baseExpr"]],
                lambda elems: ("mulExpr", elems),
            ),
            ParserRule(
                "baseExpr",
                [["1"], ["2"]],
                lambda elems: ("baseExpr", elems),
            ),
        ],
    )

    assert parser.parse("1") == (
        "expr",
        [("addExpr", [("mulExpr", [("baseExpr", ["1"])])])],
    )

    assert parser.parse("1*2") == (
        "expr",
        [
            (
                "addExpr",
                [
                    (
                        "mulExpr",
                        [("mulExpr", [("baseExpr", ["1"])]), "*", ("baseExpr", ["2"])],
                    )
                ],
            )
        ],
    )

    assert parser.parse("2+1*2+1") == (
        "expr",
        [
            (
                "addExpr",
                [
                    (
                        "addExpr",
                        [
                            ("addExpr", [("mulExpr", [("baseExpr", ["2"])])]),
                            "+",
                            (
                                "mulExpr",
                                [
                                    ("mulExpr", [("baseExpr", ["1"])]),
                                    "*",
                                    ("baseExpr", ["2"]),
                                ],
                            ),
                        ],
                    ),
                    "+",
                    ("mulExpr", [("baseExpr", ["1"])]),
                ],
            )
        ],
    )


def test_parser():
    parser = Parser(
        [TokenGroup(["x"], r"x"), TokenGroup(["y"], r"y"), TokenGroup(["z"], r"z")],
        "list",
        [
            ParserRule(
                "list",
                [["x", "elem"], ["list", "x", "elem"]],
                lambda nodes: ("my_list", nodes),
            ),
            ParserRule(
                "elem",
                [["y"], ["z"]],
                lambda nodes: ("my_elem", nodes),
            ),
        ],
    )

    result = parser.parse("xyxzxy")
    assert result == (
        "my_list",
        [
            (
                "my_list",
                [("my_list", ["x", ("my_elem", ["y"])]), "x", ("my_elem", ["z"])],
            ),
            "x",
            ("my_elem", ["y"]),
        ],
    )


def test_expression_grammar():
    parser = Parser(
        [
            TokenGroup(["PLUS"], r"\+"),
            TokenGroup(["ASTERIKS"], r"\*"),
            TokenGroup(["INTEGER"], r"\d+"),
        ],
        "expr",
        [
            ParserRule(
                "expr",
                [["addExpr"]],
                lambda elems: elems[0],
            ),
            ParserRule(
                "addExpr",
                [["mulExpr"], ["addExpr", "PLUS", "mulExpr"]],
                lambda elems: elems[0] if len(elems) == 1 else elems[0] + elems[2],
            ),
            ParserRule(
                "mulExpr",
                [["baseExpr"], ["mulExpr", "ASTERIKS", "baseExpr"]],
                lambda elems: elems[0] if len(elems) == 1 else elems[0] * elems[2],
            ),
            ParserRule(
                "baseExpr",
                [["INTEGER"]],
                lambda elems: int(elems[0]),
            ),
        ],
        ignore_pattern=r"[ \n\t\r]*",
    )

    assert parser.parse("10 *20  \n\t\r+30*40") == 1400
    assert parser.parse("1234     \n\t\r") == 1234
    assert parser.parse("10*20+30*40+50*60") == 4400


def test_f_string_with_dicts():
    parser = Parser(
        [
            TokenGroup(["STRING_LITERAL"], r'"[^"{]*"'),
            TokenGroup(["F_STRING_LEFT"], r'"[^{]*{'),
            TokenGroup(["F_STRING_MIDDLE"], r"}[^{]*{"),
            TokenGroup(["F_STRING_RIGHT"], r'}[^"]*"'),
            TokenGroup(["L_BRACE"], "{"),
            TokenGroup(["R_BRACE"], "}"),
        ],
        "expr",
        [
            ParserRule(
                "expr",
                [["string"], ["dict"]],
                lambda elems: elems[0],
            ),
            ParserRule(
                "dict",
                [["L_BRACE", "R_BRACE"]],
                lambda elems: "DICT",
            ),
            ParserRule(
                "string",
                [["STRING_LITERAL"], ["f_string"]],
                lambda elems: elems[0],
            ),
            ParserRule(
                "f_string",
                [["F_STRING_LEFT", "f_string_contents", "F_STRING_RIGHT"]],
                lambda elems: ("f_string", [elems[0], *elems[1], elems[2]]),
            ),
            ParserRule(
                "f_string_contents",
                [["expr"], ["f_string_contents", "F_STRING_MIDDLE", "expr"]],
                lambda elems: (
                    [elems[0]] if len(elems) == 1 else [*elems[0], elems[1], elems[2]]
                ),
            ),
        ],
    )

    assert parser.parse('"string"') == '"string"'
    assert parser.parse('"string {{}} end"') == (
        "f_string",
        ['"string {', "DICT", '} end"'],
    )
    assert parser.parse('"a{{}}b{{}}c"') == (
        "f_string",
        ['"a{', "DICT", "}b{", "DICT", '}c"'],
    )
    assert parser.parse('"a{"c{"e{{}}f"}d"}b"') == (
        "f_string",
        [
            '"a{',
            ("f_string", ['"c{', ("f_string", ['"e{', "DICT", '}f"']), '}d"']),
            '}b"',
        ],
    )


def test_callback_to_differentiate_tokens():
    parser = Parser(
        [
            TokenGroup(
                ["IDENTIFIER", "FALSE", "TRUE"],
                r"[A-Za-z][0-9A-Za-z]*",
                lambda value: (
                    "TRUE"
                    if value == "true"
                    else ("FALSE" if value == "false" else "IDENTIFIER")
                ),
            ),
            TokenGroup(["COMMA"], r","),
        ],
        "list",
        [
            ParserRule(
                "list",
                [["listElem"], ["list", "COMMA", "listElem"]],
                lambda elems: [elems[0]] if len(elems) == 1 else [*elems[0], elems[2]],
            ),
            ParserRule(
                "listElem",
                [["IDENTIFIER"], ["trueList"], ["falseList"]],
                lambda elems: elems[0],
            ),
            ParserRule(
                "trueList",
                [["TRUE", "COMMA", "IDENTIFIER", "COMMA", "IDENTIFIER"]],
                lambda elems: ("true list", elems[2], elems[4]),
            ),
            ParserRule(
                "falseList",
                [["FALSE", "COMMA", "IDENTIFIER"]],
                lambda elems: ("false list", elems[2]),
            ),
        ],
        r"[ \n\t\r]*",
    )

    assert parser.parse("thing") == ["thing"]
    assert parser.parse("aa, bb, cc") == ["aa", "bb", "cc"]

    assert parser.parse("aa, false, bb, cc") == ["aa", ("false list", "bb"), "cc"]
    assert parser.parse("aa, true, bb, cc, dd") == [
        "aa",
        ("true list", "bb", "cc"),
        "dd",
    ]
    assert parser.parse("true, a, b, false, c, false, e, f") == [
        ("true list", "a", "b"),
        ("false list", "c"),
        ("false list", "e"),
        "f",
    ]
