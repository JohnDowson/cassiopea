use crate::map::builders::{bsp::BspBuilder, simple::SimpleMapBuilder, MapBuilder};
use rltk::RandomNumberGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Eq, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        if weight > 0 {
            let entry = Entry {
                name: name.into(),
                weight,
            };
            if self.entries.insert(entry) {
                self.total_weight += weight;
            }
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

pub fn random_map_builder(dim_x: i32, dim_y: i32, layer: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::seeded(69);
    match rng.roll_dice(1, 2) {
        1 => Box::new(SimpleMapBuilder::new(dim_x, dim_y, layer)),
        _ => Box::new(BspBuilder::new(dim_x, dim_y, layer)),
    }
}
