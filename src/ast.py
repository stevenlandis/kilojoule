from dataclasses import dataclass
from typing import Union


class Expr:
    pass


@dataclass(frozen=True)
class Expr_Echo(Expr):
    pass


@dataclass(frozen=True)
class Expr_ExprAccess(Expr):
    expr: Expr


@dataclass(frozen=True)
class Expr_RangeAccessStartEnd(Expr):
    start: Expr
    end: Expr


@dataclass(frozen=True)
class Expr_RangeAccessStart(Expr):
    start: Expr


@dataclass(frozen=True)
class Expr_RangeAccessEnd(Expr):
    end: Expr


@dataclass(frozen=True)
class Spread:
    expr: Expr


@dataclass(frozen=True)
class DictOmit:
    key_to_omit: str


@dataclass(frozen=True)
class Expr_Array(Expr):
    elements: list[Union[Expr, Spread]]


@dataclass(frozen=True)
class DictKvPair:
    key: Expr
    value: Expr


@dataclass(frozen=True)
class DictAccessShortcut:
    expr: Expr
    key_expr: Expr


@dataclass(frozen=True)
class Expr_Dict(Expr):
    elements: list[Union[DictKvPair, Spread, DictOmit, DictAccessShortcut]]


@dataclass(frozen=True)
class Expr_UnaryFcn(Expr):
    fcn_name: str
    expr: Expr


@dataclass(frozen=True)
class Expr_NoArgFcn(Expr):
    fcn_name: str


@dataclass(frozen=True)
class Expr_Pipe(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_StringLiteral(Expr):
    value: str


@dataclass(frozen=True)
class Expr_FormatString(Expr):
    parts: Union[Expr, Expr_StringLiteral]


@dataclass(frozen=True)
class Expr_NumberLiteral(Expr):
    value: float


@dataclass(frozen=True)
class Expr_Negate(Expr):
    expr: Expr


@dataclass(frozen=True)
class Expr_LessThan(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_LessThanOrEqual(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_GreaterThan(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_GreaterThanOrEqual(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Equals(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_NotEqual(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Or(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_And(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Add(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Mul(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Div(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_OptionalDefault(Expr):
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Expr_Null(Expr):
    pass


@dataclass(frozen=True)
class Expr_True(Expr):
    pass


@dataclass(frozen=True)
class Expr_False(Expr):
    pass
