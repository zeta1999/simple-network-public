use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct OrSet<T: Eq + Hash + Clone> {
    added: HashMap<T, HashSet<String>>,
    removed: HashSet<String>,
}

impl<T: Eq + Hash + Clone> Default for OrSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> OrSet<T> {
    pub fn new() -> Self {
        Self {
            added: HashMap::new(),
            removed: HashSet::new(),
        }
    }

    pub fn add(&mut self, element: T, tag: String) {
        self.added.entry(element).or_default().insert(tag);
    }

    pub fn remove(&mut self, element: &T) {
        if let Some(tags) = self.added.get(element) {
            for tag in tags {
                self.removed.insert(tag.clone());
            }
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        if let Some(tags) = self.added.get(element) {
            tags.iter().any(|tag| !self.removed.contains(tag))
        } else {
            false
        }
    }

    pub fn merge(&mut self, other: &OrSet<T>) {
        for (element, tags) in &other.added {
            let my_tags = self.added.entry(element.clone()).or_default();
            for tag in tags {
                my_tags.insert(tag.clone());
            }
        }
        for tag in &other.removed {
            self.removed.insert(tag.clone());
        }
    }

    pub fn elements(&self) -> HashSet<T> {
        let mut res = HashSet::new();
        for (element, tags) in &self.added {
            if tags.iter().any(|tag| !self.removed.contains(tag)) {
                res.insert(element.clone());
            }
        }
        res
    }
}
