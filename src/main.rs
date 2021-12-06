use cassiopea::{
    components::*,
    gui::{GameLog, MainMenuSelection},
    random::random_map_builder,
    spawner::player,
    state::{RunState, State},
};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .with_fps_cap(30.0)
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

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
    gs.ecs.register::<SerializationHelper>();

    let mut rng = rltk::RandomNumberGenerator::seeded(69);
    let mut builder = random_map_builder();
    let (map, player_spawn) = builder.build(60, 60, 0, &mut rng);

    gs.ecs.insert(rng);
    builder.spawn(&map, &mut gs.ecs, 0);
    let player = player(&mut gs.ecs, player_spawn);

    gs.ecs.insert(player);
    gs.ecs.insert(map);
    gs.ecs
        .insert(RunState::MainMenu(MainMenuSelection::NewGame));
    gs.ecs.insert(GameLog::default());

    rltk::main_loop(context, gs)
}
