use cassiopea::{
    components::*,
    gui::GameLog,
    map::Map,
    state::{RunState, State},
};
use rltk::{to_cp437, RGB};
use specs::prelude::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .with_fps_cap(30.0)
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Control>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Enemy>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Blocker>();
    gs.ecs.register::<Stats>();
    gs.ecs.register::<MeleeAttack>();
    gs.ecs.register::<TakeDamage>();

    let map = Map::new(80, 43);

    let mut rng = rltk::RandomNumberGenerator::new();
    let (x, y) = rng.random_slice_entry(&map.rooms).unwrap().center();

    gs.ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Control)
        .with(Viewshed {
            visible_tiles: Default::default(),
            range: 8,
            dirty: true,
        })
        .with(Blocker)
        .with(Stats {
            base_power: 10,
            base_health: 20,
            hp: 20,
            defense: 3,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build();

    for r in &map.rooms {
        let (e_x, e_y) = r.center();
        if e_x == x && e_y == y {
            continue;
        }
        let (glyph, name) = rng
            .random_slice_entry(&[('&', "Snake"), ('$', "Skeleton")])
            .unwrap();
        gs.ecs
            .create_entity()
            .with(Position { x: e_x, y: e_y })
            .with(Renderable {
                glyph: to_cp437(*glyph),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Default::default(),
                range: 8,
                dirty: true,
            })
            .with(Enemy)
            .with(Name {
                name: name.to_string(),
            })
            .with(Blocker)
            .with(Stats {
                base_power: 5,
                base_health: 5,
                hp: 5,
                defense: 5,
            })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(GameLog::default());

    rltk::main_loop(context, gs)
}
