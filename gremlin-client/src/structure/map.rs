use crate::structure::{Edge, GValue, Vertex};
use crate::Token;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, PartialEq, Clone)]
pub struct Map(HashMap<GKey, GValue>);

impl From<HashMap<GKey, GValue>> for Map {
    fn from(val: HashMap<GKey, GValue>) -> Self {
        Map(val)
    }
}

impl From<HashMap<String, GValue>> for Map {
    fn from(val: HashMap<String, GValue>) -> Self {
        let map = val.into_iter().map(|(k, v)| (GKey::String(k), v)).collect();
        Map(map)
    }
}

impl From<BTreeMap<String, GValue>> for Map {
    fn from(val: BTreeMap<String, GValue>) -> Self {
        let map = val.into_iter().map(|(k, v)| (GKey::String(k), v)).collect();
        Map(map)
    }
}

impl Map {
    pub(crate) fn remove<T>(&mut self, key: T) -> Option<GValue>
    where
        T: Into<GKey>,
    {
        self.0.remove(&key.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&GKey, &GValue)> {
        self.0.iter()
    }

    pub fn get<T>(&self, key: T) -> Option<&GValue>
    where
        T: Into<GKey>,
    {
        self.0.get(&key.into())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Into<GKey>> std::ops::Index<T> for Map {
    type Output = GValue;

    fn index(&self, key: T) -> &GValue {
        self.0.get(&key.into()).expect("no entry found for key")
    }
}
impl std::iter::FromIterator<(String, GValue)> for Map {
    fn from_iter<I: IntoIterator<Item = (String, GValue)>>(iter: I) -> Self {
        Map(iter
            .into_iter()
            .map(|(k, v)| (GKey::String(k), v))
            .collect())
    }
}
#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum GKey {
    String(String),
    Token(Token),
    Vertex(Vertex),
    Edge(Edge),
}

impl From<&str> for GKey {
    fn from(val: &str) -> Self {
        GKey::String(String::from(val))
    }
}

impl From<&Vertex> for GKey {
    fn from(val: &Vertex) -> Self {
        GKey::Vertex(val.clone())
    }
}

impl From<&Edge> for GKey {
    fn from(val: &Edge) -> Self {
        GKey::Edge(val.clone())
    }
}