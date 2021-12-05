use crate::{
    components::{MeleeAttack, Name, Stats, TakeDamage},
    gui::GameLog,
};
use specs::prelude::*;

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, MeleeAttack>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, TakeDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, (entities, mut melee, names, stats, mut damage, mut log): Self::SystemData) {
        for (_, melee, name, stat) in (&entities, &melee, &names, &stats).join() {
            if stat.hp > 0 {
                let target_stats = stats.get(melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(melee.target).unwrap();

                    let dmg_amount = i32::max(0, stat.base_power - target_stats.defense);

                    if dmg_amount == 0 {
                        log.entry(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entry(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, dmg_amount
                        ));
                        TakeDamage::new_damage(&mut damage, melee.target, dmg_amount);
                    }
                }
            }
        }
        melee.clear()
    }
}

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, Stats>, WriteStorage<'a, TakeDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        (&mut stats, &damage)
            .par_join()
            .for_each(|(mut stats, damage)| {
                stats.hp -= damage.amount;
            });
        damage.clear();
    }
}
