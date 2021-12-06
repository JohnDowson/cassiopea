use crate::{
    components::{Enemy, MeleeAttack, Name, Position, Viewshed},
    map::Map,
    player::Player,
    state::RunState,
};
use specs::prelude::*;

pub struct EnemyAI;

impl<'a> System<'a> for EnemyAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, MeleeAttack>,
        ReadExpect<'a, RunState>,
        ReadExpect<'a, Player>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut viewshed,
            mut pos,
            enemy,
            name,
            mut map,
            mut melee,
            run_state,
            player,
        ): Self::SystemData,
    ) {
        if *run_state != RunState::NPCTurn {
            return;
        }
        let want_to_melee = (&entities, &mut viewshed, &name, &enemy, &mut pos)
            .join()
            .filter_map(|(ent, viewshed, _name, _, pos)| {
                let distance = rltk::DistanceAlg::Pythagoras
                    .distance2d(pos.as_point(), player.position.as_point());
                if distance < 1.5 {
                    return Some(ent);
                }
                if viewshed.visible_tiles.contains(&player.position.as_point()) {
                    let path = rltk::a_star_search(
                        map.coords_to_idx(pos.x, pos.y),
                        map.coords_to_idx(player.position.x, player.position.y),
                        &*map,
                    );
                    if dbg! {path.success} && path.steps.len() > 1 {
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
                .insert(
                    entity,
                    MeleeAttack {
                        target: player.entity,
                    },
                )
                .expect("Failed to insert melee");
        }
    }
}
