use rltk::{Algorithm2D, BaseMap, Point, RGB};
use serde::{Deserialize, Serialize};
use specs::Entity;
use std::{
    cmp::{max, min},
    ops::{Index, IndexMut},
};

pub mod builders;

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    TerminalDown,
    TerminalUp,
    TerminalService,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
    pub fn x1(&self) -> i32 {
        self.top_left.0
    }
    pub fn x2(&self) -> i32 {
        self.bottom_right.0
    }
    pub fn y1(&self) -> i32 {
        self.top_left.1
    }
    pub fn y2(&self) -> i32 {
        self.bottom_right.1
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

    fn intersects(&self, other: &Self) -> bool {
        let (x1, y1) = self.top_left;
        let (x2, y2) = self.bottom_right;
        let (rhs_x1, rhs_y1) = other.top_left;
        let (rhs_x2, rhs_y2) = other.bottom_right;
        x1 <= rhs_x2 && x2 >= rhs_x1 && y1 <= rhs_y2 && y2 >= rhs_y1
    }
}

#[test]
fn intersecting() {
    let a = Rect::new(0, 0, 10, 10);
    let b = Rect::new(0, 0, 10, 10);
    assert! {a.intersects(&b)};
    let a = Rect::new(11, 0, 10, 10);
    let b = Rect::new(0, 0, 10, 10);
    assert! {!a.intersects(&b)};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    inner: Vec<Tile>,
    pub rooms: Vec<Rect>,
    pub revealed: Vec<bool>,
    #[serde(skip)]
    pub visible: Vec<bool>,
    #[serde(skip)]
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
        Self {
            inner: vec![Tile::Wall; (dim_x * dim_y) as usize],
            rooms: Vec::new(),
            revealed: vec![false; (dim_x * dim_y) as usize],
            visible: vec![false; (dim_x * dim_y) as usize],
            passable: vec![false; (dim_x * dim_y) as usize],
            tile_content: vec![Vec::new(); (dim_x * dim_y) as usize],
            dim_x,
            dim_y,
            layer,
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn populate_passable(&mut self) {
        for (i, tile) in self.inner.iter().enumerate() {
            const PASSABLE_TILES: [Tile; 4] = [
                Tile::Floor,
                Tile::TerminalDown,
                Tile::TerminalService,
                Tile::TerminalUp,
            ];
            self.passable[i] = PASSABLE_TILES.contains(tile);
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
                self.wall_glyph(x, y),
            ),
            Tile::TerminalDown => (
                if self.visible[idx] {
                    RGB::from_f32(0.4, 0.4, 0.4)
                } else {
                    RGB::from_f32(0.2, 0.2, 0.2)
                },
                RGB::named(rltk::SKY_BLUE),
                rltk::to_cp437('▼'),
            ),
            Tile::TerminalUp => (
                if self.visible[idx] {
                    RGB::from_f32(0.4, 0.4, 0.4)
                } else {
                    RGB::from_f32(0.2, 0.2, 0.2)
                },
                RGB::named(rltk::SKY_BLUE),
                rltk::to_cp437('▲'),
            ),
            Tile::TerminalService => (
                if self.visible[idx] {
                    RGB::from_f32(0.4, 0.4, 0.4)
                } else {
                    RGB::from_f32(0.2, 0.2, 0.2)
                },
                RGB::named(rltk::SKY_BLUE),
                rltk::to_cp437('◙'),
            ),
        }
    }
    fn is_revealed_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.coords_to_idx(x, y);
        self.revealed[idx] && self.inner[idx] == Tile::Wall
    }
    fn wall_glyph(&self, x: i32, y: i32) -> u16 {
        if x < 1 || x > self.dim_x - 2 || y < 1 || y > self.dim_y - 2 {
            return 35;
        }
        let mut mask = 0u8;
        if self.is_revealed_wall(x, y - 1) {
            mask += 1;
        }
        if self.is_revealed_wall(x, y + 1) {
            mask += 2;
        }
        if self.is_revealed_wall(x - 1, y) {
            mask += 4;
        }
        if self.is_revealed_wall(x + 1, y) {
            mask += 8;
        }
        match mask {
            0 => 9,    // Pillar because we can't see neighbors
            1 => 186,  // Wall only to the north
            2 => 186,  // Wall only to the south
            3 => 186,  // Wall to the north and south
            4 => 205,  // Wall only to the west
            5 => 188,  // Wall to the north and west
            6 => 187,  // Wall to the south and west
            7 => 185,  // Wall to the north, south and west
            8 => 205,  // Wall only to the east
            9 => 200,  // Wall to the north and east
            10 => 201, // Wall to the south and east
            11 => 204, // Wall to the north, south and east
            12 => 205, // Wall to the east and west
            13 => 202, // Wall to the east, west, and south
            14 => 203, // Wall to the east, west, and north
            15 => 206, // ╬ Wall on all sides
            _ => 35,   // We missed one?
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
