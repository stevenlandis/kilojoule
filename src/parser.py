from src import ast
from src.first_use_cache import FirstUseCache
from src.parser_generator import ParserRule, Parser, TokenGroup


class ParserCache(FirstUseCache):
    def getter(self):
        inner_single_quote_pattern = r"(?:[^'\\{}]|\\.)*"
        inner_double_quote_pattern = r'(?:[^"\\{}]|\\.)*'

        keyword_map = {
            "null": "NULL",
            "true": "TRUE",
            "false": "FALSE",
            "and": "AND",
            "or": "OR",
        }

        return Parser(
            [
                TokenGroup(["INTEGER"], r"[0-9]+"),
                TokenGroup(["FLOAT"], r"[0-9]+(?:\.[0-9]+)?"),
                TokenGroup(
                    ["IDENTIFIER", *keyword_map.values()],
                    r"[_A-Za-z][_A-Za-z0-9]*",
                    lambda text: (
                        keyword_map[text] if text in keyword_map else "IDENTIFIER"
                    ),
                ),
                TokenGroup(["DOT"], r"\."),
                TokenGroup(["COMMA"], r"\,"),
                TokenGroup(["COLON"], r"\:"),
                TokenGroup(["PIPE"], r"\|"),
                TokenGroup(["ASTERISK"], r"\*"),
                TokenGroup(["DIVIDE"], r"/"),
                TokenGroup(["PLUS"], r"\+"),
                TokenGroup(["MINUS"], r"\-"),
                TokenGroup(["LEFT_PAREN"], r"\("),
                TokenGroup(["RIGHT_PAREN"], r"\)"),
                TokenGroup(["LEFT_BRACKET"], r"\["),
                TokenGroup(["RIGHT_BRACKET"], r"\]"),
                TokenGroup(["LEFT_BRACE"], r"\{"),
                TokenGroup(["RIGHT_BRACE"], r"\}"),
                TokenGroup(["LESS_THAN_OR_EQUAL"], r"\<\="),
                TokenGroup(["LESS_THAN"], r"\<"),
                TokenGroup(["GREATER_THAN_OR_EQUAL"], r"\>\="),
                TokenGroup(["GREATER_THAN"], r"\>"),
                TokenGroup(["EQUAL"], r"\="),
                TokenGroup(["DOUBLE_EQUAL"], r"\=\="),
                TokenGroup(["NOT_EQUAL"], r"\!\="),
                TokenGroup(["DOUBLE_QUESTION"], r"\?\?"),
                TokenGroup(
                    ["STRING_SINGLE_QUOTE"], r"'" + inner_single_quote_pattern + r"'"
                ),
                TokenGroup(
                    ["F_STRING_SINGLE_QUOTE_LEFT"],
                    r"'" + inner_single_quote_pattern + r"{",
                ),
                TokenGroup(
                    ["F_STRING_SINGLE_QUOTE_MIDDLE"],
                    (r"}" + inner_single_quote_pattern + r"{"),
                ),
                TokenGroup(
                    ["F_STRING_SINGLE_QUOTE_RIGHT"],
                    r"}" + inner_single_quote_pattern + r"'",
                ),
                TokenGroup(
                    ["STRING_DOUBLE_QUOTE"], r'"' + inner_double_quote_pattern + r'"'
                ),
                TokenGroup(
                    ["F_STRING_DOUBLE_QUOTE_LEFT"],
                    r'"' + inner_double_quote_pattern + r"{",
                ),
                TokenGroup(
                    ["F_STRING_DOUBLE_QUOTE_MIDDLE"],
                    r"}" + inner_double_quote_pattern + r"{",
                ),
                TokenGroup(
                    ["F_STRING_DOUBLE_QUOTE_RIGHT"],
                    r"}" + inner_double_quote_pattern + r'"',
                ),
            ],
            "expr",
            [
                ParserRule(
                    "expr",
                    [["op_pipe_expr"]],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "op_pipe_expr",
                    [
                        ["op_unary_fcn_expr"],
                        ["op_pipe_expr", "PIPE", "op_unary_fcn_expr"],
                    ],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_Pipe(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "op_unary_fcn_expr",
                    [["op_option_default_expr"], ["IDENTIFIER", "op_unary_fcn_expr"]],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_UnaryFcn(elems[0], elems[1])
                    ),
                ),
                ParserRule(
                    "op_option_default_expr",
                    [
                        ["op_or_expr"],
                        ["op_option_default_expr", "DOUBLE_QUESTION", "op_or_expr"],
                    ],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_OptionalDefault(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "op_or_expr",
                    [["op_and_expr"], ["op_or_expr", "OR", "op_and_expr"]],
                    lambda elems: (
                        elems[0] if len(elems) == 1 else ast.Expr_Or(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "op_and_expr",
                    [["op_equality_expr"], ["op_and_expr", "AND", "op_equality_expr"]],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_And(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "equality_operator",
                    [
                        ["LESS_THAN"],
                        ["LESS_THAN_OR_EQUAL"],
                        ["GREATER_THAN"],
                        ["GREATER_THAN_OR_EQUAL"],
                        ["DOUBLE_EQUAL"],
                        ["NOT_EQUAL"],
                    ],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "op_equality_expr",
                    [
                        ["op_add_expr"],
                        ["op_add_expr", "equality_operator", "op_add_expr"],
                    ],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else (
                            {
                                "<": ast.Expr_LessThan,
                                "<=": ast.Expr_LessThanOrEqual,
                                ">": ast.Expr_GreaterThan,
                                ">=": ast.Expr_GreaterThanOrEqual,
                                "==": ast.Expr_Equals,
                                "!=": ast.Expr_NotEqual,
                            }[elems[1]]
                        )(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "addition_operator",
                    [
                        ["PLUS"],
                    ],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "op_add_expr",
                    [
                        ["op_mul_expr"],
                        ["op_add_expr", "addition_operator", "op_mul_expr"],
                    ],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_Add(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "mul_operator",
                    [
                        ["ASTERISK"],
                        ["DIVIDE"],
                    ],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "op_mul_expr",
                    [
                        ["op_minus_expr"],
                        ["op_mul_expr", "mul_operator", "op_minus_expr"],
                    ],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else (
                            {
                                "*": ast.Expr_Mul,
                                "/": ast.Expr_Div,
                            }[elems[1]]
                        )(elems[0], elems[2])
                    ),
                ),
                ParserRule(
                    "op_minus_expr",
                    [["op_no_arg_fcn_expr"], ["MINUS", "op_no_arg_fcn_expr"]],
                    lambda elems: (
                        elems[0] if len(elems) == 1 else ast.Expr_Negate(elems[1])
                    ),
                ),
                ParserRule(
                    "op_no_arg_fcn_expr",
                    [["no_arg_fcn_expr"], ["op_dot_expr"]],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "op_dot_expr",
                    [["op_base_expr"], ["op_dot_expr", "access_node"]],
                    lambda elems: (
                        elems[0]
                        if len(elems) == 1
                        else ast.Expr_Pipe(elems[0], elems[1])
                    ),
                ),
                ParserRule(
                    "op_base_expr",
                    [
                        ["base_dot_expr"],
                        ["base_dot_access"],
                        ["string_literal"],
                        ["format_string"],
                        ["number_literal"],
                        ["array_expr"],
                        ["dict_expr"],
                        ["null_expr"],
                        ["true_expr"],
                        ["false_expr"],
                        ["LEFT_PAREN", "expr", "RIGHT_PAREN"],
                    ],
                    lambda elems: elems[0] if len(elems) == 1 else elems[1],
                ),
                ParserRule("null_expr", [["NULL"]], lambda _: ast.Expr_Null()),
                ParserRule("true_expr", [["TRUE"]], lambda _: ast.Expr_True()),
                ParserRule("false_expr", [["FALSE"]], lambda _: ast.Expr_False()),
                ParserRule(
                    "no_arg_fcn_expr",
                    [["IDENTIFIER"]],
                    lambda elems: ast.Expr_NoArgFcn(elems[0]),
                ),
                ParserRule(
                    "base_dot_expr",
                    [["DOT"]],
                    lambda elems: ast.Expr_Echo(),
                ),
                ParserRule(
                    "access_node",
                    [["base_dot_access"], ["base_bracket_access"]],
                    lambda elems: elems[0] if len(elems) == 1 else elems[1],
                ),
                ParserRule(
                    "base_dot_access",
                    [["DOT", "IDENTIFIER"]],
                    lambda elems: ast.Expr_ExprAccess(ast.Expr_StringLiteral(elems[1])),
                ),
                ParserRule(
                    "base_bracket_access",
                    [["LEFT_BRACKET", "bracket_access_inner_expr", "RIGHT_BRACKET"]],
                    lambda elems: elems[1],
                ),
                ParserRule(
                    "bracket_access_inner_expr",
                    [
                        ["expr"],
                        ["COLON", "expr"],
                        ["expr", "bracket_access_inner_expr_colon_right"],
                    ],
                    lambda elems: (
                        ast.Expr_ExprAccess(elems[0])
                        if len(elems) == 1
                        else (
                            ast.Expr_RangeAccessEnd(elems[1])
                            if elems[0] == ":"
                            else (
                                ast.Expr_RangeAccessStart(elems[0])
                                if len(elems[1]) == 1
                                else ast.Expr_RangeAccessStartEnd(elems[0], elems[1][1])
                            )
                        )
                    ),
                ),
                ParserRule(
                    "bracket_access_inner_expr_colon_right",
                    [["COLON"], ["COLON", "expr"]],
                    lambda elems: elems,
                ),
                ParserRule(
                    "array_expr",
                    [
                        ["LEFT_BRACKET", "RIGHT_BRACKET"],
                        ["LEFT_BRACKET", "array_expr_contents", "RIGHT_BRACKET"],
                    ],
                    lambda elems: (
                        ast.Expr_Array([])
                        if len(elems) == 2
                        else ast.Expr_Array(elems[1])
                    ),
                ),
                ParserRule(
                    "array_expr_contents",
                    [
                        ["array_element"],
                        ["array_expr_contents", "COMMA", "array_element"],
                    ],
                    lambda elems: (
                        [elems[0]] if len(elems) == 1 else [*elems[0], elems[2]]
                    ),
                ),
                ParserRule(
                    "array_element",
                    [
                        ["expr"],
                        ["ASTERISK", "expr"],
                    ],
                    lambda elems: elems[0] if len(elems) == 1 else ast.Spread(elems[1]),
                ),
                ParserRule(
                    "dict_expr",
                    [
                        ["LEFT_BRACE", "RIGHT_BRACE"],
                        ["LEFT_BRACE", "dict_contents", "RIGHT_BRACE"],
                    ],
                    lambda elems: (
                        ast.Expr_Dict([])
                        if len(elems) == 2
                        else ast.Expr_Dict(elems[1])
                    ),
                ),
                ParserRule(
                    "dict_contents",
                    [
                        ["dict_contents_elem"],
                        ["dict_contents", "COMMA", "dict_contents_elem"],
                    ],
                    lambda elems: (
                        [elems[0]] if len(elems) == 1 else [*elems[0], elems[2]]
                    ),
                ),
                ParserRule(
                    "dict_contents_elem",
                    [
                        ["dict_elem_kv_pair"],
                        ["dict_elem_string_literal_kv_pair"],
                        ["dict_elem_bracket_kv_pair"],
                        ["dict_elem_spread"],
                        ["dict_elem_omit"],
                        ["dict_elem_access_shortcut"],
                    ],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "dict_elem_kv_pair",
                    [["IDENTIFIER", "COLON", "expr"]],
                    lambda elems: ast.DictKvPair(
                        ast.Expr_StringLiteral(elems[0]), elems[2]
                    ),
                ),
                ParserRule(
                    "dict_elem_string_literal_kv_pair",
                    [["string_literal", "COLON", "expr"]],
                    lambda elems: ast.DictKvPair(elems[0], elems[2]),
                ),
                ParserRule(
                    "dict_elem_bracket_kv_pair",
                    [["LEFT_BRACKET", "expr", "RIGHT_BRACKET", "COLON", "expr"]],
                    lambda elems: ast.DictKvPair(elems[1], elems[4]),
                ),
                ParserRule(
                    "dict_elem_spread",
                    [["ASTERISK", "expr"]],
                    lambda elems: ast.Spread(elems[1]),
                ),
                ParserRule(
                    "dict_elem_omit",
                    [["MINUS", "IDENTIFIER"]],
                    lambda elems: ast.DictOmit(elems[1]),
                ),
                ParserRule(
                    "dict_elem_access_shortcut",
                    [
                        ["base_dot_access"],
                        ["DOT", "LEFT_BRACKET", "expr", "RIGHT_BRACKET"],
                    ],
                    lambda elems: (
                        ast.DictAccessShortcut(ast.Expr_Echo(), elems[0].expr)
                        if len(elems) == 1
                        else ast.DictAccessShortcut(ast.Expr_Echo(), elems[2])
                    ),
                ),
                ParserRule(
                    "string_literal",
                    [["single_quote_string_literal"], ["double_quote_string_literal"]],
                    lambda elems: elems[0],
                ),
                ParserRule(
                    "single_quote_string_literal",
                    [["STRING_SINGLE_QUOTE"]],
                    lambda elems: ast.Expr_StringLiteral(
                        escape_string_literal(elems[0][1:-1])
                    ),
                ),
                ParserRule(
                    "double_quote_string_literal",
                    [["STRING_DOUBLE_QUOTE"]],
                    lambda elems: ast.Expr_StringLiteral(
                        escape_string_literal(elems[0][1:-1])
                    ),
                ),
                ParserRule(
                    "format_string",
                    [
                        [
                            "F_STRING_SINGLE_QUOTE_LEFT",
                            "inner_format_string_single_quote",
                            "F_STRING_SINGLE_QUOTE_RIGHT",
                        ],
                        [
                            "F_STRING_DOUBLE_QUOTE_LEFT",
                            "inner_format_string_double_quote",
                            "F_STRING_DOUBLE_QUOTE_RIGHT",
                        ],
                    ],
                    lambda elems: ast.Expr_FormatString(
                        [
                            ast.Expr_StringLiteral(
                                escape_string_literal(elems[0][1:-1])
                            ),
                            *elems[1],
                            ast.Expr_StringLiteral(
                                escape_string_literal(elems[2][1:-1])
                            ),
                        ]
                    ),
                ),
                ParserRule(
                    "inner_format_string_single_quote",
                    [
                        ["expr"],
                        [
                            "inner_format_string_single_quote",
                            "F_STRING_SINGLE_QUOTE_MIDDLE",
                            "expr",
                        ],
                    ],
                    lambda elems: (
                        [elems[0]]
                        if len(elems) == 1
                        else [
                            *elems[0],
                            ast.Expr_StringLiteral(
                                escape_string_literal(elems[1][1:-1])
                            ),
                            elems[2],
                        ]
                    ),
                ),
                ParserRule(
                    "inner_format_string_double_quote",
                    [
                        ["expr"],
                        [
                            "inner_format_string_double_quote",
                            "F_STRING_DOUBLE_QUOTE_MIDDLE",
                            "expr",
                        ],
                    ],
                    lambda elems: (
                        [elems[0]]
                        if len(elems) == 1
                        else [
                            *elems[0],
                            ast.Expr_StringLiteral(
                                escape_string_literal(elems[1][1:-1])
                            ),
                            elems[2],
                        ]
                    ),
                ),
                ParserRule(
                    "number_literal",
                    [["INTEGER"], ["FLOAT"]],
                    lambda elems: (
                        ast.Expr_NumberLiteral(float(elems[0]))
                        if len(elems) == 1
                        else ast.Expr_NumberLiteral(float(f"{elems[0]}"))
                    ),
                ),
            ],
            ignore_pattern=r"[ \n\t\r]*",
        )


PARSER_CACHE = ParserCache()

HARDCODED_ESCAPE_MAP = {
    "n": "\n",
    "t": "\t",
    "r": "\r",
    '"': '"',
    "'": "'",
    "{": "{",
    "}": "}",
}


def escape_string_literal(text: str):
    result = ""
    idx = 0
    while idx < len(text):
        char = text[idx]
        if char == "\\":
            result += HARDCODED_ESCAPE_MAP[text[idx + 1]]
            idx += 1
        else:
            result += char
        idx += 1

    return result


def parse(text: str) -> ast.Expr:
    return PARSER_CACHE.get().parse(text)
