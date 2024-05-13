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
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Val>),
    Map(ValHashMap),
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
        let mut sorted_pairs = pairs.iter().collect::<Vec<_>>();
        sorted_pairs.sort_by_key(|(key, _)| key);

        let mut hasher = DefaultHasher::new();
        HashTypes::Map.hash(&mut hasher);
        for (key, val) in sorted_pairs {
            key.hash(&mut hasher);
            val.hash(&mut hasher);
        }
        let hash = hasher.finish();

        return Val {
            val: Rc::new(InnerVal {
                hash,
                val: ValType::Map(ValHashMap::from_pairs(&pairs)),
            }),
        };
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
}

#[derive(Hash)]
enum HashTypes {
    Null,
    Bool,
    Number,
    String,
    List,
    Map,
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
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match &self.val.val {
            ValType::Null => match other.val.val {
                ValType::Null => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Less,
            },
            ValType::String(val) => match &other.val.val {
                ValType::Null => std::cmp::Ordering::Greater,
                ValType::Bool(_) => std::cmp::Ordering::Greater,
                ValType::Number(_) => std::cmp::Ordering::Greater,
                ValType::String(other_val) => val.cmp(&other_val),
                _ => std::cmp::Ordering::Less,
            },
            _ => {
                panic!("Unimplemented sort");
            }
        }
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct ValHashMap {
    pairs: Vec<(Val, Val)>,
    key_to_idx: HashMap<Val, usize>,
}

impl ValHashMap {
    fn new() -> Self {
        return ValHashMap {
            pairs: Vec::new(),
            key_to_idx: HashMap::new(),
        };
    }

    fn from_pairs(pairs: &Vec<(Val, Val)>) -> Self {
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
            self.pairs.push((key.clone(), value.clone()));
        } else {
            self.pairs[idx].1 = value.clone();
        }
    }

    pub fn get(&self, key: &Val) -> Option<&Val> {
        return match self.key_to_idx.get(key) {
            None => None,
            Some(idx) => Some(&self.pairs[*idx].1),
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
                    let self_val = &self.pairs[*idx].1;
                    if self_val != other_val {
                        return false;
                    }
                }
            }
        }

        return true;
    }
}