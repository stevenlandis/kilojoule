from dataclasses import dataclass
import json
import re
from typing import Any, Callable, Optional, Union


@dataclass(frozen=True)
class Rule:
    name: str
    steps: list[str]
    handler: Optional[Callable] = None


@dataclass(frozen=True)
class RuleState:
    rule: int
    step: int

    def incr(self):
        return RuleState(self.rule, self.step + 1)

    def __lt__(self, other: "RuleState"):
        return (self.rule, self.step) < (other.rule, other.step)


def expand_states(
    states: list[RuleState], rules: list[Rule], rule_map: dict[str, list[int]]
):
    reached_states: set[RuleState] = set()

    def helper(state: RuleState):
        if state in reached_states:
            return
        reached_states.add(state)

        # none of the states should be finished here
        assert state.step < len(rules[state.rule].steps)
        next_step = rules[state.rule].steps[state.step]
        if next_step in rule_map:
            for new_rule in rule_map[next_step]:
                helper(RuleState(new_rule, 0))

    for state in states:
        helper(state)

    all_states = tuple(sorted(reached_states))
    return all_states


def get_first_tokens(rules: list[Rule], rule_map: dict[str, list[int]]):
    rule_to_first_tokens: dict[str, set[str]] = {}

    def helper(rule_name: str):
        if rule_name in rule_to_first_tokens:
            return rule_to_first_tokens[rule_name]
        if rule_name not in rule_map:
            first_tokens = {rule_name}
            rule_to_first_tokens[rule_name] = first_tokens
            return first_tokens
        first_tokens = set()
        rule_to_first_tokens[rule_name] = first_tokens

        for rule_idx in rule_map[rule_name]:
            first_step = rules[rule_idx].steps[0]
            first_tokens.update(helper(first_step))

        return first_tokens

    for rule in rules:
        helper(rule.name)
        for token in rule.steps:
            helper(token)

    return rule_to_first_tokens
    # return {
    #     rule_name: tuple(sorted(firsts))
    #     for rule_name, firsts in rule_to_first_tokens.items()
    # }


def get_state_to_following_tokens(
    rules: list[Rule],
    rule_map: dict[str, list[int]],
    rule_to_first_tokens: dict[str, set[int]],
):
    state_to_following_tokens: dict[RuleState, set[str]] = {}

    rule_to_parent_states: dict[str, set[RuleState]] = {
        rule_name: set() for rule_name in rule_map
    }
    for rule_idx, rule in enumerate(rules):
        for step_idx, token in enumerate(rule.steps):
            if token in rule_map:
                rule_to_parent_states[token].add(RuleState(rule_idx, step_idx))

    def helper(state: RuleState):
        if state in state_to_following_tokens:
            return state_to_following_tokens[state]
        following_tokens = set()
        state_to_following_tokens[state] = following_tokens

        rule = rules[state.rule]
        if state.step + 1 < len(rule.steps):
            following_token = rule.steps[state.step + 1]
            if following_token in rule_map:
                following_tokens.update(
                    rule_to_first_tokens[rule.steps[state.step + 1]]
                )
            else:
                following_tokens.add(following_token)
        else:
            for parent_state in rule_to_parent_states[rule.name]:
                following_tokens.update(helper(parent_state))

        return following_tokens

    for rule_idx, rule in enumerate(rules):
        for step_idx in range(len(rule.steps)):
            helper(RuleState(rule_idx, step_idx))

    return state_to_following_tokens


@dataclass(frozen=True)
class Edge:
    token: str
    follow_token: Optional[str]
    follow_tokens: Optional[set[str]]
    reduce_rule: Optional[int]
    next_states: Optional[list[RuleState]]


def get_next_states(
    states: list[RuleState],
    rules: list[Rule],
    rule_map: dict[str, list[int]],
    state_to_following: dict[RuleState, set[str]],
    rule_to_first_tokens: dict[str, set[str]],
):
    next_groups: dict[str, list[RuleState]] = {}
    for state in states:
        assert state.step < len(rules[state.rule].steps)
        next_step = rules[state.rule].steps[state.step]
        next_groups.setdefault(next_step, []).append(state)

    next_actions: list[Edge] = []
    for token, next_states in next_groups.items():
        reduce_states: list[RuleState] = []
        shift_states: list[RuleState] = []
        is_base_token = token not in rule_map
        for state in next_states:
            if state.step + 1 < len(rules[state.rule].steps):
                shift_states.append(state.incr())
            else:
                reduce_states.append(state)

        if len(reduce_states) == 1 and len(shift_states) == 0:
            next_actions.append(
                Edge(
                    token=token,
                    follow_token=None,
                    follow_tokens=(
                        state_to_following[reduce_states[0]] if is_base_token else None
                    ),
                    reduce_rule=reduce_states[0].rule,
                    next_states=None,
                )
            )
        elif len(reduce_states) == 0:
            next_actions.append(
                Edge(
                    token=token,
                    follow_token=None,
                    follow_tokens=(
                        {
                            token
                            for state in shift_states
                            for token in rule_to_first_tokens[
                                rules[state.rule].steps[state.step]
                            ]
                        }
                        if is_base_token
                        else None
                    ),
                    reduce_rule=None,
                    next_states=shift_states,
                )
            )
        else:
            # only use lookahead when token is a rule
            assert not is_base_token

            # group states by following tokens
            follow_groups: dict[str, list[RuleState]] = {}
            for state in next_states:
                following_tokens = state_to_following[state]
                for follow_token in following_tokens:
                    follow_groups.setdefault(follow_token, []).append(state)

            for follow_token, states_group in follow_groups.items():
                reduce_states: list[RuleState] = []
                shift_states: list[RuleState] = []
                for state in states_group:
                    if state.step + 1 < len(rules[state.rule].steps):
                        shift_states.append(state.incr())
                    else:
                        reduce_states.append(state)
                if len(reduce_states) > 0:
                    assert len(reduce_states) == 1, "reduce/reduce conflict"
                    assert len(shift_states) == 0, "shift/reduce conflict"
                    next_actions.append(
                        Edge(
                            token=token,
                            follow_token=follow_token,
                            follow_tokens=None,
                            reduce_rule=reduce_states[0].rule,
                            next_states=None,
                        )
                    )
                else:
                    assert len(shift_states) > 0
                    next_actions.append(
                        Edge(
                            token=token,
                            follow_token=follow_token,
                            follow_tokens=None,
                            reduce_rule=None,
                            next_states=shift_states,
                        )
                    )

    return next_actions


@dataclass(frozen=True)
class LookupRow:
    state: int
    token: str
    follow_token: Optional[str]
    token_group: int
    next_state: Optional[int]
    reduce_rule: Optional[int]


def get_lookup_tbl_rows(rules: list[Rule], start_rule: int):
    rule_map: dict[str, list[int]] = {}
    for rule_idx, rule in enumerate(rules):
        rule_map.setdefault(rule.name, []).append(rule_idx)

    rule_to_first_tokens = get_first_tokens(rules, rule_map)
    state_to_following_tokens = get_state_to_following_tokens(
        rules, rule_map, rule_to_first_tokens
    )

    states_to_idx: dict[tuple[RuleState, ...], int] = {}
    lookup_rows: list[LookupRow] = []
    tokens_to_group: dict[tuple[str, ...], int] = {}

    def register_token_group(tokens: set[str]):
        tokens_key = tuple(sorted(tokens))
        token_group = tokens_to_group.get(tokens_key)
        if token_group is None:
            token_group = len(tokens_to_group)
            tokens_to_group[tokens_key] = token_group

        return token_group

    # register the starting token group first so it is always
    # group 0
    register_token_group(rule_to_first_tokens[rules[start_rule].name])

    def helper(states: tuple[RuleState, ...]):
        states = expand_states(states, rules, rule_map)
        states_idx = states_to_idx.get(states)
        if states_idx is not None:
            return states_idx
        states_idx = len(states_to_idx)
        states_to_idx[states] = states_idx

        for next_action in get_next_states(
            states, rules, rule_map, state_to_following_tokens, rule_to_first_tokens
        ):
            if next_action.follow_tokens is None:
                token_group = None
            else:
                token_group = register_token_group(next_action.follow_tokens)

            if next_action.reduce_rule is not None:
                lookup_rows.append(
                    LookupRow(
                        state=states_idx,
                        token=next_action.token,
                        follow_token=next_action.follow_token,
                        token_group=token_group,
                        next_state=None,
                        reduce_rule=next_action.reduce_rule,
                    )
                )
            elif next_action.next_states is not None:
                next_states_idx = helper(next_action.next_states)
                lookup_rows.append(
                    LookupRow(
                        state=states_idx,
                        token=next_action.token,
                        follow_token=next_action.follow_token,
                        token_group=token_group,
                        next_state=next_states_idx,
                        reduce_rule=None,
                    )
                )
            else:
                raise Exception("unreachable")

        return states_idx

    helper([RuleState(start_rule, 0)])
    lookup_rows.sort(key=lambda row: (row.state, row.token))

    token_groups = {group_idx: tokens for tokens, group_idx in tokens_to_group.items()}

    return lookup_rows, token_groups


@dataclass(frozen=True)
class ElemWrapper:
    rule_name: str
    value: Any


class ParserState:
    def __init__(self, lookup_tbl_rows: list[LookupRow], rules: list[Rule]):
        self.rules = rules
        self.state_stack = [0]
        self.val_stack: list[ElemWrapper] = []
        self.lookup_tbl: dict[tuple[int, str, Optional[str]], LookupRow] = {}
        self.token_group = 0
        for row in lookup_tbl_rows:
            key = (row.state, row.token, row.follow_token)
            assert key not in self.lookup_tbl
            self.lookup_tbl[key] = row

    def get_value(self):
        assert len(self.val_stack) == 1
        return self.val_stack[-1].value

    def step(self, token: str, token_value: str):
        assert (len(self.state_stack) == len(self.val_stack)) or (
            len(self.state_stack) == len(self.val_stack) + 1
        )

        # reduce until ready to accept next token
        while len(self.state_stack) == len(self.val_stack):
            state = self.state_stack[-1]
            prev_rule = self.val_stack[-1].rule_name
            lookup_row = self.lookup_tbl.get((state, prev_rule, None))
            if lookup_row is None:
                lookup_row = self.lookup_tbl[(state, prev_rule, token)]

            if lookup_row.next_state is not None:
                self.state_stack.append(lookup_row.next_state)
            else:
                reduce_rule = lookup_row.reduce_rule
                assert reduce_rule is not None
                rule = self.rules[reduce_rule]
                n_stack_elems = len(rule.steps)
                elems = [
                    elem.value for elem in get_list_tail(self.val_stack, n_stack_elems)
                ]
                pop_list(self.val_stack, n_stack_elems)
                pop_list(self.state_stack, n_stack_elems - 1)
                value = rule.handler(elems)
                self.val_stack.append(ElemWrapper(rule.name, value))

        assert len(self.state_stack) == len(self.val_stack) + 1

        state = self.state_stack[-1]
        lookup_row = self.lookup_tbl[(state, token, None)]
        self.token_group = lookup_row.token_group

        if lookup_row.next_state is not None:
            self.state_stack.append(lookup_row.next_state)
            self.val_stack.append(ElemWrapper(token, token_value))
        else:
            reduce_rule = lookup_row.reduce_rule
            assert reduce_rule is not None
            rule = self.rules[reduce_rule]
            n_stack_elems = len(rule.steps) - 1
            elems = [
                *[elem.value for elem in get_list_tail(self.val_stack, n_stack_elems)],
                token_value,
            ]
            pop_list(self.val_stack, n_stack_elems)
            pop_list(self.state_stack, n_stack_elems)
            value = rule.handler(elems)
            self.val_stack.append(ElemWrapper(rule.name, value))


def pop_list(lst: list, count: int):
    if count > 0:
        del lst[-count:]


def get_list_tail(lst: list, count: int):
    if count > 0:
        return lst[-count:]
    return []


class Parser:
    def __init__(
        self,
        tokens: list[tuple[str, str]],
        skip_pattern: Optional[str],
        start_rule: str,
        rules: list[Rule],
    ):
        self.rules = [Rule("MAIN", [start_rule, "END"], lambda elems: elems[0]), *rules]
        self.skip_pattern = None if skip_pattern is None else re.compile(skip_pattern)
        self.lookup_rows, self.token_groups = get_lookup_tbl_rows(self.rules, 0)

        rule_names = {rule.name for rule in rules}
        state_to_next_tokens: dict[int, set[str]] = {}
        for row in self.lookup_rows:
            if row.token not in rule_names and row.token != "END":
                state_to_next_tokens.setdefault(row.state, set()).add(row.token)

        self.token_map = {
            name: (idx, re.compile(pattern))
            for idx, (name, pattern) in enumerate(tokens)
        }

    def parse(self, text: str):
        parser_state = ParserState(self.lookup_rows, self.rules)

        text_idx = 0
        while True:
            token_name, value, text_idx = self.get_next_token(
                text, text_idx, parser_state.token_group
            )
            parser_state.step(token_name, value)
            if token_name == "END":
                return parser_state.get_value()

    def get_next_token(self, text: str, text_idx: int, token_group: int):
        if self.skip_pattern is not None:
            match = self.skip_pattern.match(text, text_idx)
            if match is not None:
                start_idx, end_idx = match.span()
                text_idx += end_idx - start_idx

        if text_idx >= len(text):
            return ("END", "", text_idx)

        best_match_token: Optional[str] = None
        best_match_len: Optional[int] = None
        best_match_text: Optional[str] = None
        best_idx: Optional[int] = None
        for token_name in self.token_groups[token_group]:
            if token_name == "END":
                continue
            token_idx, token_pattern = self.token_map[token_name]
            match = token_pattern.match(text, text_idx)
            if match is not None:
                start_idx, end_idx = match.span()
                match_len = end_idx - start_idx
                match_text = text[start_idx:end_idx]
                if (
                    best_match_token is None
                    or match_len > best_match_len
                    or (match_len == best_match_len and token_idx < best_idx)
                ):
                    best_match_token = token_name
                    best_match_len = match_len
                    best_match_text = match_text
                    best_idx = token_idx

        assert (
            best_match_text is not None
        ), f"Unable to parse token at idx={text_idx}\ntokens = {self.token_groups[token_group]}\ntext = {json.dumps(text[text_idx:text_idx+10])}"

        assert (
            best_match_len
        ) > 0, f"The regex pattern for token_name={best_match_token} must match at least 1 character."
        text_idx += best_match_len
        return (best_match_token, best_match_text, text_idx)