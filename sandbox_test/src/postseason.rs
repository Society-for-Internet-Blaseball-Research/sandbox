use sandbox::{entities::World, rng::Rng, Game};
use crate::schedule::generate_game;
use uuid::Uuid;
use std::cmp::Ordering;

pub fn generate_seeding(divisions: &Vec<Uuid>, standings: &Vec<i16>, fates: &Vec<usize>, world: &World, rng: &mut Rng) -> (Vec<Uuid>, Vec<Uuid>) {
    //indices of teams in the division Vec
    let mut indices: Vec<usize> = (0..20).collect();
    
    indices.sort_by(|&a, &b| {
        if let Ordering::Equal = standings[b].cmp(&standings[a]) {
            fates[a].cmp(&fates[b])
        } else {
            standings[b].cmp(&standings[a])
        }
    });
    
    //how many playoff teams are in each division
    let mut division_playoffs: [u8; 4] = [0; 4];
    let mut playoff_seeds1: Vec<Uuid> = Vec::new();
    let mut playoff_seeds2: Vec<Uuid> = Vec::new();
    let mut indices_wc: Vec<usize> = indices.clone();
    for &idx in indices.iter() {
        if idx < 10 {
            if playoff_seeds1.len() < 4 {
                playoff_seeds1.push(divisions[idx]);
                division_playoffs[idx / 5] += 1; //idx / 5 is the index of the division
                indices_wc.retain(|&i| i != idx);
            }
        } else {
            if playoff_seeds2.len() < 4 {
                playoff_seeds2.push(divisions[idx]);
                division_playoffs[idx / 5] += 1;
                indices_wc.retain(|&i| i != idx);
            }
        }

        for div in 0..4 {
            let oppo = if div % 2 == 0 { div + 1 } else { div - 1 }; //the other division in the league
            if division_playoffs[div] == 0 && division_playoffs[oppo] == 3 {
                let div_winner_idx = *(indices.iter().find(|&&i| i >= div * 5 && i < (div + 1) * 5).unwrap());
                if div < 2 {
                    playoff_seeds1.push(divisions[div_winner_idx]);
                } else {
                    playoff_seeds2.push(divisions[div_winner_idx]);
                }
                division_playoffs[div] += 1;
                indices_wc.retain(|&i| i != div_winner_idx);
            }
        }
        if division_playoffs[0] + division_playoffs[1] + division_playoffs[2] + division_playoffs[3] == 8 {
            break;
        }
    }
    let indices_wc1: Vec<usize> = indices_wc.iter().copied().filter(|&i| i < 10).collect();
    let indices_wc2: Vec<usize> = indices_wc.iter().copied().filter(|&i| i >= 10).collect();
    playoff_seeds1.push(divisions[indices_wc1[rng.index(6)]]);
    playoff_seeds2.push(divisions[indices_wc2[rng.index(6)]]);
    (playoff_seeds1, playoff_seeds2)
}

pub fn generate_wildcard(playoff_seeds1: &Vec<Uuid>, playoff_seeds2: &Vec<Uuid>, round: usize, world: &World, rng: &mut Rng) -> Vec<Game> {
    let mut games_active: Vec<Game> = Vec::new();
    let higher_seed_hosts = round % 2 == 0;
    let wins_1_4 = world.team(playoff_seeds1[3]).postseason_wins;
    let wins_1_5 = world.team(playoff_seeds1[4]).postseason_wins;
    let wins_2_4 = world.team(playoff_seeds2[3]).postseason_wins;
    let wins_2_5 = world.team(playoff_seeds2[4]).postseason_wins;
    if wins_1_4 < 2 && wins_1_5 < 2 || wins_1_4 == wins_1_5 {
        games_active.push(generate_game(
            playoff_seeds1[if higher_seed_hosts { 3 } else { 4 }],
            playoff_seeds1[if higher_seed_hosts { 4 } else { 3 }],
            99 + round as usize,
                rng,
                world
            ));
        }
    if wins_2_4 < 2 && wins_2_5 < 2 || wins_2_4 == wins_2_5 {
        games_active.push(generate_game(
            playoff_seeds2[if higher_seed_hosts { 3 } else { 4 }],
            playoff_seeds2[if higher_seed_hosts { 4 } else { 3 }],
            99 + round as usize,
            rng,
            world
        ));
    }
    games_active
}

pub fn generate_divisional(playoff_seeds1: &Vec<Uuid>, playoff_seeds2: &Vec<Uuid>, round: usize, world: &World, rng: &mut Rng) -> Vec<Game> {
    let mut games_active: Vec<Game> = Vec::new();
    let higher_seed_hosts = round % 2 == 0;
    let mut wins_1 = [0; 4];
    for j in 0..4 {
        wins_1[j] = world.team(playoff_seeds1[j]).postseason_wins;
    }
    let mut wins_2 = [0; 4];
    for j in 0..4 {
        wins_2[j] = world.team(playoff_seeds2[j]).postseason_wins;
    }
    if wins_1[0] < 3 && wins_1[3] < 3 || wins_1[0] == wins_1[3] {
        games_active.push(generate_game(
            playoff_seeds1[if higher_seed_hosts { 0 } else { 3 }],
            playoff_seeds1[if higher_seed_hosts { 3 } else { 0 }],
            102 + round as usize,
            rng,
            world
        ));
    }
    if wins_1[1] < 3 && wins_1[2] < 3 || wins_1[1] == wins_1[2] {
        games_active.push(generate_game(
            playoff_seeds1[if higher_seed_hosts { 1 } else { 2 }],
            playoff_seeds1[if higher_seed_hosts { 2 } else { 1 }],
            102 + round as usize,
            rng,
            world
        ));
    }
    if wins_2[0] < 3 && wins_2[3] < 3 || wins_2[0] == wins_2[3] {
        games_active.push(generate_game(
            playoff_seeds2[if higher_seed_hosts { 0 } else { 3 }],
            playoff_seeds2[if higher_seed_hosts { 3 } else { 0 }],
            102 + round as usize,
            rng,
            world
        ));
    }
    if wins_2[1] < 3 && wins_2[2] < 3 || wins_2[1] == wins_2[2] {
        games_active.push(generate_game(
            playoff_seeds2[if higher_seed_hosts { 1 } else { 2 }],
            playoff_seeds2[if higher_seed_hosts { 2 } else { 1 }],
            102 + round as usize,
            rng,
            world
        ));
    }
    games_active
}

pub fn generate_championship(playoff_seeds1: &Vec<Uuid>, playoff_seeds2: &Vec<Uuid>, round: usize, world: &World, rng: &mut Rng) -> Vec<Game> {
    let mut games_active: Vec<Game> = Vec::new();
    let higher_seed_hosts = round % 2 == 0;
    let wins_1_1 = world.team(playoff_seeds1[0]).postseason_wins;
    let wins_1_2 = world.team(playoff_seeds1[1]).postseason_wins;
    let wins_2_1 = world.team(playoff_seeds2[0]).postseason_wins;
    let wins_2_2 = world.team(playoff_seeds2[1]).postseason_wins;
    if wins_1_1 < 3 && wins_1_2 < 3 || wins_1_1 == wins_1_2 {
        games_active.push(generate_game(
            playoff_seeds1[if higher_seed_hosts { 0 } else { 1 }],
            playoff_seeds1[if higher_seed_hosts { 1 } else { 0 }],
            107 + round as usize,
            rng,
            world
        ));
    }
    if wins_2_1 < 3 && wins_2_2 < 3 || wins_2_1 == wins_2_2 {
        games_active.push(generate_game(
            playoff_seeds2[if higher_seed_hosts { 0 } else { 1 }],
            playoff_seeds2[if higher_seed_hosts { 1 } else { 0 }],
            107 + round as usize,
            rng,
            world
        ));
    }
    games_active
}
