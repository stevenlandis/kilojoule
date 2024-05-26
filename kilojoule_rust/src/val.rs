use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::{hash::DefaultHasher, rc::Rc};

#[derive(Debug, Clone)]
pub struct Val {
    pub val: Rc<InnerVal>,
}

#[derive(Debug)]
pub struct InnerVal {
    pub hash: u64,
    pub val: ValType,
}

#[derive(Debug, PartialEq)]
pub enum ValType {
    Error(String),
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Val>),
    Map(ValHashMap),
    Bytes(Vec<u8>),
}

impl Val {
    pub fn new_null() -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::Null.hash(&mut hasher);
        let hash = hasher.finish();
        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::Null,
            }),
        };
    }

    pub fn new_bool(val: bool) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::Bool.hash(&mut hasher);
        val.hash(&mut hasher);
        return Val {
            val: Rc::new(InnerVal {
                hash: hasher.finish(),
                val: ValType::Bool(val),
            }),
        };
    }

    pub fn new_number(val: f64) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::Number.hash(&mut hasher);
        (val as u64).hash(&mut hasher);
        return Val {
            val: Rc::new(InnerVal {
                hash: hasher.finish(),
                val: ValType::Number(val),
            }),
        };
    }

    pub fn new_string(text: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::String.hash(&mut hasher);
        text.hash(&mut hasher);
        let hash = hasher.finish();
        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::String(text.to_string()),
            }),
        };
    }

    pub fn new_err(text: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::String.hash(&mut hasher);
        text.hash(&mut hasher);
        let hash = hasher.finish();
        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::Error(text.to_string()),
            }),
        };
    }

    pub fn new_list(values: &[Val]) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::List.hash(&mut hasher);
        for val in values {
            val.val.hash.hash(&mut hasher);
        }
        let hash = hasher.finish();

        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::List(values.iter().cloned().collect()),
            }),
        };
    }

    pub fn new_map_from_entries_iter(pairs: Vec<(Val, Val)>) -> Self {
        let map = ValHashMap::from_pairs(&pairs);

        let mut hasher = DefaultHasher::new();
        HashTypes::Map.hash(&mut hasher);
        map.hash(&mut hasher);
        let hash = hasher.finish();

        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::Map(map),
            }),
        };
    }

    pub fn new_map(map: ValHashMap) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::Map.hash(&mut hasher);
        map.hash(&mut hasher);

        return Val {
            val: Rc::new(InnerVal {
                hash: hasher.finish(),
                val: ValType::Map(map),
            }),
        };
    }

    pub fn new_bytes(bytes: Vec<u8>) -> Self {
        let mut hasher = DefaultHasher::new();
        HashTypes::Bytes.hash(&mut hasher);
        bytes.hash(&mut hasher);

        Val {
            val: Rc::new(InnerVal {
                hash: hasher.finish(),
                val: ValType::Bytes(bytes),
            }),
        }
    }

    pub fn from_json_str(json_str: &str) -> Self {
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();

        fn helper(node: &serde_json::Value) -> Val {
            match node {
                serde_json::Value::Null => Val::new_null(),
                serde_json::Value::Bool(val) => Val::new_bool(*val),
                serde_json::Value::Number(val) => Val::new_number(val.as_f64().unwrap()),
                serde_json::Value::String(val) => Val::new_string(val.as_str()),
                serde_json::Value::Array(val) => Val::new_list(
                    val.iter()
                        .map(|elem| helper(elem))
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
                serde_json::Value::Object(val) => {
                    return Val::new_map_from_entries_iter(
                        val.iter()
                            .map(|(key, val)| (Val::new_string(key.as_str()), helper(val)))
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        return helper(&value);
    }

    pub fn write_json_str<W>(&self, writer: &mut W, use_indent: bool)
    where
        W: std::io::Write,
    {
        struct Writer<'a, W>
        where
            W: std::io::Write,
        {
            writer: &'a mut W,
        }

        impl<W> Writer<'_, W>
        where
            W: std::io::Write,
        {
            pub fn outer_write(&mut self, val: &Val, use_indent: bool) -> std::io::Result<usize> {
                if let ValType::Bytes(bytes) = &val.val.val {
                    self.writer.write(bytes.as_slice())?;
                    return Ok(0);
                }
                self.write(val, 0, use_indent)?;
                if use_indent {
                    self.str("\n")?;
                }
                Ok(0)
            }

            fn write(
                &mut self,
                val: &Val,
                indent: u64,
                use_indent: bool,
            ) -> std::io::Result<usize> {
                match &val.val.val {
                    ValType::Null => {
                        self.str("null")?;
                    }
                    ValType::Bool(val) => {
                        if *val {
                            self.str("true")?;
                        } else {
                            self.str("false")?;
                        }
                    }
                    ValType::Number(val) => {
                        self.str(val.to_string().as_str())?;
                    }
                    ValType::String(val) => {
                        self.str(
                            serde_json::to_string(&serde_json::Value::String(val.clone()))?
                                .as_str(),
                        )?;
                    }
                    ValType::List(val) => {
                        self.str("[")?;
                        for (idx, elem) in val.iter().enumerate() {
                            if idx > 0 {
                                if use_indent {
                                    self.str(", ")?;
                                } else {
                                    self.str(",")?;
                                }
                            }
                            if use_indent {
                                self.str("\n")?;
                                self.indent(indent + 1)?;
                            }
                            self.write(elem, indent + 1, use_indent)?;
                        }
                        if val.len() > 0 && use_indent {
                            self.str("\n")?;
                            self.indent(indent)?;
                        }
                        self.str("]")?;
                    }
                    ValType::Map(val) => {
                        self.str("{")?;
                        for (idx, (key, val)) in val
                            .pairs
                            .iter()
                            .filter_map(|pair| match pair {
                                None => None,
                                Some(pair) => Some(pair),
                            })
                            .enumerate()
                        {
                            if idx > 0 {
                                if use_indent {
                                    self.str(", ")?;
                                } else {
                                    self.str(",")?;
                                }
                            }
                            if use_indent {
                                self.str("\n")?;
                                self.indent(indent + 1)?;
                            }
                            self.write(key, indent + 1, use_indent)?;
                            if use_indent {
                                self.str(": ")?;
                            } else {
                                self.str(":")?;
                            }
                            self.write(val, indent + 1, use_indent)?;
                        }
                        if val.len() > 0 && use_indent {
                            self.str("\n")?;
                            self.indent(indent)?;
                        }
                        self.str("}")?;
                    }
                    ValType::Error(message) => {
                        self.str("{\"ERROR\":")?;
                        self.write(&Val::new_string(message.as_str()), 0, false)?;
                        self.str("}")?;
                    }
                    ValType::Bytes(bytes) => {
                        self.str("\"")?;
                        self.writer.write(STANDARD.encode(bytes).as_bytes())?;
                        self.str("\"")?;
                    }
                }
                Ok(0)
            }

            fn str(&mut self, text: &str) -> std::io::Result<usize> {
                self.writer.write(text.as_bytes())?;
                Ok(0)
            }

            fn indent(&mut self, indent: u64) -> std::io::Result<usize> {
                for _ in 0..indent {
                    self.str("  ")?;
                }
                Ok(0)
            }
        }

        let mut writer = Writer { writer };
        let _ = writer.outer_write(self, use_indent);
    }
}

#[derive(Hash)]
enum HashTypes {
    Null,
    Bool,
    Number,
    String,
    List,
    Map,
    Bytes,
}

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        if self.val.hash != other.val.hash {
            return false;
        }
        return self.val.val == other.val.val;
    }
}

impl Eq for Val {}

impl Hash for Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash.hash(state);
    }
}

impl Ord for Val {
    /*
    Order
    - Error
    - Null
    - Bool
    - Number
    - String
    - List
    - Map
    */
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match &self.val.val {
            ValType::Error(err0) => match &other.val.val {
                ValType::Error(err1) => err0.cmp(&err1),
                ValType::Null => std::cmp::Ordering::Less,
                ValType::Bool(_) => std::cmp::Ordering::Less,
                ValType::Number(_) => std::cmp::Ordering::Less,
                ValType::String(_) => std::cmp::Ordering::Less,
                ValType::List(_) => std::cmp::Ordering::Less,
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::Null => match other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Equal,
                ValType::Bool(_) => std::cmp::Ordering::Less,
                ValType::Number(_) => std::cmp::Ordering::Less,
                ValType::String(_) => std::cmp::Ordering::Less,
                ValType::List(_) => std::cmp::Ordering::Less,
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::Bool(val0) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(val1) => val0.cmp(val1),
                ValType::Number(_) => std::cmp::Ordering::Less,
                ValType::String(_) => std::cmp::Ordering::Less,
                ValType::List(_) => std::cmp::Ordering::Less,
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::Number(val0) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(val1) => val0.total_cmp(val1),
                ValType::String(_) => std::cmp::Ordering::Less,
                ValType::List(_) => std::cmp::Ordering::Less,
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::String(val0) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(_) => std::cmp::Ordering::Greater,
                ValType::String(val1) => val0.cmp(val1),
                ValType::List(_) => std::cmp::Ordering::Less,
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::List(val0) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(_) => std::cmp::Ordering::Greater,
                ValType::String(_) => std::cmp::Ordering::Greater,
                ValType::List(val1) => list_cmp(val0, val1),
                ValType::Map(_) => std::cmp::Ordering::Less,
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::Map(_) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(_) => std::cmp::Ordering::Greater,
                ValType::String(_) => std::cmp::Ordering::Greater,
                ValType::List(_) => std::cmp::Ordering::Greater,
                ValType::Map(_) => panic!("map comparison is unimplemented"),
                ValType::Bytes(_) => std::cmp::Ordering::Less,
            },
            ValType::Bytes(_) => match &other.val.val {
                ValType::Error(_) => std::cmp::Ordering::Greater,
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(_) => std::cmp::Ordering::Greater,
                ValType::String(_) => std::cmp::Ordering::Greater,
                ValType::List(_) => std::cmp::Ordering::Greater,
                ValType::Map(_) => std::cmp::Ordering::Greater,
                ValType::Bytes(_) => panic!("bytes comparison is unimplemented"),
            },
        }
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn list_cmp(left: &Vec<Val>, right: &Vec<Val>) -> std::cmp::Ordering {
    for idx in 0..std::cmp::min(left.len(), right.len()) {
        let result = left[idx].cmp(&right[idx]);
        if result != std::cmp::Ordering::Equal {
            return result;
        }
    }

    return left.len().cmp(&right.len());
}

#[derive(Debug)]
pub struct ValHashMap {
    pairs: Vec<Option<(Val, Val)>>,
    key_to_idx: HashMap<Val, usize>,
}

impl ValHashMap {
    pub fn new() -> Self {
        return ValHashMap {
            pairs: Vec::new(),
            key_to_idx: HashMap::new(),
        };
    }

    pub fn from_pairs(pairs: &Vec<(Val, Val)>) -> Self {
        let mut map = Self::new();

        for (key, val) in pairs {
            map.insert(key, val);
        }

        return map;
    }

    pub fn insert(&mut self, key: &Val, value: &Val) {
        let idx = *self
            .key_to_idx
            .entry(key.clone())
            .or_insert(self.pairs.len());

        if idx >= self.pairs.len() {
            self.pairs.push(Some((key.clone(), value.clone())));
        } else {
            self.pairs[idx].as_mut().unwrap().1 = value.clone();
        }
    }

    pub fn get(&self, key: &Val) -> Option<&Val> {
        return match self.key_to_idx.get(key) {
            None => None,
            Some(idx) => Some(&self.pairs[*idx].as_ref().unwrap().1),
        };
    }

    pub fn len(&self) -> usize {
        return self.key_to_idx.len();
    }
}

impl PartialEq for ValHashMap {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, idx) in self.key_to_idx.iter() {
            match other.get(key) {
                None => {
                    return false;
                }
                Some(other_val) => {
                    let self_val = &self.pairs[*idx].as_ref().unwrap().1;
                    if self_val != other_val {
                        return false;
                    }
                }
            }
        }

        return true;
    }
}

impl Hash for ValHashMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut sorted_pairs = self
            .pairs
            .iter()
            .filter_map(|pair| match pair {
                None => None,
                Some(pair) => Some(pair),
            })
            .collect::<Vec<_>>();
        sorted_pairs.sort_by_key(|(key, _)| key);

        for (key, val) in sorted_pairs {
            key.hash(state);
            val.hash(state);
        }
    }
}
