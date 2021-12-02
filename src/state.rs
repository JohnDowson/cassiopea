use crate::{
    components::*,
    gui::{draw_ui, GameLog},
    map::{Map, Tile},
    systems::{
        ai::EnemyAI,
        map_system::MapSystem,
        melee_combat::{DamageSystem, MeleeCombatSystem},
        visability::VisibilitySystem,
    },
};
use rltk::{console, GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::cmp::{max, min};

pub struct State {
    pub ecs: World,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    PreRun,
    AwaitingInput,
    PlayerTurn,
    NPCTurn,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;
        vis.run_now(&self.ecs);
        let mut enemy = EnemyAI;
        enemy.run_now(&self.ecs);
        let mut map_sys = MapSystem;
        map_sys.run_now(&self.ecs);
        let mut melee_sys = MeleeCombatSystem;
        melee_sys.run_now(&self.ecs);
        let mut damage_sys = DamageSystem;
        damage_sys.run_now(&self.ecs);
        self.ecs.maintain()
    }

    fn delete_dead(&mut self) {
        let mut dead = Vec::new();
        {
            let stats = self.ecs.read_storage::<Stats>();
            let entities = self.ecs.entities();
            let players = self.ecs.read_storage::<Control>();
            let names = self.ecs.read_storage::<Name>();
            let mut log = self.ecs.write_resource::<GameLog>();

            for (ent, stat) in (&entities, &stats).join() {
                if stat.hp < 1 {
                    let player = players.get(ent);
                    match player {
                        None => {
                            let victim_name = names.get(ent);
                            if let Some(victim_name) = victim_name {
                                log.entry(format!("{} is dead", &victim_name.name));
                            }
                            dead.push(ent)
                        }
                        Some(_) => log.entry("You are dead".into()),
                    }
                }
            }
        }

        self.ecs
            .delete_entities(&dead)
            .expect("Unable to delete dead");
    }

    fn player_input(&mut self, ctx: &mut Rltk) -> RunState {
        use VirtualKeyCode::*;
        match ctx.key {
            None => return RunState::AwaitingInput,
            Some(key) => match key {
                A => self.try_move_player(-1, 0),
                D => self.try_move_player(1, 0),
                W => self.try_move_player(0, -1),
                S => self.try_move_player(0, 1),
                Q => self.try_move_player(-1, -1),
                E => self.try_move_player(1, -1),
                Z => self.try_move_player(-1, 1),
                X => self.try_move_player(1, 1),
                _ => return RunState::AwaitingInput,
            },
        }
        RunState::NPCTurn
    }

    fn try_move_player(&mut self, delta_x: i32, delta_y: i32) {
        let mut positions = self.ecs.write_storage::<Position>();
        let mut players = self.ecs.write_storage::<Control>();
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let stats = self.ecs.read_storage::<Stats>();
        let mut melee = self.ecs.write_storage::<MeleeAttack>();
        let entities = self.ecs.entities();
        let map = self.ecs.fetch::<Map>();

        for (entity, _, pos, vis) in
            (&entities, &mut players, &mut positions, &mut viewsheds).join()
        {
            let x = min(79, max(0, pos.x + delta_x));
            let y = min(49, max(0, pos.y + delta_y));
            for maybe_target in map.tile_content[map.coords_to_idx(x, y)].iter() {
                if let Some(_t) = stats.get(*maybe_target) {
                    melee
                        .insert(
                            entity,
                            MeleeAttack {
                                target: *maybe_target,
                            },
                        )
                        .expect("Can't insert Melee");
                    console::log("Player attacks");
                    return;
                }
            }
            if map.passable[map.coords_to_idx(x, y)] {
                pos.x = x;
                pos.y = y;
            }
            vis.dirty = true;
        }
    }

    fn draw_map(&mut self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.dim_x {
            for y in 0..map.dim_y {
                let coords = map.coords_to_idx(x, y);
                if map.revealed[coords] {
                    let tile = map[(x, y)];
                    let (glyph, fg) = match tile {
                        Tile::Floor => (
                            rltk::to_cp437('.'),
                            if map.visible[coords] {
                                RGB::from_f32(0.0, 0.5, 0.5)
                            } else {
                                RGB::from_f32(0.0, 0.2, 0.2)
                            },
                        ),
                        Tile::Wall => (
                            rltk::to_cp437('#'),
                            if map.visible[coords] {
                                RGB::from_f32(0.4, 0.4, 0.4)
                            } else {
                                RGB::from_f32(0.2, 0.2, 0.2)
                            },
                        ),
                    };
                    ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
                }
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_run_state = { *self.ecs.fetch::<RunState>() };
        new_run_state = match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => self.player_input(ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::NPCTurn
            }
            RunState::NPCTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
        };
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        self.delete_dead();

        self.draw_map(ctx);
        let map = self.ecs.fetch::<Map>();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            let coords = map.coords_to_idx(pos.x, pos.y);
            if map.visible[coords] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
            }
        }
        draw_ui(&self.ecs, ctx);
    }
}
