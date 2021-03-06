use cassiopea::{
    components::*,
    gui::{GameLog, MainMenuSelection},
    spawner::player,
    state::{RunState, State},
    systems::particle,
};
use rltk::RGBA;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .with_font_bg("Curses-square-24.png", 384, 384, RGBA::from_u8(0, 0, 0, 0))
        .with_fps_cap(30.0)
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.register::<SerializationHelper>();

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
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Slots>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<EquipBonus>();
    gs.ecs.register::<Effect>();
    gs.ecs.register::<InInventory>();
    gs.ecs.register::<HasInventory>();
    gs.ecs.register::<WantsToPickUp>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<LevelUp>();
    gs.ecs.register::<Particle>();
    gs.ecs.register::<TraceTimer>();
    gs.ecs.register::<Drops>();

    let player = player(&mut gs.ecs);
    gs.ecs.insert(player);
    let seed = rltk::RandomNumberGenerator::new().rand::<u64>();
    gs.ecs.insert(rltk::RandomNumberGenerator::seeded(seed));

    gs.generate_map(128, 128, 0);

    gs.ecs
        .insert(RunState::MainMenu(MainMenuSelection::NewGame));
    gs.ecs.insert(GameLog::default());
    gs.ecs.insert(particle::RequestQueue::new());

    rltk::main_loop(context, gs)
}
