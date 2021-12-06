#![allow(deprecated)]
use std::{fs::File, path::Path};

use crate::{components::*, map::Map, player::Player};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{
        DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker,
        SimpleMarkerAllocator,
    },
    World,
};

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        eprintln! {"Deserializing {}", std::any::type_name::<$type>()};
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    let mapcopy = ecs.get_mut::<Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Control,
            Viewshed,
            Enemy,
            Name,
            Blocker,
            Stats,
            TakeDamage,
            MeleeAttack,
            Item,
            Consumable,
            Equippable,
            Slots,
            Equipped,
            InInventory,
            HasInventory,
            WantsToPickUp,
            WantsToUseItem,
            Effect,
            EquipBonus,
            LevelUp,
            SerializationHelper
        );
    }
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}

pub fn load_game(ecs: &mut World) {
    ecs.delete_all();

    let data = std::fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Control,
            Viewshed,
            Enemy,
            Name,
            Blocker,
            Stats,
            TakeDamage,
            MeleeAttack,
            Item,
            Consumable,
            Equippable,
            Slots,
            Equipped,
            InInventory,
            HasInventory,
            WantsToPickUp,
            WantsToUseItem,
            Effect,
            EquipBonus,
            LevelUp,
            SerializationHelper
        );
    }

    let mut deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let control = ecs.read_storage::<Control>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); h.map.size()];
            worldmap.visible = vec![false; h.map.size()];
            worldmap.passable = vec![false; h.map.size()];
            worldmap.populate_passable();
            deleteme = Some(e);
        }
        let mut vis = ecs.write_storage::<Viewshed>();
        for vis in (&mut vis).join() {
            vis.dirty = true;
        }

        for (e, _p, pos) in (&entities, &control, &position).join() {
            let mut player_resource = ecs.write_resource::<Player>();
            *player_resource = Player {
                entity: e,
                position: *pos,
            };
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
}

pub fn save_exists() -> bool {
    Path::new("./savegame.json").exists()
}
