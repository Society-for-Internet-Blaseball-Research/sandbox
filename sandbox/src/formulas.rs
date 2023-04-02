use crate::entities::Player;

pub fn strike_threshold(pitcher: &Player, batter: &Player, flinch: bool) -> f64 {
    let fwd = 0.5; // todo: ballparks
    let ruth = pitcher.ruthlessness; // todo: vibes

    let constant = if flinch { 0.4 } else { 0.2 };
    (constant + 0.285 * ruth + 0.2 * fwd + 0.1 * batter.musclitude).min(0.86)
}

pub fn swing_threshold(pitcher: &Player, batter: &Player, is_strike: bool) -> f64 {
    let visc = 0.5;
    if is_strike {
        let combined_batting = (batter.divinity
            + batter.musclitude
            + (1.0 - batter.patheticism)
            + batter.thwackability)
            / 4.0;
        0.7 + 0.35 * combined_batting - 0.4 * pitcher.ruthlessness + 0.2 * (visc - 0.5)
    } else {
        let combined = (12.0 * batter.ruthlessness - 5.0 * batter.moxie
            + 5.0 * batter.patheticism
            + 4.0 * visc)
            / 20.0;
        (combined.powf(1.5)).min(0.95).max(0.1)
    }
}

pub fn contact_threshold(pitcher: &Player, batter: &Player, is_strike: bool) -> f64 {
    let fort = 0.5 - 0.5;
    let visc = 0.5 - 0.5;
    let fwd = 0.5 - 0.5;

    let ruth = pitcher.ruthlessness;

    let ballpark_sum = (fort + 3.0 * visc - 6.0 * fwd) / 10.0;

    if is_strike {
        let combined_batting =
            (batter.divinity + batter.musclitude + batter.thwackability - batter.patheticism) / 2.0;

        (0.78 - 0.08 * ruth + 0.16 * ballpark_sum + 0.17 * combined_batting.powf(1.2)).min(0.925)
    } else {
        (0.4 - 0.1 * ruth + 0.35 * ((1.0 - batter.patheticism).powf(1.5)) + 0.14 * ballpark_sum)
            .min(1.0)
    }
}

pub fn foul_threshold(_pitcher: &Player, batter: &Player) -> f64 {
    let fwd = 0.5;
    let obt = 0.5;
    let batter_sum = (batter.musclitude + batter.thwackability + batter.divinity) / 3.0;
    0.25 + 0.1 * fwd - 0.1 * obt + 0.1 * batter_sum
}

pub fn out_threshold(pitcher: &Player, batter: &Player, defender: &Player) -> f64 {
    let grand_center = 0.0;
    let obt_center = 0.0;
    let omi_center = 0.0;
    let incon_center = 0.0;
    let visc_center = 0.0;
    let fwd_center = 0.0;

    let thwack = batter.thwackability; // with vibes
    let unthwack = pitcher.unthwackability; // with vibes
    let omni = defender.omniscience; // with vibes

    0.3115 + 0.1 * thwack - 0.08 * unthwack - 0.065 * omni
        + 0.01 * grand_center
        + 0.0085 * obt_center
        - 0.0033 * omi_center
        - 0.0015 * incon_center
        - 0.0033 * visc_center
        + 0.01 * fwd_center
}

pub fn fly_threshold(batter: &Player) -> f64 {
    let omi_center = 0.0;
    let buoy = batter.buoyancy;
    let supp = batter.suppression;

    0.18 + 0.3 * buoy - 0.16 * supp - 0.1 * omi_center
}

pub fn hr_threshold(pitcher: &Player, batter: &Player) -> f64 {
    let div = batter.divinity;
    let opw = pitcher.overpowerment;
    let supp = pitcher.suppression;

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

pub fn triple_threshold(pitcher: &Player, batter: &Player, fielder: &Player) -> f64 {
    let gf = batter.ground_friction;
    let opw = pitcher.overpowerment;
    let chase = fielder.chasiness;
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

pub fn double_threshold(pitcher: &Player, batter: &Player, fielder: &Player) -> f64 {
    let musc = batter.musclitude;
    let opw = pitcher.overpowerment;
    let chase = fielder.chasiness;
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
