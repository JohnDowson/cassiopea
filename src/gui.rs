use crate::components::{Control, Stats};
use rltk::{Rltk, RGB};
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
    let player = ecs.read_storage::<Control>();
    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(40, y, s);
        }
        y += 1;
    }

    for (_, stats) in (&player, &stats).join() {
        let health = format!("{}/{}", stats.hp, stats.base_health);

        ctx.print(1, 43, &health);
        ctx.draw_bar_vertical(
            1,
            44,
            5,
            stats.hp,
            stats.base_health,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        )
    }
}
