use std::cmp::min;

use crate::{
    camera,
    components::{
        Consumable, Control, Equippable, Equipped, InInventory, Name, Position, Slot, Slots, Stats,
        Viewshed,
    },
    map::Map,
    player::Player,
    state::RunState,
    DBG_SHOW_COORDINATE_TOOLTIP,
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub struct GameLog {
    entries: Vec<String>,
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }
    pub fn entry(&mut self, msg: String) {
        self.entries.push(msg)
    }
}

impl Default for GameLog {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a GameLog {
    type Item = &'a String;

    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    let stats = ecs.read_storage::<Stats>();
    let player_control = ecs.read_storage::<Control>();
    let map = ecs.read_resource::<Map>();
    let log = ecs.fetch::<GameLog>();
    let equipped = ecs.read_storage::<Equipped>();
    let slots = ecs.read_storage::<Slots>();
    let names = ecs.read_storage::<Name>();

    ctx.print_centered(43, format!("LAYER#{}", map.layer));

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(42, y, s);
        }
        y += 1;
    }

    let player = ecs.fetch::<Player>();
    for (_, stats) in (&player_control, &stats).join() {
        let player_items = (&equipped, &names)
            .par_join()
            .filter(|&(e, _)| e.owner == player.entity)
            .map(|(e, n)| (e.slot, &*n.name))
            .collect::<Vec<(Slot, &str)>>();
        for (i, slot) in slots
            .get(player.entity)
            .expect("This player has no slots")
            .slots
            .iter()
            .enumerate()
        {
            let slot_item = player_items
                .iter()
                .find_map(
                    |(i_slot, name)| {
                        if i_slot == slot {
                            Some(*name)
                        } else {
                            None
                        }
                    },
                )
                .unwrap_or("None");
            ctx.print(21, 44 + i, format!("{:?}:{}", slot, slot_item));
        }

        ctx.print(13, 44, format!("ATK:{}", stats.base_power));
        ctx.print(13, 45, format!("DEF:{}", stats.defense));

        const MAX_BARS: i32 = 5;
        let health = format!("{}/{}", stats.hp, stats.base_hp);
        ctx.print(1, 43, &health);
        let per_bar = stats.base_hp / MAX_BARS;
        for i in 0..min(stats.base_hp / per_bar, MAX_BARS) {
            ctx.draw_bar_vertical(
                1 + i,
                44,
                5,
                (stats.hp - i * per_bar).clamp(0, per_bar),
                per_bar,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
            );
        }

        let compute = format!("{}/{}", stats.compute, stats.base_compute);
        ctx.print(7, 43, &compute);
        let per_bar = stats.base_compute / MAX_BARS;
        for i in 0..min(stats.base_compute / per_bar, MAX_BARS) {
            ctx.draw_bar_vertical(
                7 + i,
                44,
                5,
                (stats.compute - i * per_bar).clamp(0, per_bar),
                per_bar,
                RGB::named(rltk::BLUEVIOLET),
                RGB::named(rltk::BLACK),
            );
        }
    }
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(ecs, ctx)
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, min_y, _, _) = camera::get_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let mouse_pos = ctx.mouse_pos();
    let mut map_mouse_pos = mouse_pos;
    map_mouse_pos.0 += min_x;
    map_mouse_pos.1 += min_y;

    if mouse_pos.0 >= map.dim_x || mouse_pos.1 >= map.dim_y || mouse_pos.0 < 1 || mouse_pos.1 < 1 {
        return;
    }
    let tooltip: Vec<String> = (&names, &positions)
        .par_join()
        .filter_map(|(name, position)| {
            if position.x == map_mouse_pos.0
                && position.y == map_mouse_pos.1
                && map.is_visible(position.x, position.y)
            {
                Some(name.name.clone())
            } else {
                None
            }
        })
        .collect();

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::BLACK),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::BLACK),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::BLACK),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::BLACK),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::BLACK),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::BLACK),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    } else if DBG_SHOW_COORDINATE_TOOLTIP {
        let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
        let left_x = mouse_pos.0 - 8;
        let y = mouse_pos.1;
        let s = format!("x{};y{}", map_mouse_pos.0, map_mouse_pos.1);
        ctx.print_color(
            left_x,
            y,
            RGB::named(rltk::BLACK),
            RGB::named(rltk::GREY),
            &s,
        );
        let padding = (8 - s.len() as i32) - 1;
        for i in 0..padding {
            ctx.print_color(
                arrow_pos.x - i,
                y,
                RGB::named(rltk::BLACK),
                RGB::named(rltk::GREY),
                &" ".to_string(),
            );
        }
    }
}

pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn show_inventory(ecs: &mut World, ctx: &mut Rltk) -> ItemMenuResult {
    let player = ecs.fetch::<Player>();
    let names = ecs.read_storage::<Name>();
    let inventory = ecs.write_storage::<InInventory>();
    let consumables = ecs.read_storage::<Consumable>();
    let equippables = ecs.read_storage::<Equippable>();

    let inventory = (&inventory, &names)
        .par_join()
        .filter_map(|(inv, name)| {
            if inv.owner == player.entity {
                Some((inv.item, name.name.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    for (j, (_, name)) in inventory.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, name);
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    let entity = inventory[selection as usize].0;
                    if consumables.get(entity).is_some() || equippables.get(entity).is_some() {
                        ItemMenuResult::Selected(entity)
                    } else {
                        ItemMenuResult::NoResponse
                    }
                } else {
                    ItemMenuResult::NoResponse
                }
            }
        },
    }
}

pub enum TargetingResult {
    Cancel,
    Tile(i32, i32),
    Entity(Entity),
    NoResponse,
}

pub fn show_targeting(
    ecs: &mut World,
    ctx: &mut Rltk,
    range: i32,
    radius: Option<i32>,
) -> TargetingResult {
    let (min_x, min_y, max_x, max_y) = camera::get_bounds(ecs, ctx);
    let player = ecs.fetch::<Player>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    let prompt = "TARGETING";
    ctx.print_color(
        40 - (prompt.len() / 2),
        42,
        RGB::named(rltk::RED),
        RGB::named(rltk::BLACK),
        prompt,
    );

    let mut available = Vec::new();
    if let Some(visible) = viewsheds.get(player.entity) {
        for p in &visible.visible_tiles {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(player.position.as_point(), *p);
            if distance <= range as f32 {
                let screen_x = p.x - min_x;
                let screen_y = p.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::YELLOWGREEN));
                    available.push(p);
                }
            }
        }
    } else {
        return TargetingResult::Cancel;
    }

    let mouse_pos = ctx.mouse_pos();
    let mut map_mouse_pos = mouse_pos;
    map_mouse_pos.0 += min_x;
    map_mouse_pos.1 += min_y;
    let valid_target = available
        .into_iter()
        .any(|p| p.x == map_mouse_pos.0 && p.y == map_mouse_pos.1);

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if let (Some(radius), Some(visible)) = (radius, viewsheds.get(player.entity)) {
            for p in &visible.visible_tiles {
                let distance = rltk::DistanceAlg::Pythagoras
                    .distance2d(Point::new(map_mouse_pos.0, map_mouse_pos.1), *p);
                if distance <= radius as f32 {
                    let screen_x = p.x - min_x;
                    let screen_y = p.y - min_y;
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::ORANGE_RED));
                }
            }
        }
        if ctx.left_click {
            return TargetingResult::Tile(map_mouse_pos.0, map_mouse_pos.1);
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return TargetingResult::Cancel;
        }
    }
    TargetingResult::NoResponse
}

#[derive(Clone, Copy, PartialEq)]
pub enum MainMenuSelection {
    NewGame,
    SaveGame,
    LoadGame,
    Quit,
}
pub enum MainMenuResult {
    Selected(MainMenuSelection),
    Confirmed(MainMenuSelection),
}

pub fn show_main_menu(ecs: &mut World, ctx: &mut Rltk) -> MainMenuResult {
    static ACTIVE: (u8, u8, u8) = rltk::YELLOWGREEN;
    static INACTIVE: (u8, u8, u8) = rltk::WHITE;
    let run_state = ecs.fetch::<RunState>();
    if let RunState::MainMenu(selection) = *run_state {
        let (new_game_color, load_game_color, quit_color, save_color) = match selection {
            MainMenuSelection::NewGame => (ACTIVE, INACTIVE, INACTIVE, INACTIVE),
            MainMenuSelection::SaveGame => (INACTIVE, INACTIVE, INACTIVE, ACTIVE),
            MainMenuSelection::LoadGame => (INACTIVE, ACTIVE, INACTIVE, INACTIVE),
            MainMenuSelection::Quit => (INACTIVE, INACTIVE, ACTIVE, INACTIVE),
        };
        ctx.print_color_centered(
            20,
            RGB::named(new_game_color),
            RGB::named(rltk::BLACK),
            "New Game",
        );
        ctx.print_color_centered(
            21,
            RGB::named(save_color),
            RGB::named(rltk::BLACK),
            "Save Game",
        );
        ctx.print_color_centered(
            22,
            RGB::named(load_game_color),
            RGB::named(rltk::BLACK),
            "Load Game",
        );
        ctx.print_color_centered(23, RGB::named(quit_color), RGB::named(rltk::BLACK), "Quit");
        use VirtualKeyCode::*;
        match ctx.key {
            Some(key) => match key {
                Up => selection_next(selection),
                Down => selection_previous(selection),
                Return => MainMenuResult::Confirmed(selection),
                _ => MainMenuResult::Selected(selection),
            },
            None => MainMenuResult::Selected(selection),
        }
    } else {
        MainMenuResult::Selected(MainMenuSelection::NewGame)
    }
}

fn selection_next(selection: MainMenuSelection) -> MainMenuResult {
    MainMenuResult::Selected(match selection {
        MainMenuSelection::NewGame => MainMenuSelection::Quit,
        MainMenuSelection::SaveGame => MainMenuSelection::NewGame,
        MainMenuSelection::LoadGame => MainMenuSelection::SaveGame,
        MainMenuSelection::Quit => MainMenuSelection::LoadGame,
    })
}

fn selection_previous(selection: MainMenuSelection) -> MainMenuResult {
    MainMenuResult::Selected(match selection {
        MainMenuSelection::NewGame => MainMenuSelection::SaveGame,
        MainMenuSelection::SaveGame => MainMenuSelection::LoadGame,
        MainMenuSelection::LoadGame => MainMenuSelection::Quit,
        MainMenuSelection::Quit => MainMenuSelection::NewGame,
    })
}
