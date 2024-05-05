import itertools
import json
import os
import subprocess
import sys
from typing import Any
from src.ast import (
    DictAccessShortcut,
    DictOmit,
    Expr,
    Expr_Add,
    Expr_And,
    Expr_Array,
    Expr_Dict,
    DictKvPair,
    Expr_Div,
    Expr_Equals,
    Expr_False,
    Expr_FormatString,
    Expr_GreaterThan,
    Expr_GreaterThanOrEqual,
    Expr_LessThan,
    Expr_LessThanOrEqual,
    Expr_Mul,
    Expr_Negate,
    Expr_NoArgFcn,
    Expr_NotEqual,
    Expr_Null,
    Expr_NumberLiteral,
    Expr_OptionalDefault,
    Expr_Or,
    Expr_RangeAccessEnd,
    Expr_RangeAccessStart,
    Expr_RangeAccessStartEnd,
    Expr_StringLiteral,
    Expr_True,
    Expr_UnaryFcn,
    Spread,
    Expr_Echo,
    Expr_ExprAccess,
    Expr_Pipe,
)
from src.parser import parse


def run_json_query_expr(obj, query: str):
    expr = parse(query)
    return evaluate_expr(obj, expr)


class OutputNode:
    def __init__(self, value):
        self.value = value


def evaluate_expr(obj, expr: Expr):
    if isinstance(expr, Expr_Echo):
        return obj
    if isinstance(expr, Expr_ExprAccess):
        if obj is None:
            return None
        assert isinstance(obj, (list, dict))
        index = evaluate_expr(obj, expr.expr)
        if isinstance(obj, list):
            if isinstance(index, float) and index == int(index):
                index = int(index)
            return obj[index]
        if isinstance(obj, dict):
            assert isinstance(index, str)
            return obj.get(index)
    if isinstance(expr, Expr_RangeAccessStartEnd):
        assert isinstance(obj, list)
        start_idx = evaluate_expr_int(obj, expr.start)
        end_idx = evaluate_expr_int(obj, expr.end)
        return obj[start_idx:end_idx]
    if isinstance(expr, Expr_RangeAccessStart):
        assert isinstance(obj, list)
        start_idx = evaluate_expr_int(obj, expr.start)
        return obj[start_idx:]
    if isinstance(expr, Expr_RangeAccessEnd):
        assert isinstance(obj, list)
        end_idx = evaluate_expr_int(obj, expr.end)
        return obj[:end_idx]
    if isinstance(expr, Expr_StringLiteral):
        return expr.value
    if isinstance(expr, Expr_FormatString):
        result = ""
        for part in expr.parts:
            if isinstance(part, Expr_StringLiteral):
                result += part.value
            else:
                result += obj_to_str(evaluate_expr(obj, part))
        return result
    if isinstance(expr, Expr_NumberLiteral):
        return expr.value
    if isinstance(expr, Expr_Array):
        result = []
        for elem in expr.elements:
            if isinstance(elem, Spread):
                list_to_spread = evaluate_expr(obj, elem.expr)
                assert isinstance(list_to_spread, list)
                result.extend(list_to_spread)
            elif isinstance(elem, Expr):
                result.append(evaluate_expr(obj, elem))
            else:
                raise Exception(f"unreachable list elem: {elem}")
        return result
    if isinstance(expr, Expr_Dict):
        result = {}
        for elem in expr.elements:
            if isinstance(elem, DictKvPair):
                result[evaluate_expr(obj, elem.key)] = evaluate_expr(obj, elem.value)
            elif isinstance(elem, Spread):
                dict_to_spread = evaluate_expr(obj, elem.expr)
                assert isinstance(dict_to_spread, dict)
                result.update(dict_to_spread)
            elif isinstance(elem, DictOmit):
                if elem.key_to_omit in result:
                    del result[elem.key_to_omit]
            elif isinstance(elem, DictAccessShortcut):
                key_value = evaluate_expr(obj, elem.key_expr)
                assert isinstance(key_value, str)
                val_to_access = evaluate_expr(obj, elem.expr)
                assert isinstance(val_to_access, dict)
                result[key_value] = val_to_access.get(key_value)
            else:
                raise Exception(f"unreachable dict elem: {elem}")
        return result
    if isinstance(expr, Expr_UnaryFcn):
        if expr.fcn_name == "map":
            if isinstance(obj, list):
                return [evaluate_expr(elem, expr.expr) for elem in obj]
            if isinstance(obj, dict):
                return {
                    key: evaluate_expr(value, expr.expr) for key, value in obj.items()
                }
            raise Exception(f"Unable to map over object with type {type(obj)}")
        if expr.fcn_name == "filter":
            assert isinstance(obj, list)
            return [elem for elem in obj if evaluate_expr_bool(elem, expr.expr)]
        if expr.fcn_name == "group":
            assert isinstance(obj, list)
            assert (
                isinstance(expr.expr, Expr_Array)
                and len(expr.expr.elements) == 2
                and isinstance(expr.expr.elements[0], Expr)
                and isinstance(expr.expr.elements[1], Expr)
            )
            key_getter = expr.expr.elements[0]
            aggregator = expr.expr.elements[1]
            keys = [evaluate_expr(elem, key_getter) for elem in obj]
            tuple_keys = [deep_array_to_tuple(elem) for elem in keys]
            group_map: dict[Any, tuple[Any, list[Any]]] = {}
            for tuple_key, key, row in zip(tuple_keys, keys, obj):
                if tuple_key not in group_map:
                    group_map[tuple_key] = (key, [row])
                else:
                    group_map[tuple_key][1].append(row)
            return [
                evaluate_expr({"key": key, "rows": rows}, aggregator)
                for key, rows in group_map.values()
            ]
        if expr.fcn_name == "sort":
            assert isinstance(obj, list)
            result = [*obj]
            result.sort(
                key=lambda elem: deep_array_to_tuple(evaluate_expr(elem, expr.expr))
            )
            return result
        if expr.fcn_name == "split":
            assert isinstance(obj, str)
            split_str = evaluate_expr_str(obj, expr.expr)
            return obj.split(split_str)
        if expr.fcn_name == "join":
            assert isinstance(obj, list) and all(isinstance(elem, str) for elem in obj)
            arg = evaluate_expr_str(obj, expr.expr)
            return arg.join(obj)
        if expr.fcn_name == "exec":
            assert isinstance(expr.expr, Expr_Array) and len(expr.expr.elements) >= 1
            values = [evaluate_expr_str(obj, elem) for elem in expr.expr.elements]
            assert obj is None or isinstance(obj, str)
            proc = subprocess.Popen(
                values,
                stdin=None if obj is None else subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=sys.stderr,
            )
            if isinstance(obj, str):
                proc.stdin.write(obj.encode())
                proc.stdin.close()
            proc.wait()
            result = proc.stdout.read().decode()
            return result
        if expr.fcn_name == "recursivemap":
            assert isinstance(expr.expr, Expr_Array) and len(expr.expr.elements) == 2
            child_getter = expr.expr.elements[0]
            mapper = expr.expr.elements[1]

            def helper(node):
                return evaluate_expr(
                    {
                        "node": node,
                        "vals": [
                            helper(child) for child in evaluate_expr(node, child_getter)
                        ],
                    },
                    mapper,
                )

            return helper(obj)
        if expr.fcn_name == "recursiveflatten":
            child_getter = expr.expr
            results = []

            def helper(node):
                results.append(node)
                children = evaluate_expr(node, child_getter)
                if children is not None:
                    for child in children:
                        helper(child)

            helper(obj)
            return results
        if expr.fcn_name == "if":
            assert isinstance(expr.expr, Expr_Array) and len(expr.expr.elements) == 3
            cond_expr = expr.expr.elements[0]
            true_getter = expr.expr.elements[1]
            false_getter = expr.expr.elements[2]
            cond = evaluate_expr_bool(obj, cond_expr)
            if cond:
                return evaluate_expr(obj, true_getter)
            else:
                return evaluate_expr(obj, false_getter)
        if expr.fcn_name == "not":
            value = evaluate_expr_bool(obj, expr.expr)
            return not value
        if expr.fcn_name == "read":
            file_name = evaluate_expr_str(obj, expr.expr)
            with open(file_name) as fid:
                return fid.read()
        if expr.fcn_name == "write":
            assert isinstance(obj, str)
            file_name = evaluate_expr_str(obj, expr.expr)
            with open(file_name, "w") as fid:
                fid.write(obj)
            return obj
        raise Exception(f"Unreachable unary fcn: {expr}")
    if isinstance(expr, Expr_NoArgFcn):
        if expr.fcn_name == "len":
            assert isinstance(obj, list)
            return len(obj)
        if expr.fcn_name == "entries":
            assert isinstance(obj, dict)
            return [{"key": key, "val": val} for key, val in obj.items()]
        if expr.fcn_name == "todict":
            assert isinstance(obj, list)
            if all(
                isinstance(elem, dict) and isinstance(elem.get("key"), str)
                for elem in obj
            ):
                return {elem["key"]: elem.get("val") for elem in obj}
            if all(
                isinstance(elem, list) and len(elem) == 2 and isinstance(elem[0], str)
                for elem in obj
            ):
                return {key: val for key, val in obj}
            raise Exception(f"Unable to turn into map: {obj}")
        if expr.fcn_name == "keys":
            assert isinstance(obj, dict)
            return list(obj.keys())
        if expr.fcn_name == "values":
            assert isinstance(obj, dict)
            return list(obj.values())
        if expr.fcn_name == "sum":
            assert isinstance(obj, list)
            result = None
            for elem in obj:
                if elem is None:
                    continue
                else:
                    assert isinstance(elem, (int, float))
                    if result is None:
                        result = elem
                    else:
                        result += elem
            return result
        if expr.fcn_name == "in":
            return sys.stdin.read()
        if expr.fcn_name == "out":
            return OutputNode(obj)
        if expr.fcn_name == "parse":
            assert isinstance(obj, str)
            return json.loads(obj)
        if expr.fcn_name == "trim":
            assert isinstance(obj, str)
            return obj.strip()
        if expr.fcn_name == "lines":
            assert isinstance(obj, str)
            return obj.splitlines()
        if expr.fcn_name == "sort":
            return evaluate_expr(obj, Expr_UnaryFcn("sort", Expr_Echo()))
        if expr.fcn_name == "lower":
            assert isinstance(obj, str)
            return obj.lower()
        if expr.fcn_name == "upper":
            assert isinstance(obj, str)
            return obj.upper()
        if expr.fcn_name == "split":
            assert isinstance(obj, str)
            return obj.split()
        if expr.fcn_name == "number":
            assert isinstance(obj, str)
            return float(obj)
        if expr.fcn_name == "read":
            assert isinstance(obj, str)
            with open(obj) as fid:
                return fid.read()
        if expr.fcn_name == "isfile":
            assert isinstance(obj, str)
            return os.path.isfile(obj)
        if expr.fcn_name == "isdir":
            assert isinstance(obj, str)
            return os.path.isdir(obj)
        if expr.fcn_name == "exists":
            assert isinstance(obj, str)
            return os.path.exists(obj)
        if expr.fcn_name == "listdir":
            assert isinstance(obj, str)
            return os.listdir(obj)
        if expr.fcn_name == "joinlines":
            assert isinstance(obj, list) and all(isinstance(elem, str) for elem in obj)
            if len(obj) == 0:
                return ""
            return "\n".join(obj) + "\n"
        if expr.fcn_name == "env":
            return dict(os.environ)
        if expr.fcn_name == "flatten":
            assert isinstance(obj, list) and all(isinstance(elem, list) for elem in obj)
            return [elem for sub_list in obj for elem in sub_list]
        if expr.fcn_name == "all":
            assert isinstance(obj, list) and all(isinstance(elem, bool) for elem in obj)
            return all(obj)
        if expr.fcn_name == "any":
            assert isinstance(obj, list) and all(isinstance(elem, bool) for elem in obj)
            return any(obj)
        if expr.fcn_name == "not":
            assert isinstance(obj, bool)
            return not obj
        if expr.fcn_name == "sh":
            return run_shell(obj)
        if expr.fcn_name == "combinations":
            assert isinstance(obj, list) and all(isinstance(elem, list) for elem in obj)
            return [list(elem) for elem in itertools.product(*obj)]
        if expr.fcn_name == "zip":
            assert isinstance(obj, list) and all(isinstance(elem, list) for elem in obj)
            return [list(elem) for elem in zip(*obj)]
        raise Exception(f"Unreachable unary fcn: {expr}")
    if isinstance(expr, Expr_Pipe):
        return evaluate_expr(evaluate_expr(obj, expr.left), expr.right)
    if isinstance(expr, Expr_Or):
        left = evaluate_expr_bool(obj, expr.left)
        right = evaluate_expr_bool(obj, expr.right)
        return left or right
    if isinstance(expr, Expr_And):
        left = evaluate_expr_bool(obj, expr.left)
        right = evaluate_expr_bool(obj, expr.right)
        return left and right
    if isinstance(expr, Expr_Negate):
        val = evaluate_expr(obj, expr.expr)
        assert isinstance(val, (int, float))
        return -val
    if isinstance(expr, Expr_LessThan):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left < right
    if isinstance(expr, Expr_LessThanOrEqual):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left <= right
    if isinstance(expr, Expr_GreaterThan):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left > right
    if isinstance(expr, Expr_GreaterThanOrEqual):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left >= right
    if isinstance(expr, Expr_Equals):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left == right
    if isinstance(expr, Expr_NotEqual):
        left = evaluate_expr(obj, expr.left)
        right = evaluate_expr(obj, expr.right)
        return left != right
    if isinstance(expr, Expr_Add):
        left = evaluate_expr_int_float(obj, expr.left)
        right = evaluate_expr_int_float(obj, expr.right)
        return left + right
    if isinstance(expr, Expr_Mul):
        left = evaluate_expr_int_float(obj, expr.left)
        right = evaluate_expr_int_float(obj, expr.right)
        return left * right
    if isinstance(expr, Expr_Div):
        left = evaluate_expr_int_float(obj, expr.left)
        right = evaluate_expr_int_float(obj, expr.right)
        return left / right
    if isinstance(expr, Expr_OptionalDefault):
        left = evaluate_expr(obj, expr.left)
        if left is None:
            right = evaluate_expr(obj, expr.right)
            return right
        return left
    if isinstance(expr, Expr_Null):
        return None
    if isinstance(expr, Expr_True):
        return True
    if isinstance(expr, Expr_False):
        return False

    raise Exception(f"Unreachale expr: {expr}")


def evaluate_expr_bool(stack, expr: Expr):
    result = evaluate_expr(stack, expr)
    assert isinstance(result, bool)
    return result


def evaluate_expr_str(stack, expr: Expr):
    result = evaluate_expr(stack, expr)
    assert isinstance(result, str)
    return result


def evaluate_expr_int(stack, expr: Expr):
    result = evaluate_expr(stack, expr)
    if isinstance(result, float) and result == int(result):
        result = int(result)
    assert isinstance(result, int)
    return result


def evaluate_expr_int_float(stack, expr: Expr):
    result = evaluate_expr(stack, expr)
    assert isinstance(result, (int, float))
    return result


def obj_to_str(obj, indent=None):
    if isinstance(obj, str):
        return obj
    return json.dumps(deep_float_to_int(obj), separators=(",", ":"), indent=indent)


def deep_float_to_int(obj):
    if isinstance(obj, float):
        if obj == int(obj):
            return int(obj)
    if obj is None or isinstance(obj, (int, float, str)):
        return obj
    if isinstance(obj, dict):
        return {key: deep_float_to_int(value) for key, value in obj.items()}
    if isinstance(obj, list):
        return tuple(deep_float_to_int(elem) for elem in obj)
    raise Exception(f"Unable to intify {obj}")


class HashableDict:
    def __init__(self, obj: dict):
        self.obj = obj

    def __hash__(self):
        return hash(tuple(self.obj.items()))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, HashableDict):
            return False

        return self.obj == other.obj


def to_printable_str(obj):
    if isinstance(obj, str):
        return json.dumps(obj)
    else:
        return obj_to_str(obj, indent=2)


def deep_array_to_tuple(obj):
    if obj is None or isinstance(obj, (int, float, str)):
        return obj
    if isinstance(obj, dict):
        return HashableDict(
            {key: deep_array_to_tuple(value) for key, value in obj.items()}
        )
    if isinstance(obj, list):
        return tuple(deep_array_to_tuple(elem) for elem in obj)
    raise Exception(f"Unable to tuplify {obj}")


def run_shell(obj):
    import readline

    while True:
        query = input("> ")
        result = evaluate_expr(obj, parse(query))
        if isinstance(result, OutputNode):
            return result.value
        print(to_printable_str(result))
