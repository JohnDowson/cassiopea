use crate::{components::*, gui::GameLog};
use specs::prelude::*;

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUp>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InInventory>,
    );

    fn run(
        &mut self,
        (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut inventory): Self::SystemData,
    ) {
        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);

            inventory
                .insert(
                    pickup.item,
                    InInventory {
                        owner: pickup.collector,
                        item: pickup.item,
                    },
                )
                .expect("Could not insert item in inventory");

            if pickup.collector == *player_entity {
                gamelog.entry(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemConsumptionSystem;

impl<'a> System<'a> for ItemConsumptionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Effect>,
        WriteStorage<'a, Stats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drink,
            names,
            potions,
            mut combat_stats,
        ) = data;

        for (entity, drink, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let effect = potions.get(drink.item);
            match effect {
                None => {}
                Some(effect) => match effect {
                    Effect::Heal(amount) => {
                        stats.hp = i32::min(stats.base_hp, stats.hp + amount);
                        if entity == *player_entity {
                            gamelog.entry(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(drink.item).unwrap().name,
                                amount
                            ));
                        }
                        entities.delete(drink.item).expect("Delete failed");
                    }
                },
            }
        }

        wants_drink.clear();
    }
}
