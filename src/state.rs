use crate::{
    components::*,
    gui::{draw_ui, show_inventory, show_targeting, GameLog, TargetingResult},
    map::{Map, Tile},
    player::{player_input, Player},
    systems::{
        ai::EnemyAI,
        inventory_system::{ItemCollectionSystem, ItemConsumptionSystem},
        map_system::MapSystem,
        melee_combat::{DamageSystem, MeleeCombatSystem},
        visability::VisibilitySystem,
    },
};
use rltk::{GameState, Rltk, RGB};
use specs::{prelude::*, rayon::iter::ParallelExtend};

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
    Targeting { range: i32, item: Entity },
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
        let mut items = Vec::new();
        {
            let stats = self.ecs.read_storage::<Stats>();
            let entities = self.ecs.entities();
            let players = self.ecs.read_storage::<Control>();
            let names = self.ecs.read_storage::<Name>();
            let inventory = self.ecs.read_storage::<HasInventory>();
            let mut in_inventory = self.ecs.write_storage::<InInventory>();
            let mut position = self.ecs.write_storage::<Position>();
            let mut log = self.ecs.write_resource::<GameLog>();

            for (ent, stat, pos) in (&entities, &stats, &position).join() {
                if stat.hp <= 0 {
                    if inventory.get(ent).is_some() {
                        items.par_extend(in_inventory.par_join().filter_map(|item| {
                            if item.owner == ent {
                                Some((item.item, *pos))
                            } else {
                                None
                            }
                        }));
                    }
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

            for (item, pos) in items {
                in_inventory.remove(item);
                position
                    .insert(item, pos)
                    .expect("Failed to inser position");
            }
        }
        self.ecs
            .delete_entities(&dead)
            .expect("Unable to delete dead");
        self.ecs.maintain();
    }

    fn draw_entities(&mut self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let mut to_render = (&positions, &renderables).join().collect::<Vec<_>>();
        to_render.sort_by(|&(_, r1), &(_, r2)| r1.render_order.cmp(&r2.render_order));
        for (pos, render) in to_render {
            let coords = map.coords_to_idx(pos.x, pos.y);
            if map.visible[coords] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
            }
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
                    let effect = self.ecs.read_storage::<Effect>();
                    let mut new_state = RunState::PlayerTurn;
                    match effect.get(e) {
                        Some(effect) => match effect {
                            Effect::HealSelf(_) => {
                                intent
                                    .insert(
                                        self.ecs.fetch::<Player>().entity,
                                        WantsToUseItem {
                                            item: e,
                                            target: Target::Itself,
                                        },
                                    )
                                    .expect("Unable to insert intent");
                            }
                            Effect::DamageRanged { range, .. } => {
                                new_state = RunState::Targeting {
                                    range: *range,
                                    item: e,
                                };
                            }
                            Effect::DamageAOE { range, .. } => {
                                new_state = RunState::Targeting {
                                    range: *range,
                                    item: e,
                                };
                            }
                        },
                        None => todo!(),
                    };

                    new_state
                }
            },
            RunState::Targeting { range, item } => {
                match show_targeting(&mut self.ecs, ctx, range) {
                    TargetingResult::Cancel => RunState::AwaitingInput,
                    TargetingResult::Tile(x, y) => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                self.ecs.fetch::<Player>().entity,
                                WantsToUseItem {
                                    item,
                                    target: Target::Tile(x, y),
                                },
                            )
                            .expect("Unable to insert intent");
                        RunState::PlayerTurn
                    }
                    TargetingResult::Entity(_) => todo!(),
                    TargetingResult::NoResponse => RunState::Targeting { range, item },
                }
            }
        };
        self.draw_entities(ctx);
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        self.delete_dead();

        draw_ui(&self.ecs, ctx);
    }
}
