use sandbox::{entities::World, rng::Rng, Game};
use uuid::Uuid;
use std::cmp::Ordering;

pub fn generate_seeding(divisions: &Vec<Uuid>, standings: &Vec<i16>, fates: &Vec<usize>, rng: &mut Rng) -> (Vec<Uuid>, Vec<Uuid>) {
    let league_size = divisions.len();
    let div_size = league_size / 4;
    let subleague_size = league_size / 2;
    //indices of teams in the division Vec
    let mut indices: Vec<usize> = (0..league_size).collect();
    
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
        if idx < subleague_size {
            if playoff_seeds1.len() < 4 {
                playoff_seeds1.push(divisions[idx]);
                division_playoffs[idx / div_size] += 1; //idx / 5 is the index of the division
                indices_wc.retain(|&i| i != idx);
            }
        } else {
            if playoff_seeds2.len() < 4 {
                playoff_seeds2.push(divisions[idx]);
                division_playoffs[idx / div_size] += 1;
                indices_wc.retain(|&i| i != idx);
            }
        }

        for div in 0..4 {
            let oppo = if div % 2 == 0 { div + 1 } else { div - 1 }; //the other division in the league
            if division_playoffs[div] == 0 && division_playoffs[oppo] == 3 {
                let div_winner_idx = *(indices.iter().find(|&&i| i >= div * div_size && i < (div + 1) * div_size).unwrap());
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
    let indices_wc1: Vec<usize> = indices_wc.iter().copied().filter(|&i| i < subleague_size).collect();
    let indices_wc2: Vec<usize> = indices_wc.iter().copied().filter(|&i| i >= subleague_size).collect();
    playoff_seeds1.push(divisions[indices_wc1[rng.index(subleague_size - 4)]]);
    playoff_seeds2.push(divisions[indices_wc2[rng.index(subleague_size - 4)]]);
    (playoff_seeds1, playoff_seeds2)
}

pub fn update_party(divisions: &Vec<Uuid>, standings: &Vec<i16>, fates: &Vec<usize>, day: usize, world: &mut World, rng: &mut Rng) {
    let league_size = divisions.len();
    let div_size = league_size / 4;
    let subleague_size = league_size / 2;
    //indices of teams in the division Vec
    let mut indices: Vec<usize> = (0..league_size).collect();
    
    indices.sort_by(|&a, &b| {
        if let Ordering::Equal = standings[a].cmp(&standings[b]) {
            fates[a].cmp(&fates[b])
        } else {
            standings[a].cmp(&standings[b])
        }
    });
    
    //how many playoff teams are in each division
    let mut division_playoffs: [u8; 4] = [0; 4];
    let mut playoff_seeds1: Vec<Uuid> = Vec::new();
    let mut playoff_seeds2: Vec<Uuid> = Vec::new();
    for &idx in indices.iter() {
        if idx < subleague_size {
            if playoff_seeds1.len() < 4 {
                playoff_seeds1.push(divisions[idx]);
                division_playoffs[idx / div_size] += 1; //idx / 5 is the index of the division
            }
        } else {
            if playoff_seeds2.len() < 4 {
                playoff_seeds2.push(divisions[idx]);
                division_playoffs[idx / div_size] += 1;
            }
        }

        for div in 0..4 {
            let oppo = if div % 2 == 0 { div + 1 } else { div - 1 }; //the other division in the league
            if division_playoffs[div] == 0 && division_playoffs[oppo] == 3 {
                let div_winner_idx = *(indices.iter().find(|&&i| i >= div * div_size && i < (div + 1) * div_size).unwrap());
                if div < 2 {
                    playoff_seeds1.push(divisions[div_winner_idx]);
                } else {
                    playoff_seeds2.push(divisions[div_winner_idx]);
                }
                division_playoffs[div] += 1;
            }
        }
        if division_playoffs[0] + division_playoffs[1] + division_playoffs[2] + division_playoffs[3] == 8 {
            break;
        }
    }

    let mut losses1: Vec<i16> = playoff_seeds1
        .iter()
        .map(|&id| world.team(id).losses)
        .collect();
    losses1.sort();
    let max_losses1 = losses1.last().unwrap();
    
    let mut losses2: Vec<i16> = playoff_seeds2
        .iter()
        .map(|&id| world.team(id).losses)
        .collect();
    losses2.sort();
    let max_losses2 = losses2.last().unwrap();

    for i in 0..league_size {
        let team = world.team_mut(divisions[i]);
        //todo: this doesn't include fate...
        let max_losses = if i < subleague_size { max_losses1 } else { max_losses2 }; 
        if (team.losses - max_losses) as i128 > (99 - day) as i128 && !team.partying { 
            //todo: i128???
            team.partying = true;
            println!("Partytime: day {}, {}, {} losses, {} max playoff losses", day, team.name, team.losses, max_losses);
        }
    }   
}

pub fn generate_wildcard(playoff_seeds1: &Vec<Uuid>, playoff_seeds2: &Vec<Uuid>, round: usize, world: &World, rng: &mut Rng) -> Vec<Game> {
    let mut games_active: Vec<Game> = Vec::new();
    let higher_seed_hosts = round % 2 == 0;
    let wins_1_4 = world.team(playoff_seeds1[3]).postseason_wins;
    let wins_1_5 = world.team(playoff_seeds1[4]).postseason_wins;
    let wins_2_4 = world.team(playoff_seeds2[3]).postseason_wins;
    let wins_2_5 = world.team(playoff_seeds2[4]).postseason_wins;
    if wins_1_4 < 2 && wins_1_5 < 2 || wins_1_4 == wins_1_5 {
        games_active.push(Game::new(
            playoff_seeds1[if higher_seed_hosts { 3 } else { 4 }],
            playoff_seeds1[if higher_seed_hosts { 4 } else { 3 }],
            99 + round as usize,
            None,
            world,
            rng
        ));
    }
    if wins_2_4 < 2 && wins_2_5 < 2 || wins_2_4 == wins_2_5 {
        games_active.push(Game::new(
            playoff_seeds2[if higher_seed_hosts { 3 } else { 4 }],
            playoff_seeds2[if higher_seed_hosts { 4 } else { 3 }],
            99 + round as usize,
            None,
            world,
            rng
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
        games_active.push(Game::new(
            playoff_seeds1[if higher_seed_hosts { 0 } else { 3 }],
            playoff_seeds1[if higher_seed_hosts { 3 } else { 0 }],
            102 + round as usize,
            None,
            world,
            rng
        ));
    }
    if wins_1[1] < 3 && wins_1[2] < 3 || wins_1[1] == wins_1[2] {
        games_active.push(Game::new(
            playoff_seeds1[if higher_seed_hosts { 1 } else { 2 }],
            playoff_seeds1[if higher_seed_hosts { 2 } else { 1 }],
            102 + round as usize,
            None,
            world,
            rng
        ));
    }
    if wins_2[0] < 3 && wins_2[3] < 3 || wins_2[0] == wins_2[3] {
        games_active.push(Game::new(
            playoff_seeds2[if higher_seed_hosts { 0 } else { 3 }],
            playoff_seeds2[if higher_seed_hosts { 3 } else { 0 }],
            102 + round as usize,
            None,
            world,
            rng
        ));
    }
    if wins_2[1] < 3 && wins_2[2] < 3 || wins_2[1] == wins_2[2] {
        games_active.push(Game::new(
            playoff_seeds2[if higher_seed_hosts { 1 } else { 2 }],
            playoff_seeds2[if higher_seed_hosts { 2 } else { 1 }],
            102 + round as usize,
            None,
            world,
            rng
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
        games_active.push(Game::new(
            playoff_seeds1[if higher_seed_hosts { 0 } else { 1 }],
            playoff_seeds1[if higher_seed_hosts { 1 } else { 0 }],
            107 + round as usize,
            None,
            world,
            rng
        ));
    }
    if wins_2_1 < 3 && wins_2_2 < 3 || wins_2_1 == wins_2_2 {
        games_active.push(Game::new(
            playoff_seeds2[if higher_seed_hosts { 0 } else { 1 }],
            playoff_seeds2[if higher_seed_hosts { 1 } else { 0 }],
            107 + round as usize,
            None,
            world,
            rng
        ));
    }
    games_active
}
