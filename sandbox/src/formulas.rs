use crate::entities::{LegendaryItem, Player, PlayerAttr};
use crate::mods::{Mod, Mods};
use crate::{MultiplierData, Weather};

//todo: All of these are formulas accurate around season 18-20,
//however the sim in its current state operates with DE assumptions.
//This is fixed with season rulesets

pub fn strike_threshold(pitcher: &Player, batter: &Player, flinch: bool, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let fwd = 0.5; // todo: ballparks
    let ruth = coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let vibeless_musc = coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude);
                                                                                                                                                                                  
    let constant = if flinch { 0.4 } else { 0.2 };
    match season_ruleset {
        11 => (constant + 0.35 * ruth + 0.2 * fwd + 0.1 * vibeless_musc).min(0.9),
        12 | 13 => (constant + 0.3 * ruth + 0.2 * fwd + 0.1 * vibeless_musc).min(0.85),  
        14 => (constant + 0.285 * ruth + 0.2 * fwd + 0.1 * vibeless_musc).min(0.86),
        15..24 => todo!(),
        _ => panic!("wrong season number")
    }
}

pub fn swing_threshold(pitcher: &Player, batter: &Player, is_strike: bool, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let visc = 0.5;
    if is_strike {
        let combined_batting = (coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            + coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            + (1.0 - coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism)) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability)) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            / 4.0;
        let ruth = coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
        let batter_hype = 0.0;
        let pitcher_hype = 0.0;
        match season_ruleset {
            //todo: difference between resim formula and resim notebook
            11..18 => 0.7 + 0.35 * combined_batting - 0.4 * ruth + 0.2 * (visc - 0.5),
            18 => 0.6 + 0.35 * combined_batting - 0.2 * ruth + 0.2 * (visc - 0.5),
            19..23 => 0.6 + 0.35 * combined_batting + 0.04 * batter_hype - 0.2 * ruth - 0.03125 * pitcher_hype + 0.2 * (visc - 0.5), //todo: incomplete formula
            _ => 0.0
        }
    } else {
        let combined = if season_ruleset < 18 {
            (12.0 * coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day))
            - 5.0 * coeff(PlayerAttr::Moxie, &batter.legendary_item, &batter.mods, multiplier_data, batter.moxie) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            + 5.0 * coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism)
            + 4.0 * visc)
            / 20.0
        } else {
            0.375 * (coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day))).powf(0.25)
            + 0.2 * visc
            - 0.25 * coeff(PlayerAttr::Moxie, &batter.legendary_item, &batter.mods, multiplier_data, batter.moxie) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
            + 0.25 * coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism) //todo: incomplete formula
        };
        (combined.powf(1.5)).min(0.95).max(0.1)
    }
}

pub fn contact_threshold(pitcher: &Player, batter: &Player, is_strike: bool, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let fort = 0.5 - 0.5;
    let visc = 0.5 - 0.5;
    let fwd = 0.5 - 0.5;

    let ruth = coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));

    let ballpark_sum = (fort + 3.0 * visc - 6.0 * fwd) / 10.0;

    if is_strike {
        let combined_batting =
            (coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity)
            + coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude)
            + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability)
            - coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism))
            / 2.0
            * (1.0 + 0.2 * batter.vibes(multiplier_data.day));

        match season_ruleset {
            11..14 => (0.8 - 0.08 * ruth + 0.16 * ballpark_sum + 0.16 * combined_batting.powf(1.2)).min(0.9),
            14 => (0.78 - 0.08 * ruth + 0.16 * ballpark_sum + 0.17 * combined_batting.powf(1.2)).min(0.925),
            15..23 => todo!(), //"we don't know" - Astrid
            _ => 0.0
        }
    } else {
        let inv_path = 1.0 - batter.patheticism * multiplier(PlayerAttr::Patheticism, &batter.mods, multiplier_data) * (1.0 + 0.2 * batter.vibes(multiplier_data.day));
        match season_ruleset {
            11..14 => (0.35 - 0.1 * ruth + 0.4 * inv_path.max(0.0).powf(1.5) + 0.14 * ballpark_sum).min(1.0),
            14 => (0.4 - 0.1 * ruth + 0.35 * inv_path.max(0.0).powf(1.5) + 0.14 * ballpark_sum).min(1.0),
            15..23 => todo!(),
            _ => 0.0
        }
    }
}

pub fn foul_threshold(_pitcher: &Player, batter: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let fwd = 0.5;
    let obt = 0.5;
    let batter_sum = (coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
        + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
        + coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity)) * (1.0 + 0.2 * batter.vibes(multiplier_data.day))
        / 3.0;
    0.25 + 0.1 * fwd - 0.1 * obt + 0.1 * batter_sum //consistent across all seasons
}

pub fn out_threshold(pitcher: &Player, batter: &Player, defender: &Player, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let grand_center = 0.0;
    let obt_center = 0.0;
    let omi_center = 0.0;
    let incon_center = 0.0;
    let visc_center = 0.0;
    let fwd_center = 0.0;

    let thwack = coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability) * (1.0 + 0.2 * batter.vibes(multiplier_data.day)); // all with vibes
    let unthwack = coeff(PlayerAttr::Unthwackability, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.unthwackability) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let omni = coeff(PlayerAttr::Omniscience, &defender.legendary_item, &defender.mods, multiplier_data, defender.omniscience) * (1.0 + 0.2 * defender.vibes(multiplier_data.day));

    match season_ruleset { 
        11 | 12 => {
            0.315 + 0.1 * thwack - 0.08 * unthwack - 0.07 * omni
            + 0.0145 * grand_center
            + 0.0085 * omi_center
            - 0.011 * incon_center
            - 0.005 * visc_center
            + 0.01 * fwd_center
        },
        13 => {
            0.3115 + 0.1 * thwack - 0.08 * unthwack - 0.065 * omni
            + 0.01 * grand_center
            + 0.0085 * obt_center
            - 0.0033 * omi_center
            - 0.002 * incon_center
            - 0.0033 * visc_center
            + 0.01 * fwd_center
        },
        14..18 => {
            let bp_sum = (55.0 * grand_center
                + 51.0 * fwd_center
                + 40.0 * obt_center
                - 17.0 * visc_center
                - 17.0 * omi_center
                - 10.0 * incon_center
            ) / 100.0;
            0.311 + 0.1 * thwack - 0.08 * unthwack - 0.064 * omni + 0.02 * bp_sum
        },
        18..24 => todo!(),
        _ => 0.0
    }
}

pub fn fly_threshold(batter: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let omi_center = 0.0;
    let buoy = coeff(PlayerAttr::Buoyancy, &batter.legendary_item, &batter.mods, multiplier_data, batter.buoyancy); //no vibes
    let supp = coeff(PlayerAttr::Suppression, &batter.legendary_item, &batter.mods, multiplier_data, batter.suppression); //this is tgb's doing; team should still be the pitching team

    //consistent across all seasons
    (0.18 + 0.3 * buoy - 0.16 * supp - 0.1 * omi_center).max(0.01) //todo: hype
}

pub fn hr_threshold(pitcher: &Player, batter: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let div = coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity) * (1.0 + 0.2 * batter.vibes(multiplier_data.day));
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let supp = coeff(PlayerAttr::Suppression, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.suppression) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));

    let grand_center = 0.0;
    let fort_center = 0.0;
    let visc_center = 0.0;
    let om_center = 0.0;
    let fwd_center = 0.0;

    let ballpark_sum =
        0.4 * grand_center + 0.2 * fort_center + 0.08 * visc_center + 0.08 * om_center
            - 0.24 * fwd_center;

    let opw_supp = (10.0 * opw + 1.0 * supp) / 11.0;

    0.12 + 0.16 * div - 0.08 * opw_supp - 0.18 * ballpark_sum //consistent across all seasons
}

pub fn quadruple_threshold(_pitcher: &Player, _batter: &Player, _fielder: &Player, _season_ruleset: u8, _multiplier_data: &MultiplierData) -> f64 {
    //todo
    0.015
}

pub fn triple_threshold(pitcher: &Player, batter: &Player, fielder: &Player, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let gf = coeff(PlayerAttr::GroundFriction, &batter.legendary_item, &batter.mods, multiplier_data, batter.ground_friction) * (1.0 + 0.2 * batter.vibes(multiplier_data.day));
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let chase = coeff(PlayerAttr::Chasiness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.chasiness) * (1.0 + 0.2 * fielder.vibes(multiplier_data.day));
    let fwd_center = 0.0;
    let grand_center = 0.0;
    let obt_center = 0.0;
    let omi_center = 0.0;
    let visc_center = 0.0;
    let bp_sum = (3.0 * fwd_center + 5.0 * grand_center + 5.0 * obt_center - omi_center - visc_center) / 15.0;

    match season_ruleset {
        11 | 12 => 0.05 + 0.2 * gf - 0.04 * opw - 0.06 * chase + 0.1 * bp_sum,
        13..18 => 0.045 + 0.2 * gf - 0.04 * opw - 0.05 * chase + 0.1 * bp_sum,
        18..24 => todo!(),
        _ => 0.0
    }
}

pub fn double_threshold(pitcher: &Player, batter: &Player, fielder: &Player, season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let musc = coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude) * (1.0 + 0.2 * batter.vibes(multiplier_data.day));
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let chase = coeff(PlayerAttr::Chasiness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.chasiness) * (1.0 + 0.2 * fielder.vibes(multiplier_data.day));
    let fwd_center = 0.0;
    let elong_center = 0.0;
    let omi_center = 0.0;
    let visc_center = 0.0;
    let bp_sum = 0.027 * fwd_center - 0.015 * elong_center - 0.01 * omi_center - 0.008 * visc_center;

    match season_ruleset {
        11 | 12 => 0.17 + 0.2 * musc - 0.04 * opw - 0.1 * chase + bp_sum,
        13 => 0.165 + 0.2 * musc - 0.04 * opw - 0.09 * chase + bp_sum,
        14..18 => 0.16 + 0.2 * musc - 0.04 * opw - 0.08 * chase + bp_sum,
        18..24 => todo!(),
        _ => 0.0
    }
}

pub fn steal_attempt_threshold(_runner: &Player, _defender: &Player) -> f64 {
    // todo: lol
    0.05
}

pub fn steal_success_threshold(_runner: &Player, _defender: &Player) -> f64 {
    0.8
}

//all out formulas are consistent across all seasons. probably

pub fn hit_advancement_threshold(runner: &Player, fielder: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let tenac = coeff(PlayerAttr::Tenaciousness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness); //no vibes???
    let cont = coeff(PlayerAttr::Continuation, &runner.legendary_item, &runner.mods, multiplier_data, runner.continuation);

    (0.7 - tenac + 0.6 * cont).min(0.95).max(0.01)
}

pub fn groundout_sacrifice_threshold(batter: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let mart = coeff(PlayerAttr::Martyrdom, &batter.legendary_item, &batter.mods, multiplier_data, batter.martyrdom);

    0.05 + 0.25 * mart
}

pub fn groundout_advancement_threshold(runner: &Player, fielder: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let indulg = coeff(PlayerAttr::Indulgence, &runner.legendary_item, &runner.mods, multiplier_data, runner.indulgence) * (1.0 + 0.2 * runner.vibes(multiplier_data.day));
    let tenac = coeff(PlayerAttr::Tenaciousness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness) * (1.0 + 0.2 * fielder.vibes(multiplier_data.day));
    let incon = 0.5;
    let elong = 0.5;

    0.5 + 0.35 * indulg - 0.15 * tenac - 0.15 * (incon - 0.5) - 0.15 * (elong - 0.5) //todo: batter debt cursedness
}

pub fn double_play_threshold(batter: &Player, pitcher: &Player, fielder: &Player, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let shakes = coeff(PlayerAttr::Shakespearianism, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.shakespearianism) * (1.0 + 0.2 * pitcher.vibes(multiplier_data.day));
    let trag = coeff(PlayerAttr::Tragicness, &batter.legendary_item, &batter.mods, multiplier_data, batter.tragicness);
    let tenac = coeff(PlayerAttr::Tenaciousness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness) * (1.0 + 0.2 * fielder.vibes(multiplier_data.day));
    let elong = 0.5;

    (-0.05 + 0.4 * shakes - 0.18 * (1.0 - trag) + 0.1 * tenac - 0.16 * (elong - 0.5)).max(0.001)
}

pub fn flyout_advancement_threshold(runner: &Player, base_from: u8, _season_ruleset: u8, multiplier_data: &MultiplierData) -> f64 {
    let indulg = coeff(PlayerAttr::Indulgence, &runner.legendary_item, &runner.mods, multiplier_data, runner.indulgence) * (1.0 + 0.2 * runner.vibes(multiplier_data.day));
    let elong = 0.0;
    let incon = 0.0;
    match base_from {
        0 => {
            let indulg_factor = 0.36 * indulg - 0.38 * indulg.powf(2.0) + 0.24 * indulg.powf(4.0);
            return -0.085 + indulg_factor - 0.1 * elong - 0.1 * incon;
        },
        1 => {
            let indulg_factor = 0.065 * indulg + 0.3 * indulg.powf(2.0);
            return 0.045 + indulg_factor - 0.1 * elong - 0.1 * incon;
        },
        //todo: find fifth base flyout advancement; uhhh
        2 | 3 => {
            return 0.45 + 0.35 * indulg - 0.1 * elong - 0.1 * incon;
        }
        _ => {
            return 0.0; //lol
        }
    }
}

fn coeff(attr: PlayerAttr, legendary_item: &Option<LegendaryItem>, mods: &Mods, multiplier_data: &MultiplierData, stat: f64) -> f64 {
    let mut item_stat = stat + item(attr, legendary_item, multiplier_data);
    if attr.is_negative() {
        item_stat = item_stat.min(0.99);
    } else {
        item_stat = item_stat.max(0.01);
    }
    item_stat * multiplier(attr, mods, multiplier_data)
}

fn multiplier(attr: PlayerAttr, mods: &Mods, data: &MultiplierData) -> f64 {
    let mut multiplier = 1.0;
    if mods.has(Mod::Growth) {
        multiplier += 0.05f64.min(data.day as f64 / 99.0 * 0.05);
    } else if let Weather::Birds = data.weather {
        if mods.has(Mod::AffinityForCrows) && attr.is_pitching() {
            multiplier += 0.5;
        }
    } else if mods.has(Mod::RedHot) {
        if let PlayerAttr::Thwackability = attr {
            multiplier += 4.0;
        } else if let PlayerAttr::Moxie = attr {
            multiplier += 2.0;
        }
    }
    if let Weather::Eclipse = data.weather {
        if mods.has(Mod::NightVision) && attr.is_batting() {
            multiplier += 0.5;
        }
    }
    if attr.is_negative() {
        1.0 / multiplier
    } else if let PlayerAttr::Buoyancy = attr { //unless vibes have multipliers I think it's safe
        1.0 / multiplier
    } else {
        multiplier
    }
}

fn item(attr: PlayerAttr, item: &Option<LegendaryItem>, _data: &MultiplierData) -> f64 {
    if let Some(item_type) = item {
        match item_type {
            LegendaryItem::DialTone | LegendaryItem::VibeCheck | LegendaryItem::BangersAndSmash => {
                if attr.is_negative() {
                    return -0.2;
                } else if attr.is_batting() {
                    return 0.2;
                }
            },
            LegendaryItem::LiteralArmCannon => {
                if attr.is_pitching() {
                    return 0.08;
                } else if attr.is_defense() {
                    return 0.23;
                }
            },
            LegendaryItem::GrapplingHook => {
                if attr.is_defense() || attr.is_running() {
                    return 0.6;
                }
            },
            LegendaryItem::Mushroom => {
                match attr {
                    PlayerAttr::Divinity | PlayerAttr::Musclitude => {
                        return 0.6;
                    },
                    PlayerAttr::Cinnamon => {
                        return 0.4;
                    },
                    PlayerAttr::BaseThirst | PlayerAttr::Continuation | PlayerAttr::Indulgence | PlayerAttr::Laserlikeness => {
                        return -0.4;
                    },
                    PlayerAttr::GroundFriction => {
                        return -0.1;
                    },
                    _ => {
                        return 0.0;
                    }
                }
            },
            LegendaryItem::NightVisionGoggles => {
                return 0.0;
            },
            LegendaryItem::ShrinkRay => {
                match attr {
                    PlayerAttr::Moxie => {
                        return 0.1;
                    },
                    PlayerAttr::BaseThirst | PlayerAttr::Continuation | PlayerAttr::Indulgence | PlayerAttr::Laserlikeness => {
                        return 0.2;
                    },
                    PlayerAttr::GroundFriction => {
                        return 0.175;
                    },
                    PlayerAttr::Musclitude => {
                        return -0.07;
                    },
                    PlayerAttr::Divinity => {
                        return -0.05;
                    },
                    _ => {
                        return 0.0;
                    }
                }
            },
            LegendaryItem::TheIffeyJr => {
                if attr.is_negative() {
                    return 0.51;
                } else if attr.is_batting() || attr.is_running() {
                    return -0.51;
                }
            }
        }
    }
    0.0
}
