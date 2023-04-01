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

pub fn out_threshold(_pitcher: &Player, _batter: &Player, _defender: &Player) -> f64 {
    0.4
}

pub fn fly_threshold(_defender: &Player) -> f64 {
    0.5
}

pub fn hr_threshold(_pitcher: &Player, _batter: &Player) -> f64 {
    0.05
}

pub fn triple_threshold(_pitcher: &Player, _batter: &Player) -> f64 {
    0.2
}

pub fn double_threshold(_pitcher: &Player, _batter: &Player) -> f64 {
    0.3
}

pub fn steal_attempt_threshold(_runner: &Player, _defender: &Player) -> f64 {
    0.05
}

pub fn steal_success_threshold(_runner: &Player, _defender: &Player) -> f64 {
    0.8
}
