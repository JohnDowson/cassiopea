use specs::prelude::*;

use crate::{
    components::{Blocker, Position},
    map::Map,
};

pub struct MapSystem;

impl<'a> System<'a> for MapSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Blocker>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, positions, blockers, entities): Self::SystemData) {
        map.populate_passable();
        map.clear_content_index();
        for (pos, ent) in (&positions, &entities).join() {
            let idx = map.coords_to_idx(pos.x, pos.y);
            if blockers.get(ent).is_some() {
                map.passable[idx] = false;
            }
            map.tile_content[idx].push(ent);
        }
    }
}
