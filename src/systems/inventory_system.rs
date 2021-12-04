use crate::{components::*, gui::GameLog, map::Map, player::Player};
use rltk::Point;
use specs::prelude::*;

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUp>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InInventory>,
        ReadStorage<'a, HasInventory>,
    );

    fn run(
        &mut self,
        (
            player,
            mut gamelog,
            mut wants_pickup,
            mut positions,
            names,
            mut in_inventory,
            has_inventory,
        ): Self::SystemData,
    ) {
        for (pickup, _) in (&wants_pickup, &has_inventory).join() {
            positions.remove(pickup.item);

            in_inventory
                .insert(
                    pickup.item,
                    InInventory {
                        owner: pickup.collector,
                        item: pickup.item,
                    },
                )
                .expect("Could not insert item in inventory");

            if pickup.collector == player.entity {
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
        ReadExpect<'a, Player>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Effect>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, TakeDamage>,
        ReadExpect<'a, Map>,
        ReadStorage<'a, Item>,
    );

    fn run(
        &mut self,
        (
            player,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            effects,
            mut combat_stats,
            mut take_damage,
            map,
            items,
        ): Self::SystemData,
    ) {
        for (entity, wants, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let effect = effects.get(wants.item);
            match effect {
                None => {}
                Some(effect) => match effect {
                    Effect::HealSelf(amount) => {
                        stats.hp = i32::min(stats.base_hp, stats.hp + amount);
                        if entity == player.entity {
                            gamelog.entry(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(wants.item).unwrap().name,
                                amount
                            ));
                        }
                        entities.delete(wants.item).expect("Delete failed");
                    }
                    Effect::DamageRanged { range: _, damage } => match wants.target {
                        Target::Itself => todo!(),
                        Target::Other(_) => todo!(),
                        Target::Tile(x, y) => {
                            let idx = map.coords_to_idx(x, y);
                            for mob in &map.tile_content[idx] {
                                if !items.contains(*mob) {
                                    TakeDamage::new_damage(&mut take_damage, *mob, *damage);
                                    if entity == player.entity {
                                        gamelog.entry(format!(
                                            "You use the {}, damaging {} for {} hp.",
                                            names.get(wants.item).unwrap().name,
                                            names.get(*mob).unwrap(),
                                            damage
                                        ));
                                    }
                                }
                            }
                            entities.delete(wants.item).expect("Delete failed");
                        }
                    },
                    Effect::DamageAOE {
                        range: _,
                        damage,
                        radius,
                    } => match wants.target {
                        Target::Itself => todo!(),
                        Target::Other(_) => todo!(),
                        Target::Tile(x, y) => {
                            let mut blast_tiles =
                                rltk::field_of_view(Point::new(x, y), *radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.dim_x - 1 && p.y > 0 && p.y < map.dim_y - 1
                            });
                            for tile_idx in blast_tiles.into_iter() {
                                let idx = map.coords_to_idx(tile_idx.x, tile_idx.y);
                                for mob in &map.tile_content[idx] {
                                    TakeDamage::new_damage(&mut take_damage, *mob, *damage);
                                    if entity == player.entity {
                                        gamelog.entry(format!(
                                            "You use the {}, damaging {} for {} hp.",
                                            names.get(wants.item).unwrap().name,
                                            names.get(*mob).unwrap(),
                                            damage
                                        ));
                                    }
                                }
                            }
                            entities.delete(wants.item).expect("Delete failed");
                        }
                    },
                },
            }
        }

        wants_use.clear();
    }
}
