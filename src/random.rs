use std::collections::HashSet;

use rltk::RandomNumberGenerator;

#[derive(Eq)]
pub struct Entry {
    name: String,
    weight: i32,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::hash::Hash for Entry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub struct Table {
    entries: HashSet<Entry>,
    total_weight: i32,
}

impl Table {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
            total_weight: 0,
        }
    }

    pub fn insert<S: Into<String>>(mut self, name: S, weight: i32) -> Self {
        let entry = Entry {
            name: name.into(),
            weight,
        };
        if self.entries.insert(entry) {
            self.total_weight += weight;
        }
        self
    }

    pub fn roll<'s, 'r>(&'s self, rng: &'r mut RandomNumberGenerator) -> Option<&'s str> {
        if self.total_weight == 0 {
            return None;
        }
        let mut entries = self.entries.iter();
        // Safe: we have just asserted that entries is not empty
        let mut entry = entries.next().unwrap();
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        while roll > 0 {
            if roll < entry.weight {
                return Some(&entry.name);
            }

            roll -= entry.weight;
            entry = entries.next()?
        }
        None
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
