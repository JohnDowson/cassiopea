use crate::{
    components::*,
    gui::{draw_ui, show_inventory, GameLog},
    map::{Map, Tile},
    player::player_input,
    systems::{
        ai::EnemyAI,
        inventory_system::{ItemCollectionSystem, ItemConsumptionSystem},
        map_system::MapSystem,
        melee_combat::{DamageSystem, MeleeCombatSystem},
        visability::VisibilitySystem,
    },
};
use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

pub struct State {
    pub ecs: World,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    PreRun,
    AwaitingInput,
    PlayerTurn,
    NPCTurn,
    ShowInventory,
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
        let mut item_collection = ItemCollectionSystem;
        item_collection.run_now(&self.ecs);
        let mut item_consumption = ItemConsumptionSystem;
        item_consumption.run_now(&self.ecs);
        self.ecs.maintain();
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
        self.ecs.maintain();
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
        self.draw_map(ctx);
        let mut new_run_state = { *self.ecs.fetch::<RunState>() };
        new_run_state = match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => player_input(&mut self.ecs, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::NPCTurn
            }
            RunState::NPCTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::ShowInventory => match show_inventory(&mut self.ecs, ctx) {
                crate::gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                crate::gui::ItemMenuResult::NoResponse => RunState::ShowInventory,
                crate::gui::ItemMenuResult::Selected(e) => {
                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                    intent
                        .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: e })
                        .expect("Unable to insert intent");

                    RunState::AwaitingInput
                }
            },
        };
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        self.delete_dead();
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
