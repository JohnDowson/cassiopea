use crate::{components::*, map::Map, player::Player};
use rltk::Rltk;
use specs::prelude::*;

pub fn render(ecs: &World, ctx: &mut Rltk) {
    let (min_x, min_y, max_x, max_y) = get_bounds(ecs, ctx);

    render_map(ecs, min_y, max_y, min_x, max_x, ctx);

    render_entities(ecs, min_x, min_y, ctx);
}

pub fn get_bounds(ecs: &World, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = (*ecs.fetch::<Player>()).position;
    let (x_dim, y_dim) = ctx.get_char_size();
    let (center_x, center_y) = (x_dim / 2, y_dim / 2);
    let min_x = player_pos.x - center_x as i32;
    let min_y = player_pos.y - center_y as i32;
    let max_x = min_x + x_dim as i32;
    let max_y = min_y + y_dim as i32;
    (min_x, min_y, max_x, max_y)
}

fn render_map(ecs: &World, min_y: i32, max_y: i32, min_x: i32, max_x: i32, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (map_width, map_height) = map.dimensions();
    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.coords_to_idx(tx, ty);
                if map.revealed[idx] {
                    let (fg, bg, glyph) = map.get_tile_glyph(tx, ty);
                    ctx.set(x, y, fg, bg, glyph);
                }
            }
        }
    }
}

fn render_entities(ecs: &World, min_x: i32, min_y: i32, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (map_width, map_height) = map.dimensions();
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut to_render = (&positions, &renderables).par_join().collect::<Vec<_>>();
    to_render.sort_by(|&(_, r1), &(_, r2)| r1.render_order.cmp(&r2.render_order));
    for (pos, render) in to_render.into_iter() {
        let idx = map.coords_to_idx(pos.x, pos.y);
        if map.visible[idx] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0
                && entity_screen_x < map_width
                && entity_screen_y > 0
                && entity_screen_y < map_height
            {
                ctx.set(
                    entity_screen_x,
                    entity_screen_y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
            }
        }
    }
}
