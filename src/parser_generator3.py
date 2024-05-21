from dataclasses import dataclass
from typing import Optional
from src.parser_generator2 import Rule, get_first_tokens


class State:
    def __init__(self, rule_idx: int, step_idx: int, parents: list["State"]):
        self.rule_idx = rule_idx
        self.step_idx = step_idx
        self.parents = parents
        self.following: Optional[set[str]] = None

    def get_following(self, rules: list[Rule], token_to_firsts: dict[str, set[str]]):
        if self.following is None:
            self.following = set()
            for parent in self.parents:
                rule = rules[parent.rule_idx]
                if parent.step_idx + 1 < len(rule.steps):
                    self.following.update(
                        token_to_firsts[rule.steps[parent.step_idx + 1]]
                    )
                else:
                    self.following.update(parent.get_following(rules, token_to_firsts))

        return self.following

    def step(self):
        return State(self.rule_idx, self.step_idx + 1, self.parents)


class ParserState:
    def __init__(self, rules: list[Rule], start_rule: int):
        self.rules = rules
        self.start_rule = start_rule
        self.rule_map = get_rule_map(rules)
        self.token_to_firsts = get_first_tokens(rules, self.rule_map)

        self.active_states: list[State] = [State(start_rule, 0, [])]
        self.val_stack = []

    def step(self, token: str, token_val: str):
        while True:
            next_active_states, action = self.step_or_reduce(token)
            self.active_states = next_active_states
            if action == "shift":
                self.val_stack.append((token, token_val))
                break
            else:
                reduce_rule_idx = action[1]
                reduce_rule = self.rules[reduce_rule_idx]
                self.val_stack[-len(reduce_rule.steps) :] = [
                    (reduce_rule.name, *self.val_stack[-len(reduce_rule.steps) :])
                ]

    def step_or_reduce(self, token: str):
        reduce_states: list[State] = []
        step_states: list[State] = []

        expanded_rules: dict[str, list[State]] = {}

        def traverse_state(state: State):
            rule = self.rules[state.rule_idx]
            if state.step_idx >= len(rule.steps):
                if token in state.get_following(self.rules, self.token_to_firsts):
                    reduce_states.append(state)
            else:
                step = rule.steps[state.step_idx]
                if step == token:
                    step_states.append(state)
                elif step in self.rule_map and token in self.token_to_firsts[step]:
                    # following: set[str] = set()
                    # if state.step_idx + 1 < len(rule.steps):
                    #     following.update(
                    #         self.token_to_firsts[rule.steps[state.step_idx + 1]]
                    #     )
                    # else:
                    #     following.update(state.following)

                    if step not in expanded_rules:
                        new_states = [
                            State(rule_idx, 0, [state])
                            for rule_idx in self.rule_map[step]
                        ]
                        expanded_rules[step] = new_states
                        for state in new_states:
                            traverse_state(state)
                    else:
                        for sub_rule in expanded_rules[step]:
                            sub_rule.parents.append(state)

        for state in self.active_states:
            traverse_state(state)

        next_active_states: list[State] = []
        if len(reduce_states) > 0:
            assert len(step_states) == 0, "shift/reduce conflict"
            assert len(reduce_states) == 1, "reduce/reduce conflict"
            [state] = reduce_states
            for parent in state.parents:
                next_active_states.append(parent.step())
            return next_active_states, ("reduce", state.rule_idx)
        elif len(step_states) > 0:
            for state in step_states:
                next_active_states.append(state.step())
            return next_active_states, "shift"
        else:
            assert False, "unreachable"


def get_rule_map(rules: list[Rule]):
    rule_map: dict[str, list[int]] = {}
    for rule_idx, rule in enumerate(rules):
        rule_map.setdefault(rule.name, []).append(rule_idx)
    return rule_map
