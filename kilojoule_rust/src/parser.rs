use super::ast_node::AstNode;
use super::lookup_rows::{LookupRow, LOOKUP_ROWS};
use super::rule_type::RuleType;
use super::rules::{Rule, RULES};
use super::token::Token;
use super::token_def::TOKEN_DEFS;
use super::token_groups::TOKEN_GROUPS;
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Parser<'a> {
    skip_pattern: Regex,
    token_groups: HashMap<usize, &'a [Token]>,
    token_map: HashMap<Token, (usize, Regex)>,
    rules: &'a [Rule<'a>],
    lookup_tbl: HashMap<(u64, Token, Option<Token>), LookupRow>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        return Parser {
            skip_pattern: Regex::new(r"^[ \n\t\r]*").unwrap(),
            token_groups: TOKEN_GROUPS
                .iter()
                .enumerate()
                .map(|(group_idx, group)| (group_idx, group.tokens))
                .collect::<HashMap<_, _>>(),
            token_map: TOKEN_DEFS
                .iter()
                .enumerate()
                .map(|(token_idx, token_def)| {
                    let pattern_with_start = format!("^{}", token_def.pattern);

                    (
                        token_def.token,
                        (token_idx, Regex::new(pattern_with_start.as_str()).unwrap()),
                    )
                })
                .collect::<HashMap<_, _>>(),
            rules: RULES,
            lookup_tbl: LOOKUP_ROWS
                .iter()
                .map(|row| ((row.state, row.token, row.follow_token), row.clone()))
                .collect::<HashMap<_, _>>(),
        };
    }

    pub fn parse(&self, text: &str) -> Rc<AstNode> {
        let mut tokenizer = Tokenizer::new(
            text,
            &self.skip_pattern,
            &self.token_groups,
            &self.token_map,
        );
        let mut parser_state = ParserState::new(self.rules, &self.lookup_tbl);
        loop {
            let (token, token_value) = tokenizer.next(parser_state.get_token_group());
            parser_state.step(token, token_value);
            if token == Token::END {
                return parser_state.get_value();
            }
        }
    }
}

struct ParserStateNode {
    token: Token,
    node: Rc<AstNode>,
}

struct ParserState<'a> {
    token_group: u64,
    state_stack: Vec<u64>,
    val_stack: Vec<ParserStateNode>,
    lookup_tbl: &'a HashMap<(u64, Token, Option<Token>), LookupRow>,
    rules: &'a [Rule<'a>],
}

impl<'a> ParserState<'a> {
    pub fn new(
        rules: &'a [Rule<'a>],
        lookup_tbl: &'a HashMap<(u64, Token, Option<Token>), LookupRow>,
    ) -> Self {
        return ParserState {
            token_group: 0,
            state_stack: vec![0],
            val_stack: Vec::new(),
            lookup_tbl,
            rules,
        };
    }

    pub fn get_value(&self) -> Rc<AstNode> {
        assert!(self.val_stack.len() == 1);
        return self.val_stack.last().unwrap().node.clone();
    }

    pub fn step(&mut self, token: Token, token_value: &str) {
        assert!(
            self.state_stack.len() == self.val_stack.len()
                || self.state_stack.len() == self.val_stack.len() + 1
        );

        // reduce until ready to accept next token
        while self.state_stack.len() == self.val_stack.len() {
            let state = *self.state_stack.last().unwrap();
            let prev_token = self.val_stack.last().unwrap().token;
            let lookup_row = match self.lookup_tbl.get(&(state, prev_token, None)) {
                Some(lookup_row) => lookup_row,
                None => &self.lookup_tbl[&(state, prev_token, Some(token))],
            };

            if lookup_row.next_state.is_some() {
                self.state_stack.push(lookup_row.next_state.unwrap());
            } else {
                let reduce_rule = lookup_row.reduce_rule.unwrap();
                let rule = &self.rules[reduce_rule as usize];
                let n_stack_elems = rule.steps.len();
                let elems = self
                    .val_stack
                    .drain(self.val_stack.len().saturating_sub(n_stack_elems)..)
                    .map(|elem| elem.node)
                    .collect::<Vec<_>>();
                self.val_stack.push(ParserStateNode {
                    token: rule.token,
                    node: get_reduced_rule(rule.rule_type, elems),
                });

                pop_vec(&mut self.state_stack, n_stack_elems.saturating_sub(1));
            }
        }

        assert!(self.state_stack.len() == self.val_stack.len() + 1);

        let state = *self.state_stack.last().unwrap();
        let lookup_row = &self.lookup_tbl[&(state, token, None)];
        self.token_group = lookup_row.token_group.unwrap();

        if lookup_row.next_state.is_some() {
            self.state_stack.push(lookup_row.next_state.unwrap());
            self.val_stack.push(ParserStateNode {
                token,
                node: Rc::new(get_reduced_token(token, token_value)),
            });
        } else {
            let reduce_rule = lookup_row.reduce_rule.unwrap();
            let rule = &self.rules[reduce_rule as usize];
            let n_stack_elems = rule.steps.len().saturating_sub(1);
            let mut elems = self
                .val_stack
                .drain(self.val_stack.len().saturating_sub(n_stack_elems)..)
                .map(|elem| elem.node)
                .collect::<Vec<_>>();
            elems.push(Rc::new(get_reduced_token(token, token_value)));
            self.val_stack.push(ParserStateNode {
                token: rule.token,
                node: get_reduced_rule(rule.rule_type, elems),
            });

            pop_vec(&mut self.state_stack, n_stack_elems);
        }
    }

    pub fn get_token_group(&self) -> u64 {
        return self.token_group;
    }
}

fn get_reduced_token(token: Token, text: &str) -> AstNode {
    return match token {
        Token::INTEGER => AstNode::Int(text.parse::<u64>().unwrap()),
        Token::PLUS => AstNode::Plus,
        Token::ASTERISK => AstNode::Asterisk,
        Token::END => AstNode::End,
        _ => {
            panic!("Unimplemented token reduce {:?}", token);
        }
    };
}

fn get_reduced_rule(rule: RuleType, elems: Vec<Rc<AstNode>>) -> Rc<AstNode> {
    return match rule {
        RuleType::Main__Expr_END => elems[0].clone(),
        RuleType::Expr__AddExpr => elems[0].clone(),
        RuleType::AddExpr__AddExpr_PLUS_MulExpr => {
            Rc::new(AstNode::Add(elems[0].clone(), elems[2].clone()))
        }
        RuleType::AddExpr__MulExpr => elems[0].clone(),
        RuleType::MulExpr__MulExpr_ASTERISK_INTEGER => {
            Rc::new(AstNode::Mul(elems[0].clone(), elems[2].clone()))
        }
        RuleType::MulExpr__INTEGER => elems[0].clone(),
    };
}

fn pop_vec<T>(vec: &mut Vec<T>, count: usize) {
    vec.truncate(vec.len().saturating_sub(count));
}

struct Tokenizer<'a> {
    text: &'a str,
    text_idx: usize,
    skip_pattern: &'a Regex,
    token_groups: &'a HashMap<usize, &'a [Token]>,
    token_map: &'a HashMap<Token, (usize, Regex)>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(
        text: &'a str,
        skip_pattern: &'a Regex,
        token_groups: &'a HashMap<usize, &'a [Token]>,
        token_map: &'a HashMap<Token, (usize, Regex)>,
    ) -> Self {
        return Tokenizer {
            text,
            text_idx: 0,
            skip_pattern,
            token_groups,
            token_map,
        };
    }

    pub fn next(&mut self, token_group: u64) -> (Token, &'a str) {
        match self.skip_pattern.captures(&self.text[self.text_idx..]) {
            Some(cap) => {
                let match_info = cap.get(0).unwrap();
                self.text_idx += match_info.len();
            }
            None => {}
        }

        if self.text_idx >= self.text.len() {
            return (Token::END, "");
        }

        #[derive(Clone, Copy)]
        struct BestMatch {
            token: Token,
            start_idx: usize,
            len: usize,
            token_idx: usize,
        }

        let mut best: Option<BestMatch> = None;
        for token_name in self.token_groups[&(token_group as usize)] {
            if *token_name == Token::END {
                continue;
            }
            let (token_idx, token_pattern) = &self.token_map[token_name];
            match token_pattern.captures(&self.text[self.text_idx..]) {
                Some(cap) => {
                    let match_info = cap.get(0).unwrap();
                    match best {
                        None => {
                            best = Some(BestMatch {
                                token: *token_name,
                                start_idx: match_info.start(),
                                len: match_info.len(),
                                token_idx: *token_idx,
                            });
                        }
                        Some(best_val) => {
                            if match_info.len() > best_val.len
                                || (match_info.len() == best_val.len
                                    && *token_idx < best_val.token_idx)
                            {
                                best = Some(BestMatch {
                                    token: *token_name,
                                    start_idx: match_info.start(),
                                    len: match_info.len(),
                                    token_idx: *token_idx,
                                });
                            }
                        }
                    }
                }
                None => {}
            }
        }

        let best = best.unwrap();
        assert!(best.len > 0);
        let best_text =
            &self.text[self.text_idx + best.start_idx..self.text_idx + best.start_idx + best.len];
        self.text_idx += best.len;
        return (best.token, best_text);
    }
}
