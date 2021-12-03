use crate::{
    components::{Control, Enemy, MeleeAttack, Name, Position, Viewshed},
    map::Map,
    state::RunState,
};
use specs::prelude::*;

pub struct EnemyAI;

impl<'a> System<'a> for EnemyAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Control>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, MeleeAttack>,
        ReadExpect<'a, RunState>,
    );

    fn run(
        &mut self,
        (entities, control, mut viewshed, mut pos, enemy, name, mut map, mut melee, run_state): Self::SystemData,
    ) {
        if *run_state != RunState::NPCTurn {
            return;
        }
        let (p_ent, p_pos) = {
            let (e, _, p_pos) = (&entities, &control, &pos)
                .join()
                .next()
                .expect("No player");
            (e, *p_pos)
        };
        let want_to_melee = (&entities, &mut viewshed, &name, &enemy, &mut pos)
            .join()
            .filter_map(|(ent, viewshed, _name, _, pos)| {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(pos.as_point(), p_pos.as_point());
                if distance < 1.5 {
                    return Some(ent);
                }
                if viewshed.visible_tiles.contains(&p_pos.as_point()) {
                    let path = rltk::a_star_search(
                        map.coords_to_idx(pos.x, pos.y),
                        map.coords_to_idx(p_pos.x, p_pos.y),
                        &*map,
                    );
                    if path.success && path.steps.len() > 1 {
                        pos.x = path.steps[1] as i32 % map.dim_x;
                        pos.y = path.steps[1] as i32 / map.dim_x;
                        map.passable[path.steps[1]] = false;
                        viewshed.dirty = true;
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        for entity in want_to_melee {
            melee
                .insert(entity, MeleeAttack { target: p_ent })
                .expect("Failed to insert melee");
        }
    }
}
