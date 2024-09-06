use crate::entities::{LegendaryItem, Player, PlayerAttr};
use crate::mods::{Mod, Mods};
use crate::{MultiplierData, Weather};

pub fn strike_threshold(pitcher: &Player, batter: &Player, flinch: bool, multiplier_data: &MultiplierData) -> f64 {
    let fwd = 0.5; // todo: ballparks
    let ruth = coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness); // todo: vibes

    let constant = if flinch { 0.4 } else { 0.2 };
    (constant + 0.285 * ruth + 0.2 * fwd + 0.1 * coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude)).min(0.86)
}

pub fn swing_threshold(pitcher: &Player, batter: &Player, is_strike: bool, multiplier_data: &MultiplierData) -> f64 {
    let visc = 0.5;
    if is_strike {
        let combined_batting = (coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity)
            + coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude)
            + (1.0 - coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism))
            + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability))
            / 4.0;
        0.7 + 0.35 * combined_batting - 0.4 * coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness) + 0.2 * (visc - 0.5)
    } else {
        let combined = (12.0 * coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness)
            - 5.0 * coeff(PlayerAttr::Moxie, &batter.legendary_item, &batter.mods, multiplier_data, batter.moxie)
            + 5.0 * coeff(PlayerAttr::Ruthlessness, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism)
            + 4.0 * visc)
            / 20.0;
        (combined.powf(1.5)).min(0.95).max(0.1)
    }
}

pub fn contact_threshold(pitcher: &Player, batter: &Player, is_strike: bool, multiplier_data: &MultiplierData) -> f64 {
    let fort = 0.5 - 0.5;
    let visc = 0.5 - 0.5;
    let fwd = 0.5 - 0.5;

    let ruth = coeff(PlayerAttr::Ruthlessness, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.ruthlessness);

    let ballpark_sum = (fort + 3.0 * visc - 6.0 * fwd) / 10.0;

    if is_strike {
        let combined_batting =
            (coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity)
            + coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude)
            + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability)
            - coeff(PlayerAttr::Patheticism, &batter.legendary_item, &batter.mods, multiplier_data, batter.patheticism)) / 2.0;

        (0.78 - 0.08 * ruth + 0.16 * ballpark_sum + 0.17 * combined_batting.powf(1.2)).min(0.925)
    } else {
        (0.4 - 0.1 * ruth + 0.35 * ((1.0 - batter.patheticism * multiplier(PlayerAttr::Patheticism, &batter.mods, multiplier_data)).powf(1.5)) + 0.14 * ballpark_sum)
            .min(1.0)
    }
}

pub fn foul_threshold(_pitcher: &Player, batter: &Player, multiplier_data: &MultiplierData) -> f64 {
    let fwd = 0.5;
    let obt = 0.5;
    let batter_sum = (coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude)
        + coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability)
        + coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity))
        / 3.0;
    0.25 + 0.1 * fwd - 0.1 * obt + 0.1 * batter_sum
}

pub fn out_threshold(pitcher: &Player, batter: &Player, defender: &Player, multiplier_data: &MultiplierData) -> f64 {
    let grand_center = 0.0;
    let obt_center = 0.0;
    let omi_center = 0.0;
    let incon_center = 0.0;
    let visc_center = 0.0;
    let fwd_center = 0.0;

    let thwack = coeff(PlayerAttr::Thwackability, &batter.legendary_item, &batter.mods, multiplier_data, batter.thwackability); // with vibes
    let unthwack = coeff(PlayerAttr::Unthwackability, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.unthwackability); // with vibes
    let omni = coeff(PlayerAttr::Omniscience, &defender.legendary_item, &defender.mods, multiplier_data, defender.omniscience); // with vibes

    0.3115 + 0.1 * thwack - 0.08 * unthwack - 0.065 * omni
        + 0.01 * grand_center
        + 0.0085 * obt_center
        - 0.0033 * omi_center
        - 0.0015 * incon_center
        - 0.0033 * visc_center
        + 0.01 * fwd_center
}

pub fn fly_threshold(batter: &Player, multiplier_data: &MultiplierData) -> f64 {
    let omi_center = 0.0;
    let buoy = coeff(PlayerAttr::Buoyancy, &batter.legendary_item, &batter.mods, multiplier_data, batter.buoyancy);
    let supp = coeff(PlayerAttr::Suppression, &batter.legendary_item, &batter.mods, multiplier_data, batter.suppression); //this is tgb's doing; team should still be the pitching team

    0.18 + 0.3 * buoy - 0.16 * supp - 0.1 * omi_center
}

pub fn hr_threshold(pitcher: &Player, batter: &Player, multiplier_data: &MultiplierData) -> f64 {
    let div = coeff(PlayerAttr::Divinity, &batter.legendary_item, &batter.mods, multiplier_data, batter.divinity);
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment);
    let supp = coeff(PlayerAttr::Suppression, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.suppression);

    let grand_center = 0.0;
    let fort_center = 0.0;
    let visc_center = 0.0;
    let om_center = 0.0;
    let fwd_center = 0.0;

    let ballpark_sum =
        0.4 * grand_center + 0.2 * fort_center + 0.08 * visc_center + 0.08 * om_center
            - 0.24 * fwd_center;

    let opw_supp = (10.0 * opw + 1.0 * supp) / 11.0;

    0.12 + 0.16 * div - 0.08 * opw_supp - 0.18 * ballpark_sum
}

pub fn quadruple_threshold(_pitcher: &Player, _batter: &Player, _fielder: &Player, _multiplier_data: &MultiplierData) -> f64 {
    //todo
    0.015
}

pub fn triple_threshold(pitcher: &Player, batter: &Player, fielder: &Player, multiplier_data: &MultiplierData) -> f64 {
    let gf = coeff(PlayerAttr::GroundFriction, &batter.legendary_item, &batter.mods, multiplier_data, batter.ground_friction);
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment);
    let chase = coeff(PlayerAttr::Chasiness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.chasiness);
    let fwd_center = 0.0;
    let grand_center = 0.0;
    let obt_center = 0.0;
    let omi_center = 0.0;
    let visc_center = 0.0;

    0.045 + 0.2 * gf - 0.04 * opw - 0.05 * chase
        + 0.02 * fwd_center
        + 0.034 * grand_center
        + 0.034 * obt_center
        - 0.0065 * omi_center
        - 0.0065 * visc_center
}

pub fn double_threshold(pitcher: &Player, batter: &Player, fielder: &Player, multiplier_data: &MultiplierData) -> f64 {
    let musc = coeff(PlayerAttr::Musclitude, &batter.legendary_item, &batter.mods, multiplier_data, batter.musclitude);
    let opw = coeff(PlayerAttr::Overpowerment, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.overpowerment);
    let chase = coeff(PlayerAttr::Chasiness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.chasiness);
    let fwd_center = 0.0;
    let elong_center = 0.0;
    let omi_center = 0.0;
    let visc_center = 0.0;

    0.165 + 0.2 * musc - 0.04 * opw - 0.009 * chase + 0.027 * fwd_center
        - 0.015 * elong_center
        - 0.01 * omi_center
        - 0.008 * visc_center
}

pub fn steal_attempt_threshold(_runner: &Player, _defender: &Player) -> f64 {
    // todo: lol
    0.05
}

pub fn steal_success_threshold(_runner: &Player, _defender: &Player) -> f64 {
    0.8
}

pub fn hit_advancement_threshold(runner: &Player, fielder: &Player, multiplier_data: &MultiplierData) -> f64 {
    let tenac = coeff(PlayerAttr::Tenaciousness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness);
    let cont = coeff(PlayerAttr::Continuation, &runner.legendary_item, &runner.mods, multiplier_data, runner.continuation);

    (0.7 - tenac + 0.6 * cont).min(0.95).max(0.01)
}

pub fn groundout_sacrifice_threshold(batter: &Player, multiplier_data: &MultiplierData) -> f64 {
    let mart = coeff(PlayerAttr::Martyrdom, &batter.legendary_item, &batter.mods, multiplier_data, batter.martyrdom);

    0.05 + 0.25 * mart
}

pub fn groundout_advancement_threshold(runner: &Player, fielder: &Player, multiplier_data: &MultiplierData) -> f64 {
    let indulg = coeff(PlayerAttr::Indulgence, &runner.legendary_item, &runner.mods, multiplier_data, runner.indulgence);
    let tenac = coeff(PlayerAttr::Tenaciousness, &fielder.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness);
    let incon = 0.5;
    let elong = 0.5;

    0.5 + 0.35 * indulg - 0.15 * tenac - 0.15 * (incon - 0.5) - 0.15 * (elong - 0.5)
}

pub fn double_play_threshold(batter: &Player, pitcher: &Player, fielder: &Player, multiplier_data: &MultiplierData) -> f64 {
    let shakes = coeff(PlayerAttr::Shakespearianism, &pitcher.legendary_item, &pitcher.mods, multiplier_data, pitcher.shakespearianism);
    let trag = coeff(PlayerAttr::Tragicness, &batter.legendary_item, &batter.mods, multiplier_data, batter.tragicness);
    let tenac = coeff(PlayerAttr::Tenaciousness, &batter.legendary_item, &fielder.mods, multiplier_data, fielder.tenaciousness);
    let elong = 0.5;

    (-0.05 + 0.4 * shakes - 0.18 * (1.0 - trag) + 0.1 * tenac - 0.16 * (elong - 0.5)).max(0.001)
}

pub fn flyout_advancement_threshold(runner: &Player, base_from: u8, multiplier_data: &MultiplierData) -> f64 {
    let indulg = coeff(PlayerAttr::Indulgence, &runner.legendary_item, &runner.mods, multiplier_data, runner.indulgence);
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
    (stat + item(attr, legendary_item, multiplier_data)).max(0.01) * multiplier(attr, mods, multiplier_data)
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
    if let PlayerAttr::Patheticism = attr {
        1.0 / multiplier
    } else if let PlayerAttr::Tragicness = attr {
        1.0 / multiplier
    } else {
        multiplier
    }
}

fn item(attr: PlayerAttr, item: &Option<LegendaryItem>, _data: &MultiplierData) -> f64 {
    if let Some(item_type) = item {
        match item_type {
            LegendaryItem::DialTone | LegendaryItem::VibeCheck | LegendaryItem::BangersAndSmash => {
                if let PlayerAttr::Patheticism = attr {
                    return -0.2;
                } else if let PlayerAttr::Tragicness = attr {
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
            }
        }
    }
    0.0
}
