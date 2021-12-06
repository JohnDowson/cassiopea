use crate::{
    camera,
    components::*,
    game_save::{load_game, save_game},
    gui::{
        draw_ui, show_inventory, show_main_menu, show_targeting, GameLog, MainMenuSelection,
        TargetingResult,
    },
    map::Map,
    player::{player_input, Player},
    spawner,
    systems::{
        ai::EnemyAI,
        inventory_system::{ItemCollectionSystem, ItemConsumptionSystem},
        map_system::MapSystem,
        melee_combat::{DamageSystem, MeleeCombatSystem},
        visability::VisibilitySystem,
    },
};
use rltk::{GameState, Rltk};
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
    Targeting {
        range: i32,
        item: Entity,
        radius: Option<i32>,
    },
    MainMenu(MainMenuSelection),
    SaveGame,
    LoadGame,
    NextLayer,
    RevealMap(i32),
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

    fn next_layer(&mut self) {
        let delete = self.delete_on_level_change();

        self.ecs
            .delete_entities(&delete)
            .expect("Couldnt delete entities on level change");

        let map = {
            let mut map = self.ecs.write_resource::<Map>();
            let current_layer = map.layer;
            let dim_x = map.dim_x;
            let dim_y = map.dim_y;
            *map = Map::new(dim_x, dim_y, current_layer + 1);
            map.clone()
        };
        spawner::spawn_room(&mut self.ecs);

        let player_spawn = map.rooms[1].center();
        let mut player = self.ecs.fetch_mut::<Player>();
        player.position.x = player_spawn.0;
        player.position.y = player_spawn.1;

        let mut positions = self.ecs.write_storage::<Position>();
        if let Some(p_pos) = positions.get_mut(player.entity) {
            p_pos.x = player_spawn.0;
            p_pos.y = player_spawn.1;
        }

        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        if let Some(vis) = viewsheds.get_mut(player.entity) {
            vis.dirty = true
        }

        let mut log = self.ecs.write_resource::<GameLog>();
        log.entry("You descend to the next network layer".into())
    }

    fn delete_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.fetch::<Player>();
        let in_inventory = self.ecs.read_storage::<InInventory>();
        entities
            .par_join()
            .filter(|&e| {
                let in_player_inventory = if let Some(i) = in_inventory.get(e) {
                    i.owner == player.entity
                } else {
                    false
                };
                e != player.entity || in_player_inventory
            })
            .collect::<Vec<_>>()
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut new_run_state = { *self.ecs.fetch::<RunState>() };

        match new_run_state {
            RunState::MainMenu(_) => {}
            _ => {
                camera::render(&self.ecs, ctx);
                draw_ui(&self.ecs, ctx);
            }
        }
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
                                    radius: None,
                                };
                            }
                            Effect::DamageAOE {
                                range,
                                damage: _,
                                radius,
                            } => {
                                new_state = RunState::Targeting {
                                    range: *range,
                                    item: e,
                                    radius: Some(*radius),
                                };
                            }
                        },
                        None => {
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
                    };

                    new_state
                }
            },
            RunState::Targeting {
                range,
                item,
                radius,
            } => match show_targeting(&mut self.ecs, ctx, range, radius) {
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
                TargetingResult::NoResponse => RunState::Targeting {
                    range,
                    item,
                    radius,
                },
            },
            RunState::MainMenu(_) => match show_main_menu(&mut self.ecs, ctx) {
                crate::gui::MainMenuResult::Selected(sel) => RunState::MainMenu(sel),
                crate::gui::MainMenuResult::Confirmed(selection) => match selection {
                    MainMenuSelection::NewGame => RunState::PreRun,
                    MainMenuSelection::SaveGame => RunState::SaveGame,
                    MainMenuSelection::LoadGame => {
                        load_game(&mut self.ecs);
                        RunState::PreRun
                    }
                    MainMenuSelection::Quit => std::process::exit(0),
                },
            },
            RunState::SaveGame => {
                save_game(&mut self.ecs);
                RunState::MainMenu(MainMenuSelection::SaveGame)
            }
            RunState::LoadGame => todo!(),
            RunState::NextLayer => {
                self.next_layer();
                RunState::PreRun
            }
            RunState::RevealMap(mut y) => {
                let mut map = self.ecs.write_resource::<Map>();
                for x in 0..map.dim_x - 1 {
                    let idx = map.coords_to_idx(x, y);
                    map.revealed[idx] = true;
                }
                y += 1;
                if y == map.dim_y {
                    RunState::AwaitingInput
                } else {
                    RunState::RevealMap(y)
                }
            }
        };
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        self.delete_dead();
    }
}
