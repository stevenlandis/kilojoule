from dataclasses import dataclass
import json
import re
from typing import Any, Callable, Optional, Union


@dataclass(frozen=True)
class CfgProd:
    steps: tuple[str, ...]


@dataclass(frozen=True)
class CfgRule:
    name: str
    productions: tuple[CfgProd, ...]


@dataclass(frozen=True)
class RuleState:
    rule_name: str
    prod_idx: int
    step_idx: int

    def __lt__(self, other: "RuleState"):
        return (self.rule_name, self.prod_idx, self.step_idx) < (
            other.rule_name,
            other.prod_idx,
            other.step_idx,
        )


@dataclass(frozen=True)
class RuleSourceRow:
    state: RuleState
    next_tokens: tuple[str, ...]

    def __lt__(self, other: "RuleSourceRow"):
        return (self.state, self.next_tokens) < (other.state, other.next_tokens)


@dataclass(frozen=True)
class RuleStateTable:
    rows: tuple[RuleSourceRow, ...]

    def get_possible_next_rows(
        self,
        rule_map: dict[str, CfgRule],
        token_to_first_tokens: dict[str, tuple[str, ...]],
    ):
        tokens_to_row_idx: dict[tuple[str, str], int] = {}

        for row_idx, row in enumerate(self.rows):
            state = row.state
            steps = rule_map[state.rule_name].productions[state.prod_idx].steps
            token = steps[state.step_idx]

            if state.step_idx + 1 < len(steps):
                next_tokens = token_to_first_tokens[steps[state.step_idx + 1]]
            else:
                next_tokens = row.next_tokens

            for lookahead in next_tokens:
                token_key = (token, lookahead)
                assert (
                    token_key not in tokens_to_row_idx
                ), f"Rows with the same accept token cannot have overlapping next tokens.\n(token, lookahead) = {token_key}\nexisting row: {self.rows[tokens_to_row_idx[token_key]]}\nnew row:      {row}"
                tokens_to_row_idx[token_key] = row_idx

        tokens_to_next_table: dict[
            tuple[str, str], tuple[RuleSourceRow, ReduceAction]
        ] = {}
        for (token, lookahead), row_idx in tokens_to_row_idx.items():
            row = self.rows[row_idx]
            state = row.state
            steps = rule_map[state.rule_name].productions[state.prod_idx].steps
            if state.step_idx + 1 < len(steps):
                # reduce
                next_state = RuleState(
                    state.rule_name, state.prod_idx, state.step_idx + 1
                )
                next_row = RuleSourceRow(next_state, row.next_tokens)
                tokens_to_next_table[(token, lookahead)] = (next_row, ShiftAction())
            else:
                # shift
                tokens_to_next_table[(token, lookahead)] = (
                    RuleSourceRow(None, tuple()),
                    ReduceAction(state.rule_name, state.prod_idx, len(steps)),
                )

        return tokens_to_next_table


def get_table_from_row(
    rule_map: dict[str, CfgRule],
    token_to_first_tokens: dict[str, tuple[str, ...]],
    row: RuleSourceRow,
):
    if row.state is None:
        return RuleStateTable(tuple())

    state_to_next_map = {row.state: list(row.next_tokens)}
    processed_states: set[RuleState] = set()

    def helper(state_to_update: RuleState):
        if state_to_update in processed_states:
            return
        processed_states.add(state_to_update)

        steps = (
            rule_map[state_to_update.rule_name]
            .productions[state_to_update.prod_idx]
            .steps
        )
        step = steps[state_to_update.step_idx]

        if state_to_update.step_idx + 1 < len(steps):
            next_tokens = token_to_first_tokens[steps[state_to_update.step_idx + 1]]
        else:
            next_tokens = state_to_next_map[state_to_update]

        if step in rule_map:
            next_rule = rule_map[step]
            for prod_idx, _ in enumerate(next_rule.productions):
                next_state = RuleState(next_rule.name, prod_idx, 0)
                if next_state not in state_to_next_map:
                    state_to_next_map[next_state] = []
                state_to_next_map[next_state].append(next_tokens)
                helper(next_state)

    helper(row.state)

    next_rows = set(
        RuleSourceRow(state, normalize_set(deep_flatten(nexts)))
        for state, nexts in state_to_next_map.items()
    )

    return RuleStateTable(normalize_set(next_rows))


def deep_flatten(elements):
    result = set()
    reached = set()

    def helper(elem):
        if id(elem) in reached:
            return
        reached.add(id(elem))
        if isinstance(elem, (list, tuple)):
            for sub_elem in elem:
                helper(sub_elem)
        else:
            result.add(elem)

    helper(elements)
    return result


class Action:
    pass


@dataclass(frozen=True)
class ShiftAction(Action):
    pass


@dataclass(frozen=True)
class ReduceAction(Action):
    rule_name: str
    prod_idx: int
    n_tokens: int


@dataclass(frozen=True)
class LookupTableRow:
    state: int
    next_state: int
    token: str
    lookahead: str
    action: Action
    possible_lookaheads: tuple[str, ...]


def get_lookup_table_map(rules: list[CfgRule], start_rule: str):
    rule_map = {rule.name: rule for rule in rules}
    first_tokens_map = get_first_tokens(rules)

    lookup_table_rows: list[LookupTableRow] = []

    initial_row = RuleSourceRow(RuleState(start_rule, 0, 0), tuple())

    tables: list[RuleStateTable] = []
    row_to_tbl_idx: dict[RuleSourceRow, int] = {}

    def helper(row: RuleSourceRow):
        table_idx = row_to_tbl_idx.get(row)
        if table_idx is not None:
            return table_idx

        for test_tbl_idx, other_table in enumerate(tables):
            for test_row in other_table.rows:
                if (
                    row is not None
                    and row.state == test_row.state
                    and set(row.next_tokens) - set(test_row.next_tokens) == set()
                ):
                    table_idx = test_tbl_idx
                    break
            if table_idx is not None:
                break
        if table_idx is not None:
            row_to_tbl_idx[row] = table_idx
            return table_idx

        table_idx = len(tables)
        table = get_table_from_row(rule_map, first_tokens_map, row)
        tables.append(table)
        row_to_tbl_idx[row] = table_idx

        rows_grouped_by_token = {}
        for (token, lookahead), (next_row, action) in table.get_possible_next_rows(
            rule_map, first_tokens_map
        ).items():
            if token not in rows_grouped_by_token:
                rows_grouped_by_token[token] = []
            rows_grouped_by_token[token].append((lookahead, next_row, action))

        for token, rows in rows_grouped_by_token.items():
            _, next_row0, action0 = rows[0]
            first_row_key = (next_row0, action0)
            if all(
                (next_row, action) == first_row_key for _, next_row, action in rows[1:]
            ):
                # just append the first row
                next_tbl_idx = helper(next_row0)

                lookup_table_rows.append(
                    LookupTableRow(
                        table_idx,
                        next_tbl_idx,
                        token,
                        None,
                        action0,
                        normalize_set(set(lookahead for lookahead, _, _ in rows)),
                    )
                )
            else:
                # append all the rows since lookahead matters
                for lookahead, next_row, action in rows:
                    next_tbl_idx = helper(next_row)

                    lookup_table_rows.append(
                        LookupTableRow(
                            table_idx,
                            next_tbl_idx,
                            token,
                            lookahead,
                            action,
                            (lookahead,),
                        )
                    )

        return table_idx

    helper(initial_row)

    rows = sorted(set(lookup_table_rows), key=lambda row: (row.state, row.next_state))
    return {
        (row.state, row.token, row.lookahead): (
            row.next_state,
            row.action,
            row.possible_lookaheads,
        )
        for row in rows
    }


def get_first_tokens(rules: list[CfgRule]) -> dict[str, list[str]]:
    rule_map = {rule.name: rule for rule in rules}
    rule_to_firsts: dict[str, list] = {}

    def helper(token: str):
        if token in rule_to_firsts:
            return
        rule_to_firsts[token] = []

        if token in rule_map:
            for prod in rule_map[token].productions:
                step = prod.steps[0]
                helper(step)
                if step in rule_map:
                    helper(step)
                    rule_to_firsts[token].append(rule_to_firsts[step])
                else:
                    rule_to_firsts[token].append(step)
        else:
            rule_to_firsts[token].append(token)

    for rule in rule_map.values():
        helper(rule.name)
        for prod in rule.productions:
            for step in prod.steps:
                helper(step)

    return {
        rule: normalize_set(deep_flatten(firsts))
        for rule, firsts in rule_to_firsts.items()
    }


def normalize_set(things: set[str]):
    return tuple(sorted(things))


@dataclass(frozen=True)
class ParserRule:
    name: str
    patterns: list[list[str]]
    handler: Callable


@dataclass(frozen=True)
class TokenGroup:
    token_names: list[str]
    pattern: str
    callback: Optional[Callable] = None


class Tokenizer:
    def __init__(self, text: str, skip_pattern: Optional[re.Pattern] = None):
        self.text = text
        self.skip_pattern = skip_pattern
        self.start_idx = 0
        self.t0 = None
        self.t1 = None

    def peek(self, n: int, pattern: re.Pattern, process_clbk: Callable):
        assert 0 <= n and n <= 1
        if n == 0:
            if self.t0 is None:
                self.t0 = self.__parse(0, pattern, process_clbk)
            return self.t0

        if self.t1 is None:
            self.t1 = self.__parse(len(self.t0[1]), pattern, process_clbk)
        return self.t1

    def step(self):
        assert self.t0 is not None
        self.start_idx += len(self.t0[1])
        self.t0 = self.t1
        self.t1 = None

    def __parse(self, n: int, pattern: re.Pattern, process_clbk: Callable):
        if self.skip_pattern is not None:
            match = self.skip_pattern.match(self.text, self.start_idx + n)
            if match is not None:
                start_idx, end_idx = match.span()
                self.start_idx += end_idx - start_idx

        if self.start_idx + n >= len(self.text):
            return ("END", None)
        res = pattern.match(self.text, self.start_idx + n)
        assert (
            res is not None
        ), f"Unable to parse token at idx={self.start_idx + n}\npattern = {pattern}\ntext = {json.dumps(self.text[self.start_idx + n:self.start_idx + n+10])}"
        groups = res.groupdict()
        [(pattern_name, text)] = [
            (name, text) for name, text in groups.items() if text is not None
        ]
        start_idx, end_idx = res.span()
        assert (
            end_idx - start_idx
        ) > 0, f"The regex pattern for pattern_name={pattern_name} must match at least 1 character."
        matching_text = self.text[start_idx:end_idx]
        token_name = process_clbk(pattern_name, matching_text)
        return (token_name, self.text[start_idx:end_idx])


@dataclass(frozen=True)
class ElemWrapper:
    rule_name: str
    value: Any


class Parser:
    def __init__(
        self,
        token_groups: list[TokenGroup],
        start_rule: str,
        rules: list[ParserRule],
        ignore_pattern: Optional[Union[str, re.Pattern]] = None,
    ):
        # main_rule_name = secrets.token_hex(16)
        main_rule_name = "MAIN_RULE"
        end_token = "END"
        # end_token = secrets.token_hex(16)
        cfg_rules = [CfgRule(main_rule_name, [CfgProd([start_rule, end_token])])]

        group_idx_to_group: dict[int, TokenGroup] = {}
        token_to_group_idx: dict[str, int] = {}
        for group_idx, group in enumerate(token_groups):
            assert len(group.token_names) > 0

            # make sure that token groups with more than 1 token have a callback
            # to differentiate between the tokens
            if len(group.token_names) == 1:
                assert group.callback is None
            elif len(group.token_names) > 1:
                assert group.callback is not None

            group_idx_to_group[group_idx] = group
            for token in group.token_names:
                assert token not in token_to_group_idx
                token_to_group_idx[token] = group_idx

        handler_map: dict[tuple[str, int], Callable] = {}
        rule_map: dict[str, list[CfgProd]] = {}
        for rule in rules:
            if rule.name not in rule_map:
                rule_map[rule.name] = []
            for pattern in rule.patterns:
                pattern_idx = len(rule_map[rule.name])
                rule_map[rule.name].append(CfgProd(pattern))
                handler_map[(rule.name, pattern_idx)] = rule.handler

        for rule_name, prods in rule_map.items():
            cfg_rules.append(CfgRule(rule_name, tuple(prods)))

        lookup_map = get_lookup_table_map(cfg_rules, main_rule_name)

        next_token_lookups = {}
        for (state, token, _), (_, _, possible_lookaheads) in lookup_map.items():
            if state not in next_token_lookups:
                next_token_lookups[state] = {"tokens": set(), "lookaheads": {}}
            if token in token_to_group_idx:
                next_token_lookups[state]["tokens"].add(token)

            if token not in next_token_lookups[state]["lookaheads"]:
                next_token_lookups[state]["lookaheads"][token] = {"tokens": set()}
            for lookahead in possible_lookaheads:
                if token in token_to_group_idx and lookahead in token_to_group_idx:
                    next_token_lookups[state]["lookaheads"][token]["tokens"].add(
                        lookahead
                    )

        pattern_lookup = {}
        pattern_map = {}

        def get_pattern(tokens):
            key = normalize_set(set(token_to_group_idx[token] for token in tokens))
            if key not in pattern_lookup:
                pattern_idx = len(pattern_lookup)
                pattern_lookup[key] = pattern_idx
                ordered_token_groups = sorted(
                    key,
                    key=lambda group_idx: len(group_idx_to_group[group_idx].pattern),
                    reverse=True,
                )
                pattern_map[pattern_idx] = re.compile(
                    "|".join(
                        f"(?P<G{group_idx}>{group_idx_to_group[group_idx].pattern})"
                        for group_idx in ordered_token_groups
                    )
                )
            return pattern_lookup[key]

        for val0 in next_token_lookups.values():
            if len(val0["tokens"]) > 0:
                val0["pattern_idx"] = get_pattern(val0["tokens"])
            else:
                val0["pattern_idx"] = None

            for val1 in val0["lookaheads"].values():
                if len(val1["tokens"]) > 0:
                    val1["pattern_idx"] = get_pattern(val1["tokens"])
                else:
                    val1["pattern_idx"] = None

        self.lookup_map = lookup_map
        self.next_token_lookups = next_token_lookups
        self.handler_map = handler_map
        self.pattern_map = pattern_map
        self.ignore_pattern = (
            re.compile(ignore_pattern)
            if isinstance(ignore_pattern, str)
            else ignore_pattern
        )
        self.group_idx_to_group = group_idx_to_group

    def __get_pattern(self, pattern_idx: Optional[int]):
        if pattern_idx is None:
            return None
        return self.pattern_map[pattern_idx]

    def __get_token_name(self, group_name: str, text: str):
        group_idx = int(group_name[1:])
        group = self.group_idx_to_group[group_idx]
        if group.callback is None:
            assert len(group.token_names) == 1
            return group.token_names[0]
        return group.callback(text)

    def parse(self, text: str):
        tokenizer = Tokenizer(text, self.ignore_pattern)
        stack: list[Union[int, ElemWrapper]] = [0]
        while True:
            stack_top = stack[-1]
            if isinstance(stack_top, int):
                state = stack_top
                if state not in self.next_token_lookups:
                    break
                token = tokenizer.peek(
                    0,
                    self.__get_pattern(self.next_token_lookups[state]["pattern_idx"]),
                    self.__get_token_name,
                )
                assert token[0] in self.next_token_lookups[state]["tokens"]

                lookahead = tokenizer.peek(
                    1,
                    self.__get_pattern(
                        self.next_token_lookups[state]["lookaheads"][token[0]][
                            "pattern_idx"
                        ]
                    ),
                    self.__get_token_name,
                )
                assert (
                    lookahead[0] == "END"
                    or lookahead[0]
                    in self.next_token_lookups[state]["lookaheads"][token[0]]["tokens"]
                )

                if (state, token[0], None) in self.lookup_map:
                    next_state, action, _ = self.lookup_map[(state, token[0], None)]
                else:
                    next_state, action, _ = self.lookup_map[
                        (state, token[0], lookahead[0])
                    ]

                if isinstance(action, ShiftAction):
                    stack.append(ElemWrapper(token[0], token[1]))
                    stack.append(next_state)
                elif isinstance(action, ReduceAction):
                    elem_tokens = [ElemWrapper(token[0], token[1])]
                    for _ in range(action.n_tokens - 1):
                        stack.pop()
                        elem_tokens.append(stack.pop())
                    elem_tokens.reverse()
                    wrapper = ElemWrapper(
                        action.rule_name,
                        self.handler_map[(action.rule_name, action.prod_idx)](
                            # [
                            #     elem.value if isinstance(elem, ElemWrapper) else elem
                            #     for elem in elem_tokens
                            # ]
                            [elem.value for elem in elem_tokens]
                        ),
                    )
                    stack.append(wrapper)
                tokenizer.step()
            else:
                assert isinstance(stack_top, ElemWrapper)
                state = stack[-2]
                token = stack_top.rule_name
                lookahead = tokenizer.peek(
                    0,
                    None,  # Can be None because lookahead has already been parsed
                    self.__get_token_name,
                )

                if (state, token, None) in self.lookup_map:
                    next_state, action, _ = self.lookup_map[(state, token, None)]
                else:
                    next_state, action, _ = self.lookup_map[
                        (state, token, lookahead[0])
                    ]

                if isinstance(action, ShiftAction):
                    stack.append(next_state)
                elif isinstance(action, ReduceAction):
                    elem_tokens = [stack.pop()]
                    for _ in range(action.n_tokens - 1):
                        stack.pop()
                        elem_tokens.append(stack.pop())
                    elem_tokens.reverse()
                    wrapper = ElemWrapper(
                        action.rule_name,
                        self.handler_map[(action.rule_name, action.prod_idx)](
                            # [
                            #     elem.value if isinstance(elem, ElemWrapper) else elem
                            #     for elem in elem_tokens
                            # ]
                            [elem.value for elem in elem_tokens]
                        ),
                    )
                    stack.append(wrapper)

        assert len(stack) == 3
        result = stack[1]
        return result.value
