import json
import pytest
from src.run_json_query_expr import run_json_query_expr


def test_access():
    assert run_json_query_expr({"a": 1, "b": 2}, ".") == {"a": 1, "b": 2}
    assert run_json_query_expr({"a": 1, "b": 2}, ".a") == 1
    assert run_json_query_expr({"a": 1, "b": 2}, ".b") == 2
    assert run_json_query_expr({"list": ["a", "b"]}, ".list[1]") == "b"
    assert run_json_query_expr({"list": ["a", "b"]}, ".list | .[1]") == "b"
    assert (
        run_json_query_expr({"list": ["a", {"b": "b_val"}]}, ".list | .[1].b")
        == "b_val"
    )
    assert run_json_query_expr(42, ". | . | .") == 42

    # access by expression
    assert run_json_query_expr({"a": 100, "b": 200}, ".['b']") == 200


def test_access_missing_key():
    assert run_json_query_expr({}, ".missing_key") == None
    assert run_json_query_expr(None, ".missing_key") == None


def test_range_access():
    assert run_json_query_expr(["a", "b", "c", "d"], ".[1:2]") == ["b"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[1:3]") == ["b", "c"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[1:4]") == ["b", "c", "d"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[0:4]") == ["a", "b", "c", "d"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[1:-1]") == ["b", "c"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[-2:-1]") == ["c"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[1:1]") == []

    # just with start
    assert run_json_query_expr(["a", "b", "c", "d"], ".[2:]") == ["c", "d"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[-2:]") == ["c", "d"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[-1:]") == ["d"]

    # just with end
    assert run_json_query_expr(["a", "b", "c", "d"], ".[:2]") == ["a", "b"]
    assert run_json_query_expr(["a", "b", "c", "d"], ".[:-1]") == ["a", "b", "c"]


def test_get_array():
    assert run_json_query_expr({"a": 1, "b": 2}, "[.a, .b]") == [1, 2]
    assert run_json_query_expr({"a": 1, "b": 2}, "[]") == []


def test_array_spread():
    assert run_json_query_expr([1, 2, 3], "[*., .[0], * map [.]]") == [
        1,
        2,
        3,
        1,
        [1],
        [2],
        [3],
    ]


def test_get_dict():
    assert run_json_query_expr({"a": 1, "b": 2}, "{new_a: .a, new_b: .b}") == {
        "new_a": 1,
        "new_b": 2,
    }
    assert run_json_query_expr({"a": 1, "b": 2}, "{}") == {}


def test_dict_spread():
    assert run_json_query_expr({"a": 1, "b": 2}, "{*., c: .b}") == {
        "a": 1,
        "b": 2,
        "c": 2,
    }


def test_map():
    assert run_json_query_expr(
        [{"value": 1}, {"value": 2}, {"value": 3}], "map .value"
    ) == [1, 2, 3]
    assert run_json_query_expr([1, 2, 3], "map {val: .}") == [
        {"val": 1},
        {"val": 2},
        {"val": 3},
    ]
    assert run_json_query_expr([{"a": {"b": {"c": 42}}}], "map .a.b.c") == [42]
    assert run_json_query_expr([{"a": {"b": {"c": 42}}}], "(map .a.b.c)[0]") == 42

    # map a dict
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "map [.]") == {
        "a": [1],
        "b": [2],
        "c": [3],
    }


def test_filter():
    assert run_json_query_expr([True, False, True], "map {val: .} | filter .val") == [
        {"val": True},
        {"val": True},
    ]


def test_innequalities():
    assert run_json_query_expr([4, 5, 6], "map . < 5") == [True, False, False]
    assert run_json_query_expr([4, 5, 6], "map . <= 5") == [True, True, False]
    assert run_json_query_expr([4, 5, 6], "map . > 5") == [False, False, True]
    assert run_json_query_expr([4, 5, 6], "map . >= 5") == [False, True, True]
    assert run_json_query_expr([4, 5, 6], "map . == 5") == [False, True, False]
    assert run_json_query_expr([4, 5, 6], "map . != 5") == [True, False, True]


def test_number_literal():
    assert run_json_query_expr(1, "42") == 42
    assert run_json_query_expr(1, "42.75") == 42.75
    assert run_json_query_expr(1, "-42") == -42
    assert run_json_query_expr(1, "-42.75") == -42.75


def test_negative_expr():
    assert run_json_query_expr(42, "-.") == -42
    assert run_json_query_expr([1, -2], "-.[1]") == 2
    assert run_json_query_expr({"a": 42}, "-.a") == -42
    assert run_json_query_expr({"a": {"b": 42}}, "-.a.b") == -42

    with pytest.raises(Exception):
        run_json_query_expr(1, "--1")


def test_string_literal():
    assert run_json_query_expr(1, "'stuff'") == "stuff"
    assert run_json_query_expr(1, "''") == ""

    assert run_json_query_expr(1, '"stuff"') == "stuff"
    assert run_json_query_expr(1, '""') == ""


def test_format_string():
    assert run_json_query_expr(1, "'a {1}'") == "a 1"
    assert run_json_query_expr(1, "'{1} a'") == "1 a"
    assert run_json_query_expr(1, "'a {1} b'") == "a 1 b"
    assert run_json_query_expr(1, "'a {1} b {2} c'") == "a 1 b 2 c"
    assert run_json_query_expr(1, "'a {1}{2} c'") == "a 12 c"
    assert run_json_query_expr(1, "'{1}{2}{3}'") == "123"
    assert run_json_query_expr(1, "' \\{\\}\\n\\t\\r\\'\\\" '") == " {}\n\t\r'\" "
    assert run_json_query_expr(1, "' \" '") == ' " '

    # nested
    assert run_json_query_expr(1, "'a {'b c'}'") == "a b c"
    assert run_json_query_expr(1, "'a {'b {42} c'}'") == "a b 42 c"
    assert run_json_query_expr(1, "'a {'b {'c {42}'} d'}'") == "a b c 42 d"

    # array
    assert run_json_query_expr([1, 2, 3], "'before{.}after'") == "before[1,2,3]after"

    # dict
    assert (
        run_json_query_expr({"a": 1, "b": 2}, "'before{.}after'")
        == 'before{"a":1,"b":2}after'
    )

    # boolean
    assert run_json_query_expr([False, True], "'{.[0]}||{.[1]}'") == "false||true"


def test_format_string_double_quote():
    assert run_json_query_expr(1, '"a {1}"') == "a 1"
    assert run_json_query_expr(1, '"{1} a"') == "1 a"
    assert run_json_query_expr(1, '"a {1} b"') == "a 1 b"
    assert run_json_query_expr(1, '"a {1} b {2} c"') == "a 1 b 2 c"
    assert run_json_query_expr(1, '"a {1}{2} c"') == "a 12 c"
    assert run_json_query_expr(1, '"{1}{2}{3}"') == "123"
    assert run_json_query_expr(1, '" \\{\\}\\n\\t\\r\\"\\\' "') == " {}\n\t\r\"' "
    assert run_json_query_expr(1, '" \' "') == " ' "
    assert run_json_query_expr(1, '"Name: \\"{1}\\""') == 'Name: "1"'

    # nested
    assert run_json_query_expr(1, '"a {"b c"}"') == "a b c"
    assert run_json_query_expr(1, '"a {"b {42} c"}"') == "a b 42 c"
    assert run_json_query_expr(1, '"a {"b {"c {42}"} d"}"') == "a b c 42 d"

    # array
    assert run_json_query_expr([1, 2, 3], '"before{.}after"') == "before[1,2,3]after"

    # dict
    assert (
        run_json_query_expr({"a": 1, "b": 2}, '"before{.}after"')
        == 'before{"a":1,"b":2}after'
    )

    # boolean
    assert run_json_query_expr([False, True], "'{.[0]}||{.[1]}'") == "false||true"


def test_mixed_format_strings():
    query = """
        "stuff {'and {"things"} and {"things"}'} and {'{"{'stuff'}"}'}"
    """

    assert run_json_query_expr(None, query) == "stuff and things and things and stuff"


def test_remove_from_dict():
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "{*., -b}") == {"a": 1, "c": 3}
    assert run_json_query_expr({"a": 1}, "{*., -missing_key}") == {"a": 1}


def test_len():
    assert run_json_query_expr([1, 1, 1, 1, 1], "len") == 5


def test_entries():
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "entries") == [
        {"key": "a", "val": 1},
        {"key": "b", "val": 2},
        {"key": "c", "val": 3},
    ]
    assert run_json_query_expr(
        {"a": 1, "b": 2, "c": 3}, "entries | map [.key, .val]"
    ) == [
        ["a", 1],
        ["b", 2],
        ["c", 3],
    ]


def test_todict():
    assert run_json_query_expr(
        [{"key": "a", "val": 1}, {"key": "b", "val": 2}, {"key": "c", "val": 3}],
        "todict",
    ) == {"a": 1, "b": 2, "c": 3}

    assert run_json_query_expr(
        [["a", 1], ["b", 2], ["c", 3]],
        "todict",
    ) == {"a": 1, "b": 2, "c": 3}


def test_keys():
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "keys") == ["a", "b", "c"]


def test_values():
    assert run_json_query_expr({"a": 1, "b": 2, "c": 42}, "values") == [1, 2, 42]


def test_group():
    assert run_json_query_expr(
        ["a", "a", "a", "b", "c", "c"], "group [., {key: .key, count: .rows | len}]"
    ) == [
        {"key": "a", "count": 3},
        {"key": "b", "count": 1},
        {"key": "c", "count": 2},
    ]

    assert run_json_query_expr(
        [
            {"name": "A", "value": 5},
            {"name": "B", "value": 105},
            {"name": "A", "value": 1},
            {"name": "B", "value": 102},
            {"name": "A", "value": 2},
        ],
        "group [.name, {name: .key, value: .rows | map .value | sum}]",
    ) == [
        {"name": "A", "value": 8},
        {"name": "B", "value": 207},
    ]

    # group by a dict
    assert run_json_query_expr(
        [
            {"g0": 1, "g1": 1, "val": 1},
            {"g0": 1, "g1": 1, "val": 2},
            {"g0": 1, "g1": 2, "val": 3},
            {"g0": 2, "g1": 1, "val": 4},
            {"g0": 2, "g1": 1, "val": 5},
        ],
        "group [{.g0, .g1}, {*.key, vals: .rows | map .val}]",
    ) == [
        {"g0": 1, "g1": 1, "vals": [1, 2]},
        {"g0": 1, "g1": 2, "vals": [3]},
        {"g0": 2, "g1": 1, "vals": [4, 5]},
    ]


def test_sum():
    assert run_json_query_expr([1, 4, 7], "sum") == 1 + 4 + 7


def test_sort():
    assert run_json_query_expr(
        [1, 4, 2, 3],
        "sort .",
    ) == [1, 2, 3, 4]
    assert run_json_query_expr(
        ["a", "d", "b", "c"],
        "sort .",
    ) == ["a", "b", "c", "d"]
    assert run_json_query_expr(
        [
            {"val": 1},
            {"val": 4},
            {"val": 2},
            {"val": 3},
        ],
        "sort .val",
    ) == [
        {"val": 1},
        {"val": 2},
        {"val": 3},
        {"val": 4},
    ]

    assert run_json_query_expr(
        [
            [1, 12],
            [100, 2],
            [100, 1],
            [1, 11],
        ],
        "sort .",
    ) == [
        [1, 11],
        [1, 12],
        [100, 1],
        [100, 2],
    ]


def test_fruits_with_most_common_farm():
    data = [
        {"fruit": "grape", "farm": "B", "amount": 1},
        {"fruit": "grape", "farm": "C", "amount": 2},
        {"fruit": "grape", "farm": "A", "amount": 3},
        {"fruit": "grape", "farm": "A", "amount": 4},
        {"fruit": "pear", "farm": "Z", "amount": 22},
    ]
    assert (
        run_json_query_expr(
            data,
            """
            group [
                .fruit,
                {
                    fruit: .key,
                    total_amount: .rows | map .amount | sum,
                    *.rows | group [
                        .farm,
                        {farm: .key, count: .rows | len}
                    ] | sort -.count | {most_common_farm: .[0].farm}
                }
            ]
            """,
        )
        == [
            {"fruit": "grape", "total_amount": 10, "most_common_farm": "A"},
            {"fruit": "pear", "total_amount": 22, "most_common_farm": "Z"},
        ]
    )


def test_string_literal_dict_key():
    assert run_json_query_expr(1, "{'len': 1}") == {"len": 1}
    assert run_json_query_expr(1, "{['len']: 1}") == {"len": 1}

    # and an extra one just to make sure that "len" is not a keyword
    assert run_json_query_expr(1, "{len: 1}") == {"len": 1}


def test_direct_function_composition():
    assert run_json_query_expr(
        [
            [-1, 2, 3, -4],
            [-4],
            [1, 2, -100, 3],
        ],
        "map filter . > 0",
    ) == [
        [2, 3],
        [],
        [1, 2, 3],
    ]

    assert run_json_query_expr(
        [
            [-1, 2, 3, -4],
            [-4],
            [1, 2, -100, 3],
        ],
        "map (filter . > 0 | len)",
    ) == [2, 0, 3]


def test_access_shortcut():
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "{.a, .c}") == {"a": 1, "c": 3}
    assert run_json_query_expr({"a": 1, "b": 2, "c": 3}, "{.['a'], .['c']}") == {
        "a": 1,
        "c": 3,
    }
    # assert run_json_query_expr({"a": {"b": 42}}, "{ .a.b }") == {"b": 42}
    # assert run_json_query_expr({"a": {"b": 42}}, "{ (.a).b }") == {"b": 42}
    # assert run_json_query_expr({"a": {"b": {"c": 42}}}, "{ .a.b.c }") == {"c": 42}


def test_split():
    assert run_json_query_expr("stuff and  things", "split ' '") == [
        "stuff",
        "and",
        "",
        "things",
    ]

    # calling split with no args will split on whitespace
    assert run_json_query_expr("stuff and  \n\t\r  things", "split") == [
        "stuff",
        "and",
        "things",
    ]


def test_join():
    assert run_json_query_expr(["a", "b", "c"], 'join "--"') == "a--b--c"
    assert run_json_query_expr(["a"], 'join "--"') == "a"
    assert run_json_query_expr([], 'join "--"') == ""


def test_join_lines():
    assert run_json_query_expr(["a", "b", "c"], "joinlines") == "a\nb\nc\n"
    assert run_json_query_expr(["a"], "joinlines") == "a\n"
    assert run_json_query_expr([], "joinlines") == ""


def test_parse():
    assert run_json_query_expr(json.dumps({"a": 1, "b": 2}), "parse") == {
        "a": 1,
        "b": 2,
    }


def test_trim():
    assert run_json_query_expr("  stuff     \n\t\r", "trim") == "stuff"


def test_lines():
    assert run_json_query_expr("line 1\nline 2 \n\nline 4", "lines") == [
        "line 1",
        "line 2 ",
        "",
        "line 4",
    ]


def test_sort_no_args():
    assert run_json_query_expr([5, 2, 4, 1], "sort") == [1, 2, 4, 5]

    assert run_json_query_expr(
        [
            [1, 12],
            [100, 2],
            [100, 1],
            [1, 11],
        ],
        "sort",
    ) == [
        [1, 11],
        [1, 12],
        [100, 1],
        [100, 2],
    ]


def test_lower():
    assert run_json_query_expr("AbcDe", "lower") == "abcde"


def test_number():
    assert run_json_query_expr("1234", "number") == 1234


def test_null():
    assert run_json_query_expr(1, "null") == None


def test_false():
    assert run_json_query_expr(1, "false") == False


def test_true():
    assert run_json_query_expr(1, "true") == True


MOCK_RECURSIVE_OBJ = {
    "name": "a",
    "children": [
        {"name": "b", "children": []},
        {
            "name": "c",
            "children": [
                {"name": "d", "children": []},
                {"name": "e", "children": []},
            ],
        },
    ],
}


def test_recursivemap():
    assert run_json_query_expr(
        MOCK_RECURSIVE_OBJ,
        "recursivemap [.children, {name2: .node.name, count: .vals | len, children2: .vals}]",
    ) == {
        "name2": "a",
        "count": 2,
        "children2": [
            {"name2": "b", "count": 0, "children2": []},
            {
                "name2": "c",
                "count": 2,
                "children2": [
                    {"name2": "d", "count": 0, "children2": []},
                    {"name2": "e", "count": 0, "children2": []},
                ],
            },
        ],
    }


def test_recursiveflatten():
    assert run_json_query_expr(
        MOCK_RECURSIVE_OBJ, "recursiveflatten .children | map .name"
    ) == [
        "a",
        "b",
        "c",
        "d",
        "e",
    ]


TRUTH_TABLE = [
    [False, False],
    [False, True],
    [True, False],
    [True, True],
]


def test_or():
    assert run_json_query_expr([1, 2, 3, 4, 5, 6], "filter . < 3 or . > 4") == [
        1,
        2,
        5,
        6,
    ]

    assert run_json_query_expr(TRUTH_TABLE, "map .[0] or .[1]") == [
        False,
        True,
        True,
        True,
    ]


def test_and():
    assert run_json_query_expr(TRUTH_TABLE, "map .[0] and .[1]") == [
        False,
        False,
        False,
        True,
    ]


def test_and_and_or():
    assert run_json_query_expr(
        [1, 2, 3, 4, 5, 6, 7], "filter . >= 6 or 2 <= . and . <= 4"
    ) == [2, 3, 4, 6, 7]

    # make sure expression is different with different associativity
    assert run_json_query_expr(
        [1, 2, 3, 4, 5, 6, 7], "filter (. >= 6 or 2 <= .) and . <= 4"
    ) == [2, 3, 4]


def test_add_and_subtract():
    assert run_json_query_expr(None, "42+100") == 142
    assert run_json_query_expr(None, "42+-100") == -58
    assert run_json_query_expr(None, "1+2+-10") == -7
    assert run_json_query_expr(None, "1+-2+10") == 9


def test_mul_and_div():
    assert run_json_query_expr(None, "3*4") == 12
    assert run_json_query_expr(None, "8/2") == 4
    assert run_json_query_expr(None, "3*5/3") == 5
    assert run_json_query_expr(None, "3 * -4 + 100*2") == 3 * -4 + 100 * 2


def test_flatten():
    assert run_json_query_expr([[1, 2], [3], [], [4, 5, 6]], "flatten") == [
        1,
        2,
        3,
        4,
        5,
        6,
    ]


def test_optional_default():
    assert run_json_query_expr([None, 1, "stuff"], "map . ?? 'default'") == [
        "default",
        1,
        "stuff",
    ]

    assert run_json_query_expr([None, [1, 2, 3]], "map (. ?? [] | len)") == [0, 3]


def test_if():
    assert run_json_query_expr(
        [False, True, False], 'map if [., "true value", "false value"]'
    ) == ["false value", "true value", "false value"]

    assert run_json_query_expr(
        [1, 2, 3, 4, 5, 6], 'map if [.>3, "greater than 3", "less than 3"]'
    ) == [
        "less than 3",
        "less than 3",
        "less than 3",
        "greater than 3",
        "greater than 3",
        "greater than 3",
    ]


def test_all():
    assert run_json_query_expr(
        [
            [],
            [True],
            [False],
            [False, False],
            [False, True],
            [True, False],
            [True, True],
        ],
        "map all",
    ) == [True, True, False, False, False, False, True]


def test_any():
    assert run_json_query_expr(
        [
            [],
            [True],
            [False],
            [False, False],
            [False, True],
            [True, False],
            [True, True],
        ],
        "map any",
    ) == [False, True, False, False, True, True, True]


def test_not():
    assert run_json_query_expr([False, True], "map not .") == [True, False]
    assert run_json_query_expr([False, True], "map (. | not)") == [True, False]


def test_combinations():
    assert run_json_query_expr([[1, 2], ["a", "b"]], "combinations") == [
        [1, "a"],
        [1, "b"],
        [2, "a"],
        [2, "b"],
    ]


def test_zip():
    assert run_json_query_expr([[1, 2], ["a", "b"]], "zip") == [[1, "a"], [2, "b"]]
