use std::cmp::{max, min};

use crate::components::*;
use crate::gui::GameLog;
use crate::{map::Map, state::RunState};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub struct Player {
    pub entity: Entity,
    pub position: Point,
}

pub fn player_input(ecs: &mut World, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            A => try_move_player(ecs, -1, 0),
            D => try_move_player(ecs, 1, 0),
            W => try_move_player(ecs, 0, -1),
            S => try_move_player(ecs, 0, 1),
            Q => try_move_player(ecs, -1, -1),
            E => try_move_player(ecs, 1, -1),
            Z => try_move_player(ecs, -1, 1),
            X => try_move_player(ecs, 1, 1),
            G => get_item(ecs),
            I => return RunState::ShowInventory,
            _ => return RunState::AwaitingInput,
        },
    }
    RunState::NPCTurn
}

fn get_item(ecs: &mut World) {
    let player = ecs.fetch::<Player>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player.position.x && position.y == player.position.y {
            target_item = Some(item_entity);
        }
    }
    match target_item {
        None => gamelog.entry("There is nothing here to pick up.".to_string()),
        Some(item) => {
            println!("Picking up");
            let mut pickup = ecs.write_storage::<WantsToPickUp>();
            pickup
                .insert(
                    player.entity,
                    WantsToPickUp {
                        collector: player.entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Control>();
    let mut player = ecs.write_resource::<Player>();
    let stats = ecs.read_storage::<Stats>();
    let mut melee = ecs.write_storage::<MeleeAttack>();
    let map = ecs.fetch::<Map>();

    for (_, pos, vis) in (&mut players, &mut positions, &mut viewsheds).join() {
        let x = min(79, max(0, pos.x + delta_x));
        let y = min(49, max(0, pos.y + delta_y));
        for maybe_target in map.tile_content[map.coords_to_idx(x, y)].iter() {
            if let Some(_t) = stats.get(*maybe_target) {
                melee
                    .insert(
                        player.entity,
                        MeleeAttack {
                            target: *maybe_target,
                        },
                    )
                    .expect("Can't insert Melee");
                return;
            }
        }
        if map.passable[map.coords_to_idx(x, y)] {
            pos.x = x;
            pos.y = y;
            player.position.x = x;
            player.position.y = y;
        }
        vis.dirty = true;
    }
}
