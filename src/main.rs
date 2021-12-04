use cassiopea::{
    components::*,
    gui::GameLog,
    map::Map,
    spawner::{player, spawn_room},
    state::{RunState, State},
};
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Effect>();
    gs.ecs.register::<InInventory>();
    gs.ecs.register::<HasInventory>();
    gs.ecs.register::<WantsToPickUp>();
    gs.ecs.register::<WantsToUseItem>();

    let map = Map::new(80, 43);

    let mut rng = rltk::RandomNumberGenerator::new();
    let (x, y) = rng.random_slice_entry(&map.rooms).unwrap().center();
    let player = player(&mut gs.ecs, x, y);

    gs.ecs.insert(player);
    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(GameLog::default());
    gs.ecs.insert(rltk::RandomNumberGenerator::seeded(69));

    spawn_room(&mut gs.ecs);

    rltk::main_loop(context, gs)
}
