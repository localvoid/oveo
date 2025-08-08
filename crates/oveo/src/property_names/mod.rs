use std::{
    collections::hash_map,
    sync::{
        Arc, Mutex,
        atomic::{self, AtomicU32},
    },
};

use dashmap::DashMap;
use oxc_ast::{AstBuilder, ast::*};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{OptimizerError, property_names::base54::base54};

mod base54;

pub struct PropertyMap {
    regex: Option<regex::Regex>,
    index: DashMap<Box<str>, Arc<str>>,
    used: Mutex<FxHashSet<Arc<str>>>,
    next_id: AtomicU32,
}

impl PropertyMap {
    pub fn new(regex: Option<regex::Regex>) -> Self {
        let used = Mutex::default();
        add_reserved_keywords(&mut used.lock().unwrap());

        Self { regex, index: DashMap::default(), used, next_id: AtomicU32::new(0) }
    }

    pub fn import(&mut self, data: &[u8]) -> Result<(), OptimizerError> {
        {
            let mut used = self.used.lock().unwrap();
            self.next_id.store(0, atomic::Ordering::SeqCst);

            for (i, line) in data.split(|c| *c == b'\n').enumerate() {
                let line = line.trim_ascii();
                let Ok(line) = str::from_utf8(line) else {
                    return Err(OptimizerError::PropertyMapParseError(format!(
                        "invalid utf8 at line '{}'",
                        i + 1
                    )));
                };
                if !line.is_empty() {
                    let mut split = line.split('=');
                    let Some(key) = split.next() else {
                        return Err(OptimizerError::PropertyMapParseError(format!(
                            "invalid key at line '{}'",
                            i + 1
                        )));
                    };
                    let Some(value) = split.next() else {
                        return Err(OptimizerError::PropertyMapParseError(format!(
                            "invalid value at line '{}'",
                            i + 1
                        )));
                    };
                    let v: Arc<str> = value.into();
                    self.index.insert(key.into(), Arc::clone(&v));
                    used.insert(v);
                }
            }
        }
        Ok(())
    }

    pub fn export(&self) -> Vec<u8> {
        let mut props = Vec::new();
        for i in self.index.iter() {
            props.push((i.key().to_string(), i.value().to_string()))
        }
        props.sort_by(|a, b| a.0.cmp(&b.0));

        let mut b: Vec<u8> = Vec::new();
        for i in &props {
            b.extend(i.0.as_bytes());
            b.push(b'=');
            b.extend(i.1.as_bytes());
            b.push(b'\n');
        }
        b
    }

    pub fn matches(&self, s: &str) -> bool {
        if let Some(re) = &self.regex { re.is_match(s) } else { false }
    }
}

pub struct LocalPropertyMap<'a, 'ctx> {
    map: &'ctx PropertyMap,
    cache: FxHashMap<Atom<'a>, Option<Atom<'a>>>,
}

impl<'a, 'ctx> LocalPropertyMap<'a, 'ctx> {
    pub fn new(map: &'ctx PropertyMap) -> Self {
        Self { map, cache: FxHashMap::default() }
    }

    pub fn get(&mut self, key: Atom<'a>, ast: &AstBuilder<'a>) -> Option<Atom<'a>> {
        match self.cache.entry(key) {
            hash_map::Entry::Occupied(cache_entry) => *cache_entry.get(),
            hash_map::Entry::Vacant(cache_entry) => {
                let uid = match self.map.index.entry(key.as_str().into()) {
                    dashmap::Entry::Occupied(index_entry) => Some(ast.atom(index_entry.get())),
                    dashmap::Entry::Vacant(index_entry) => {
                        if !self.map.matches(key.as_str()) {
                            None
                        } else {
                            let mut used = self.map.used.lock().unwrap();
                            let uid = loop {
                                let i = self.map.next_id.fetch_add(1, atomic::Ordering::SeqCst);
                                let s = base54(i);
                                let uid: Arc<str> = Arc::from(s.as_str());
                                if used.insert(Arc::clone(&uid)) {
                                    index_entry.insert(uid);
                                    break ast.atom(&s);
                                }
                            };
                            Some(uid)
                        }
                    }
                };
                cache_entry.insert(uid);
                uid
            }
        }
    }
}

fn add_reserved_keywords(index: &mut FxHashSet<Arc<str>>) {
    index.insert("as".into());
    index.insert("do".into());
    index.insert("if".into());
    index.insert("in".into());
    index.insert("is".into());
    index.insert("of".into());
    index.insert("any".into());
    index.insert("for".into());
    index.insert("get".into());
    index.insert("let".into());
    index.insert("new".into());
    index.insert("out".into());
    index.insert("set".into());
    index.insert("try".into());
    index.insert("var".into());
    index.insert("case".into());
    index.insert("else".into());
    index.insert("enum".into());
    index.insert("from".into());
    index.insert("meta".into());
    index.insert("null".into());
    index.insert("this".into());
    index.insert("true".into());
    index.insert("type".into());
    index.insert("void".into());
    index.insert("with".into());
}
