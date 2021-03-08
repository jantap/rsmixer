
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;

enum Element<T> {
    Single(T),
    Many(Vec<T>),
}

pub struct MultiMap<K: Eq + Hash, V>(LinkedHashMap<K, Element<V>>);

impl<K: Eq + Hash, V> MultiMap<K, V> {
    pub fn insert(&mut self, k: K, v: V) {
        match self.0.get_mut(&k) {
            Some(e) => {
                match e {
                    Element::Single(x) => {
                        self.0.insert(k, Element::Many(vec![*x, v]));
                    }
                    Element::Many(xs) => {
                        xs.push(v);
                    }
                };
            },
            None => {
                self.0.insert(k, Element::Single(v));
            }
        };
    }
}
