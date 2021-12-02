use crate::{
    components::{Control, Position, Viewshed},
    map::Map,
};
use rltk::{field_of_view_set, Point};
use specs::prelude::*;

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Control>,
    );

    fn run(&mut self, (mut map, entities, mut viewshed, pos, player): Self::SystemData) {
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view_set(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.dim_x && p.y >= 0 && p.y < map.dim_y);

                if player.get(ent).is_some() {
                    for t in map.visible.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.coords_to_idx(vis.x, vis.y);
                        map.revealed[idx] = true;
                        map.visible[idx] = true;
                    }
                }
            }
        }
    }
}
