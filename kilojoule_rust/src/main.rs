// use serde_json;
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    let parser = Parser::new();
    let ast = parser.parse("2*3 + 30*4 + 1 * 2 + 3");
    println!("Ast: {:?}", ast);
    let result = eval_ast_node(&ast);
    println!("Result = {}", result);
}

fn eval_ast_node(node: &AstNode) -> i64 {
    match node {
        AstNode::Int(val) => *val as i64,
        AstNode::Add(left, right) => eval_ast_node(&left) + eval_ast_node(&right),
        AstNode::Mul(left, right) => eval_ast_node(&left) * eval_ast_node(&right),
        _ => {
            panic!("Unimplemented eval");
        }
    }
}

struct Parser<'a> {
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

struct AstAddNode {
    left: Rc<AstNode>,
    right: Rc<AstNode>,
}

struct AstMulNode {
    left: Rc<AstNode>,
    right: Rc<AstNode>,
}

#[derive(Debug)]
enum AstNode {
    Int(u64),
    Plus,
    Asterisk,
    Add(Rc<AstNode>, Rc<AstNode>),
    Mul(Rc<AstNode>, Rc<AstNode>),
    End,
}

struct Rule<'a> {
    rule_type: RuleType,
    token: Token,
    steps: &'a [Token],
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
        _ => {
            panic!("Unimplemented rule reduce for rule={:?}", rule);
        }
    };
}

pub fn pop_vec<T>(vec: &mut Vec<T>, count: usize) {
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Token {
    INTEGER,
    PLUS,
    ASTERISK,
    END,
    Main,
    Expr,
    AddExpr,
    MulExpr,
}

#[derive(Clone)]
struct LookupRow {
    state: u64,
    token: Token,
    follow_token: Option<Token>,
    token_group: Option<u64>,
    next_state: Option<u64>,
    reduce_rule: Option<u64>,
}

struct TokenGroup<'a> {
    tokens: &'a [Token],
}

struct TokenDef<'a> {
    token: Token,
    pattern: &'a str,
}

#[derive(Debug, Clone, Copy)]
enum RuleType {
    Main__Expr_END,
    Expr__AddExpr,
    AddExpr__MulExpr,
    AddExpr__AddExpr_PLUS_MulExpr,
    MulExpr__INTEGER,
    MulExpr__MulExpr_ASTERISK_INTEGER,
}

static TOKEN_DEFS: &[TokenDef] = &[
    TokenDef {
        token: Token::INTEGER,
        pattern: r"\d+",
    },
    TokenDef {
        token: Token::PLUS,
        pattern: r"\+",
    },
    TokenDef {
        token: Token::ASTERISK,
        pattern: r"\*",
    },
];

static TOKEN_GROUPS: &[TokenGroup] = &[
    TokenGroup {
        tokens: &[Token::INTEGER],
    },
    TokenGroup { tokens: &[] },
    TokenGroup {
        tokens: &[Token::ASTERISK, Token::END, Token::PLUS],
    },
];

static RULES: &[Rule] = &[
    Rule {
        rule_type: RuleType::Main__Expr_END,
        token: Token::Main,
        steps: &[Token::Expr, Token::END],
    },
    Rule {
        rule_type: RuleType::Expr__AddExpr,
        token: Token::Expr,
        steps: &[Token::AddExpr],
    },
    Rule {
        rule_type: RuleType::AddExpr__MulExpr,
        token: Token::AddExpr,
        steps: &[Token::MulExpr],
    },
    Rule {
        rule_type: RuleType::AddExpr__AddExpr_PLUS_MulExpr,
        token: Token::AddExpr,
        steps: &[Token::AddExpr, Token::PLUS, Token::MulExpr],
    },
    Rule {
        rule_type: RuleType::MulExpr__INTEGER,
        token: Token::MulExpr,
        steps: &[Token::INTEGER],
    },
    Rule {
        rule_type: RuleType::MulExpr__MulExpr_ASTERISK_INTEGER,
        token: Token::MulExpr,
        steps: &[Token::MulExpr, Token::ASTERISK, Token::INTEGER],
    },
];

static LOOKUP_ROWS: &[LookupRow] = &[
    LookupRow {
        state: 0,
        token: Token::AddExpr,
        follow_token: Some(Token::END),
        token_group: None,
        next_state: None,
        reduce_rule: Some(1),
    },
    LookupRow {
        state: 0,
        token: Token::AddExpr,
        follow_token: Some(Token::PLUS),
        token_group: None,
        next_state: Some(2),
        reduce_rule: None,
    },
    LookupRow {
        state: 0,
        token: Token::Expr,
        follow_token: None,
        token_group: None,
        next_state: Some(1),
        reduce_rule: None,
    },
    LookupRow {
        state: 0,
        token: Token::INTEGER,
        follow_token: None,
        token_group: Some(2),
        next_state: None,
        reduce_rule: Some(4),
    },
    LookupRow {
        state: 0,
        token: Token::MulExpr,
        follow_token: Some(Token::END),
        token_group: None,
        next_state: None,
        reduce_rule: Some(2),
    },
    LookupRow {
        state: 0,
        token: Token::MulExpr,
        follow_token: Some(Token::PLUS),
        token_group: None,
        next_state: None,
        reduce_rule: Some(2),
    },
    LookupRow {
        state: 0,
        token: Token::MulExpr,
        follow_token: Some(Token::ASTERISK),
        token_group: None,
        next_state: Some(4),
        reduce_rule: None,
    },
    LookupRow {
        state: 1,
        token: Token::END,
        follow_token: None,
        token_group: Some(1),
        next_state: None,
        reduce_rule: Some(0),
    },
    LookupRow {
        state: 2,
        token: Token::PLUS,
        follow_token: None,
        token_group: Some(0),
        next_state: Some(3),
        reduce_rule: None,
    },
    LookupRow {
        state: 3,
        token: Token::INTEGER,
        follow_token: None,
        token_group: Some(2),
        next_state: None,
        reduce_rule: Some(4),
    },
    LookupRow {
        state: 3,
        token: Token::MulExpr,
        follow_token: Some(Token::END),
        token_group: None,
        next_state: None,
        reduce_rule: Some(3),
    },
    LookupRow {
        state: 3,
        token: Token::MulExpr,
        follow_token: Some(Token::PLUS),
        token_group: None,
        next_state: None,
        reduce_rule: Some(3),
    },
    LookupRow {
        state: 3,
        token: Token::MulExpr,
        follow_token: Some(Token::ASTERISK),
        token_group: None,
        next_state: Some(4),
        reduce_rule: None,
    },
    LookupRow {
        state: 4,
        token: Token::ASTERISK,
        follow_token: None,
        token_group: Some(0),
        next_state: Some(5),
        reduce_rule: None,
    },
    LookupRow {
        state: 5,
        token: Token::INTEGER,
        follow_token: None,
        token_group: Some(2),
        next_state: None,
        reduce_rule: Some(5),
    },
];
