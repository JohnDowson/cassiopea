use specs::prelude::*;

use crate::{
    components::{TakeDamage, TraceTimer},
    gui::GameLog,
    player::Player,
    state::RunState,
};

pub struct TraceTimerSystem;

impl<'a> System<'a> for TraceTimerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TraceTimer>,
        ReadExpect<'a, RunState>,
        ReadExpect<'a, Player>,
        WriteStorage<'a, TakeDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, (mut trace, state, player, mut damage, mut log): Self::SystemData) {
        if *state == RunState::PlayerTurn {
            let player_trace = trace
                .get_mut(player.entity)
                .expect("Player has no trace component");
            player_trace.timer -= 1;
            if player_trace.timer <= 10 {
                log.entry("Corporate hackers are almost caught up to you".into());
                if player_trace.timer < 0 {
                    log.entry("Netrunners have tracked you down".into());
                    damage
                        .insert(player.entity, TakeDamage { amount: 1 })
                        .expect("Could not damage player");
                }
            }
        }
    }
}
