use crate::{
    components::{EquipBonus, Equipped, MeleeAttack, Name, Stats, TakeDamage},
    gui::GameLog,
};
use specs::{prelude::*, rayon::iter::Either};

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
        ReadStorage<'a, EquipBonus>,
        ReadStorage<'a, Equipped>,
    );

    fn run(
        &mut self,
        (entities, mut melee, names, stats, mut damage, mut log, bonus, equipped): Self::SystemData,
    ) {
        for (attacker, melee, name, stat) in (&entities, &melee, &names, &stats).join() {
            if stat.hp > 0 {
                let target_stats = stats.get(melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(melee.target).unwrap();
                    let (target_equipped, attacker_equipped): (Vec<_>, Vec<_>) =
                        (&equipped, &bonus)
                            .par_join()
                            .filter(|(e, _)| e.owner == melee.target || e.owner == attacker)
                            .partition_map(|(e, b)| {
                                if e.owner == melee.target {
                                    Either::Left(b)
                                } else {
                                    Either::Right(b)
                                }
                            });
                    let effective_power =
                        attacker_equipped
                            .into_iter()
                            .fold(stat.base_power, |power, b| match b {
                                EquipBonus::Defense(_) => power,
                                EquipBonus::Attack(b) => power + b,
                            });
                    let effective_defense = target_equipped.into_iter().fold(
                        target_stats.base_defense,
                        |defense, b| match b {
                            EquipBonus::Defense(bonus) => defense + bonus,
                            EquipBonus::Attack(_) => defense,
                        },
                    );
                    let dmg_amount = i32::max(0, effective_power - effective_defense);

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
