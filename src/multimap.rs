use linked_hash_map::LinkedHashMap;
use serde::{private::de::{Content, ContentRefDeserializer}, Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;

#[derive(Clone)]
enum Element<T> {
    Single(Vec<T>),
    Many(Vec<T>),
}

impl<T: Serialize> Serialize for Element<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Single(x) => x[0].serialize(serializer),
            Self::Many(xs) => xs.serialize(serializer),
        }
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Element<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let content = Content::deserialize(deserializer)?;
        if let Ok(x) = T::deserialize(ContentRefDeserializer::<D::Error>::new(&content)) {
            return Ok(Element::Single(vec![x]));
        } else {
            let xs = Vec::<T>::deserialize(ContentRefDeserializer::<D::Error>::new(&content))?;
            return Ok(Element::Many(xs));
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MultiMap<K: Eq + Hash, V>(LinkedHashMap<K, Element<V>>);

impl<K: Eq + Hash, V> MultiMap<K, V> {
    pub fn new() -> Self {
        Self {
            0: LinkedHashMap::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        let to_push;
        match self.0.get_mut(&k) {
            Some(e) => {
                match e {
                    Element::Single(x) => {
                        to_push = Element::Many(vec![x.remove(0), v]);
                    }
                    Element::Many(xs) => {
                        xs.push(v);
                        return;
                    }
                };
            }
            None => {
                to_push = Element::Single(vec![v]);
            }
        };
        self.0.insert(k, to_push);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a K, &'a V)> + 'a {
        self.0.iter().flat_map(|(k, v)| {
            let vs = match v {
                Element::Single(x) => x,
                Element::Many(xs) => xs,
            };
            vs.iter().map(move |s| (k, s))
        })
    }

    pub fn iter_vecs(&self) -> impl Iterator<Item = (&K, &Vec<V>)> + '_ {
        self.0.iter().map(|(k, v)| match v {
            Element::Single(x) => (k, x),
            Element::Many(xs) => (k, xs),
        })
    }

    pub fn get_vec(&self, k: &K) -> Option<&Vec<V>> {
        match self.0.get(k) {
            Some(v) => match v {
                Element::Single(x) => Some(x),
                Element::Many(xs) => Some(xs),
            },
            None => None,
        }
    }
}
