use std::{
    cmp::{max, min},
    ops::{BitAnd, Index, IndexMut},
};

use rltk::{Algorithm2D, BaseMap, Point, RGB};
use serde::{Deserialize, Serialize};
use specs::Entity;
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rect {
    top_left: (i32, i32),
    bottom_right: (i32, i32),
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            top_left: (x, y),
            bottom_right: (x + w, y + h),
        }
    }
    pub fn center(&self) -> (i32, i32) {
        let (x1, y1) = self.top_left;
        let (x2, y2) = self.bottom_right;
        ((x1 + x2) / 2, (y1 + y2) / 2)
    }
    pub fn coords(&self) -> (i32, i32, i32, i32) {
        let (x1, y1) = self.top_left;
        let (x2, y2) = self.bottom_right;

        (x1, y1, x2, y2)
    }
}

impl BitAnd<&Rect> for &Rect {
    type Output = bool;

    fn bitand(self, rhs: &Rect) -> Self::Output {
        let (x1, y1) = self.top_left;
        let (x2, y2) = self.bottom_right;
        let (rhs_x1, rhs_y1) = rhs.top_left;
        let (rhs_x2, rhs_y2) = rhs.bottom_right;
        x1 <= rhs_x2 && x2 >= rhs_x1 && y1 <= rhs_y2 && y2 >= rhs_y1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    inner: Vec<Tile>,
    pub rooms: Vec<Rect>,
    pub revealed: Vec<bool>,
    pub visible: Vec<bool>,
    pub passable: Vec<bool>,
    #[serde(skip)]
    pub tile_content: Vec<Vec<Entity>>,
    pub dim_x: i32,
    pub dim_y: i32,
    pub layer: i32,
}

impl Map {
    pub fn coords_to_idx(&self, x: i32, y: i32) -> usize {
        (y * self.dim_x) as usize + x as usize
    }

    pub fn idx_to_coords(&self, idx: usize) -> (i32, i32) {
        let x = idx as i32 % self.dim_x;
        let y = idx as i32 / self.dim_x;
        (x, y)
    }

    pub fn dimensions(&self) -> (i32, i32) {
        (self.dim_x - 1, self.dim_y - 1)
    }

    pub fn size(&self) -> usize {
        (self.dim_x * self.dim_y) as usize
    }

    pub fn new(dim_x: i32, dim_y: i32, layer: i32) -> Self {
        let mut new = Self {
            inner: vec![Tile::Wall; (dim_x * dim_y) as usize],
            rooms: Vec::new(),
            revealed: vec![false; (dim_x * dim_y) as usize],
            visible: vec![false; (dim_x * dim_y) as usize],
            passable: vec![false; (dim_x * dim_y) as usize],
            tile_content: vec![Vec::new(); (dim_x * dim_y) as usize],
            dim_x,
            dim_y,
            layer,
        };

        for x in 0..dim_x {
            new[(x, 0)] = Tile::Wall;
            new[(x, dim_y - 1)] = Tile::Wall;
        }
        for y in 0..dim_y {
            new[(0, y)] = Tile::Wall;
            new[(dim_x - 1, y)] = Tile::Wall;
        }

        let mut rng = rltk::RandomNumberGenerator::new();
        for _ in 0..dim_x / 4 {
            let w = rng.range(5, 20);
            let h = rng.range(3, 10);
            let x = rng.roll_dice(1, dim_x - w - 1);
            let y = rng.roll_dice(1, dim_y - h - 1);
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for room in new.rooms.iter() {
                ok = !(room & &new_room)
            }
            if ok {
                new.add_room(&new_room);

                if !new.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = new.rooms[new.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        new.add_horizontal_tunnel(prev_x, new_x, prev_y);
                        new.add_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        new.add_vertical_tunnel(prev_y, new_y, prev_x);
                        new.add_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                new.rooms.push(new_room);
            }
        }
        new.populate_passable();
        new
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn populate_passable(&mut self) {
        for (i, tile) in self.inner.iter().enumerate() {
            self.passable[i] = *tile == Tile::Floor;
        }
    }

    fn add_room(&mut self, room: &Rect) {
        let (x1, y1) = room.top_left;
        let (x2, y2) = room.bottom_right;
        for y in y1 + 1..=y2 {
            for x in x1 + 1..=x2 {
                self[(x, y)] = Tile::Floor;
            }
        }
    }

    fn add_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.coords_to_idx(x, y);
            if idx > 0 && idx < self.dim_x as usize * self.dim_y as usize {
                self.inner[idx] = Tile::Floor;
            }
        }
    }

    fn add_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.coords_to_idx(x, y);
            if idx > 0 && idx < self.dim_x as usize * self.dim_y as usize {
                self.inner[idx] = Tile::Floor;
            }
        }
    }

    fn exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.dim_x - 1 || y < 1 || y > self.dim_y - 1 {
            false
        } else {
            self.passable[self.coords_to_idx(x, y)]
        }
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        let idx = self.coords_to_idx(x, y);
        self.visible[idx]
    }

    pub fn get_tile_glyph(&self, x: i32, y: i32) -> (RGB, RGB, u16) {
        let idx = self.coords_to_idx(x, y);
        match self.inner[idx] {
            Tile::Floor => (
                if self.visible[idx] {
                    RGB::from_f32(0.0, 0.5, 0.5)
                } else {
                    RGB::from_f32(0.0, 0.2, 0.2)
                },
                RGB::named(rltk::BLACK),
                rltk::to_cp437('.'),
            ),
            Tile::Wall => (
                if self.visible[idx] {
                    RGB::from_f32(0.4, 0.4, 0.4)
                } else {
                    RGB::from_f32(0.2, 0.2, 0.2)
                },
                RGB::named(rltk::BLACK),
                rltk::to_cp437('#'),
            ),
        }
    }
}

impl Index<(i32, i32)> for Map {
    type Output = Tile;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        let (x, y) = index;
        let index = self.coords_to_idx(x, y);
        &self.inner[index]
    }
}
impl IndexMut<(i32, i32)> for Map {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        let (x, y) = index;
        let index = self.coords_to_idx(x, y);
        &mut self.inner[index]
    }
}

impl AsRef<[Tile]> for Map {
    fn as_ref(&self) -> &[Tile] {
        &self.inner[..]
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.inner[idx as usize] == Tile::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let (x, y) = self.idx_to_coords(idx);
        let w = self.dim_x as usize;

        if self.exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        if self.exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.dim_x as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.dim_x, self.dim_y)
    }
}
