use crate::schedule::{generate_game, generate_games, generate_schedule};
use sandbox::{
    entities::{LegendaryItem, NameGen, World},
    events::Event,
    rng::Rng,
    sim::Sim,
    mods::{Mod, ModLifetime},
    Game,
};
use std::cmp::Ordering;
use uuid::Uuid;

mod schedule;

fn main() {
    //edit seed
    let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    //let mut rng = Rng::new(37, 396396396396);
    //let mut rng = Rng::new(1923746321473263448, 2938897239474837483);

    let mut world = World::new();
    //let name_gen = NameGen::new();
    let mut teams: Vec<Uuid> = Vec::new();
    let team_names: Vec<&str> = include_str!("teams.txt").split(",").collect();
    let emojis: Vec<&str> = include_str!("emojis.txt").split(" ").collect();
    //todo: dehardcode team number
    for i in 0..20 {
        teams.push(world.gen_team(&mut rng, team_names[i].to_string(), emojis[i].to_string()));
    }
    //this is supposed to be editable so it's in human readable format
    //IMPORTANT: team names in teams.txt must be sorted alphabetically
    let divisions: Vec<Uuid> = 
        vec!["Baltimore Crabs", "Breckenridge Jazz Hands", "Chicago Firefighters", "Mexico City Wild Wings", "San Francisco Lovers",
        "Boston Flowers", "Hellmouth Sunbeams", "Houston Spies", "Miami Dale", "Unlimited Tacos",
        "Canada Moist Talkers", "Dallas Steaks", "Hades Tigers", "New York Millennials", "Seattle Garages", 
        "Charleston Shoe Thieves", "Hawai'i Fridays", "Kansas City Breath Mints", "Philly Pies", "Yellowstone Magic"]
            .iter()
            .map(|s| teams[team_names.binary_search(s).expect("team not found")])
            .collect();

    //edit mods and legendary items
    //world.team_mut(team_a).mods.add(Mod::FourthStrike, ModLifetime::Season);
    //world.player_mut(world.team(team_a).lineup[0]).add_legendary_item(LegendaryItem::TheIffeyJr);

    let mut fate_pool: Vec<usize> = (0..20).collect();
    for i in 0..20 {
        let fate_roll = fate_pool[rng.index(20 - i)];
        world.team_mut(teams[i]).fate = fate_roll;
        //println!("{} {}", world.team(teams[i]).name, world.team(teams[i]).fate);
        fate_pool.retain(|&j| j != fate_roll);
    }
    let days_in_season = 99;
    let games = generate_games(generate_schedule(days_in_season, &divisions, &mut rng), &world, &mut rng);
    let mut sim = Sim::new(&mut world, &mut rng);
    for day in 0..days_in_season {
        let mut games_active: Vec<Game> = Vec::new();
        for i in (day * 10)..((day + 1) * 10) {
            games_active.push(games[i].clone());
        }
        for i in 0..10 {
            let home_team = sim.world.team(games_active[i].home_team.id);
            let home_pitcher = home_team.rotation[day % home_team.rotation.len()];
            games_active[i].home_team.pitcher = if sim.world.player(home_pitcher).mods.has(Mod::Shelled) { home_team.rotation[(day - 1) % home_team.rotation.len()] } else { home_pitcher };
            let away_team = sim.world.team(games_active[i].away_team.id);
            let away_pitcher = away_team.rotation[day % away_team.rotation.len()];
            games_active[i].away_team.pitcher = if sim.world.player(away_pitcher).mods.has(Mod::Shelled) { away_team.rotation[(day - 1) % away_team.rotation.len()] } else { away_pitcher };
        }
        let mut games_deactivated: Vec<Uuid> = vec![];
        loop {
            //todo: store games through world
            for game in games_active.iter_mut() {
                //todo: make games a field of Sim
                let evt = sim.next(game);
                // keeping sim outside the loop means it borrows world and we can't pass it as mut here, which might be fine...?
                evt.apply(game, sim.world);
    
                if let Event::GameOver = evt {
                    /*println!(
                        "game over! {}: {}, {}: {}",
                        sim.world.team(game.away_team.id).name,
                        game.away_team.score,
                        sim.world.team(game.home_team.id).name,
                        game.home_team.score
                    );*/
                    games_deactivated.push(game.id);
                }
                /*let base = if game.runners.base_number == 5 {
                    format!(
                    "[{}|{}|{}|{}]",
                    if game.runners.occupied(3) { "X" } else { " " },
                    if game.runners.occupied(2) { "X" } else { " " },
                    if game.runners.occupied(1) { "X" } else { " " },
                    if game.runners.occupied(0) { "X" } else { " " }
                    )
                } else {
                    format!(
                    "[{}|{}|{}]",
                    if game.runners.occupied(2) { "X" } else { " " },
                    if game.runners.occupied(1) { "X" } else { " " },
                    if game.runners.occupied(0) { "X" } else { " " }
                    )
                };

                let away_score = (game.away_team.score * 10.0).round() / 10.0;
                let home_score = (game.home_team.score * 10.0).round() / 10.0; //floats

                println!(
                    "{}{} {}@{} ({}b/{}s/{}o) {} {:?}",
                    if game.top { "t" } else { "b" },
                    game.inning,
                    away_score,
                    home_score,
                    game.balls,
                    game.strikes,
                    game.outs,
                    base,
                    evt
                );*/
            }
            games_active.retain(|g| !games_deactivated.contains(&g.id));
            if games_active.len() == 0 {
                break;
            }
        }
    }

    //do standings and fates need to be a vec actually
    let mut standings: Vec<i16> = Vec::new();
    let mut fates: Vec<usize> = Vec::new();
    //indices of teams in the division Vec
    let mut indices: Vec<usize> = (0..20).collect();
    
    for &t in divisions.iter() {
        let team = sim.world.team(t);
        standings.push(team.wins);
        fates.push(team.fate);
        println!("{}: {}-{}", team.name, team.wins, team.losses);
    }
    
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
                division_playoffs[idx / 5] += 1;
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
            let oppo = if div % 2 == 0 { div + 1 } else { div - 1 };
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
    playoff_seeds1.push(divisions[indices_wc1[sim.rng.index(6)]]);
    playoff_seeds2.push(divisions[indices_wc2[sim.rng.index(6)]]);

    for team in playoff_seeds1.iter() {
        println!("{}", sim.world.team(*team).name);
    }
    for team in playoff_seeds2.iter() {
        println!("{}", sim.world.team(*team).name);
    }
    
    //let mut postseason_wins: [u8; 10] = [0; 10];
    //todo: make this a method with sim either as instance or parameter
    for i in 0..3 {
        let mut games_active: Vec<Game> = Vec::new();
        let mut games_deactivated: Vec<Uuid> = Vec::new();
        let higher_seed_hosts = i % 2 == 0;
        let wins_1_4 = sim.world.team(playoff_seeds1[3]).postseason_wins;
        let wins_1_5 = sim.world.team(playoff_seeds1[4]).postseason_wins;
        let wins_2_4 = sim.world.team(playoff_seeds2[3]).postseason_wins;
        let wins_2_5 = sim.world.team(playoff_seeds2[4]).postseason_wins;
        if wins_1_4 < 2 && wins_1_5 < 2 || wins_1_4 == wins_1_5 {
            games_active.push(generate_game(
                    playoff_seeds1[if higher_seed_hosts { 3 } else { 4 }],
                    playoff_seeds1[if higher_seed_hosts { 4 } else { 3 }],
                    99 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        if wins_2_4 < 2 && wins_2_5 < 2 || wins_2_4 == wins_2_5 {
            games_active.push(generate_game(
                    playoff_seeds2[if higher_seed_hosts { 3 } else { 4 }],
                    playoff_seeds2[if higher_seed_hosts { 4 } else { 3 }],
                    99 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        loop {
            for game in games_active.iter_mut() {
                let evt = sim.next(game);
                evt.apply(game, sim.world);
                if let Event::GameOver = evt {
                    games_deactivated.push(game.id);
                }
            }
            games_active.retain(|g| !games_deactivated.contains(&g.id));
            if games_active.len() == 0 {
                break;
            }
        }
    }

    println!("Wildcard: {} {}-{} {}", sim.world.team(playoff_seeds1[3]).name, sim.world.team(playoff_seeds1[3]).postseason_wins, sim.world.team(playoff_seeds1[4]).postseason_wins, sim.world.team(playoff_seeds1[4]).name);
    println!("Wildcard: {} {}-{} {}", sim.world.team(playoff_seeds2[3]).name, sim.world.team(playoff_seeds2[3]).postseason_wins, sim.world.team(playoff_seeds2[4]).postseason_wins, sim.world.team(playoff_seeds2[4]).name);

    playoff_seeds1.retain(|&t| {
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        losses < 2
    });
    playoff_seeds2.retain(|&t| {
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        losses < 2
    });

    for i in 0..5 {
        let mut games_active: Vec<Game> = Vec::new();
        let mut games_deactivated: Vec<Uuid> = Vec::new();
        let higher_seed_hosts = i % 2 == 0;
        let mut wins_1 = [0; 4];
        for j in 0..4 {
            wins_1[j] = sim.world.team(playoff_seeds1[j]).postseason_wins;
        }
        let mut wins_2 = [0; 4];
        for j in 0..4 {
            wins_2[j] = sim.world.team(playoff_seeds2[j]).postseason_wins;
        }
        if wins_1[0] < 3 && wins_1[3] < 3 || wins_1[0] == wins_1[3] {
            games_active.push(generate_game(
                    playoff_seeds1[if higher_seed_hosts { 0 } else { 3 }],
                    playoff_seeds1[if higher_seed_hosts { 3 } else { 0 }],
                    102 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        if wins_1[1] < 3 && wins_1[2] < 3 || wins_1[1] == wins_1[2] {
            games_active.push(generate_game(
                    playoff_seeds1[if higher_seed_hosts { 1 } else { 2 }],
                    playoff_seeds1[if higher_seed_hosts { 2 } else { 1 }],
                    102 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        if wins_2[0] < 3 && wins_2[3] < 3 || wins_2[0] == wins_2[3] {
            games_active.push(generate_game(
                    playoff_seeds2[if higher_seed_hosts { 0 } else { 3 }],
                    playoff_seeds2[if higher_seed_hosts { 3 } else { 0 }],
                    102 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        if wins_2[1] < 3 && wins_2[2] < 3 || wins_2[1] == wins_2[2] {
            games_active.push(generate_game(
                    playoff_seeds2[if higher_seed_hosts { 1 } else { 2 }],
                    playoff_seeds2[if higher_seed_hosts { 2 } else { 1 }],
                    102 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        loop {
            for game in games_active.iter_mut() {
                let evt = sim.next(game);
                evt.apply(game, sim.world);
                if let Event::GameOver = evt {
                    games_deactivated.push(game.id);
                }
            }
            games_active.retain(|g| !games_deactivated.contains(&g.id));
            if games_active.len() == 0 {
                break;
            }
        }
    }

    println!("Divisional: {} {}-{} {}", sim.world.team(playoff_seeds1[0]).name, sim.world.team(playoff_seeds1[0]).postseason_wins, sim.world.team(playoff_seeds1[3]).postseason_wins, sim.world.team(playoff_seeds1[3]).name);
    println!("Divisional: {} {}-{} {}", sim.world.team(playoff_seeds1[1]).name, sim.world.team(playoff_seeds1[1]).postseason_wins, sim.world.team(playoff_seeds1[2]).postseason_wins, sim.world.team(playoff_seeds1[2]).name);
    println!("Divisional: {} {}-{} {}", sim.world.team(playoff_seeds2[0]).name, sim.world.team(playoff_seeds2[0]).postseason_wins, sim.world.team(playoff_seeds2[3]).postseason_wins, sim.world.team(playoff_seeds2[3]).name);
    println!("Divisional: {} {}-{} {}", sim.world.team(playoff_seeds2[1]).name, sim.world.team(playoff_seeds2[1]).postseason_wins, sim.world.team(playoff_seeds2[2]).postseason_wins, sim.world.team(playoff_seeds2[2]).name);
    
    playoff_seeds1.retain(|&t| {
        let wins = sim.world.team(t).postseason_wins;
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        wins > losses
    });
    playoff_seeds2.retain(|&t| {
        let wins = sim.world.team(t).postseason_wins;
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        wins > losses
    });
    
    for i in 0..5 {
        let mut games_active: Vec<Game> = Vec::new();
        let mut games_deactivated: Vec<Uuid> = Vec::new();
        let higher_seed_hosts = i % 2 == 0;
        let mut wins_1 = [0; 2];
        for j in 0..2 {
            wins_1[j] = sim.world.team(playoff_seeds1[j]).postseason_wins;
        }
        let mut wins_2 = [0; 2];
        for j in 0..2 {
            wins_2[j] = sim.world.team(playoff_seeds2[j]).postseason_wins;
        }
        if wins_1[0] < 3 && wins_1[1] < 3 || wins_1[0] == wins_1[1] {
            games_active.push(generate_game(
                    playoff_seeds1[if higher_seed_hosts { 0 } else { 1 }],
                    playoff_seeds1[if higher_seed_hosts { 1 } else { 0 }],
                    107 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        if wins_2[0] < 3 && wins_2[1] < 3 || wins_2[0] == wins_2[1] {
            games_active.push(generate_game(
                    playoff_seeds2[if higher_seed_hosts { 0 } else { 1 }],
                    playoff_seeds2[if higher_seed_hosts { 1 } else { 0 }],
                    107 + i as usize,
                    sim.rng,
                    sim.world
            ));
        }
        loop {
            for game in games_active.iter_mut() {
                let evt = sim.next(game);
                evt.apply(game, sim.world);
                if let Event::GameOver = evt {
                    games_deactivated.push(game.id);
                }
            }
            games_active.retain(|g| !games_deactivated.contains(&g.id));
            if games_active.len() == 0 {
                break;
            }
        }
    }
    println!("Championship: {} {}-{} {}", sim.world.team(playoff_seeds1[0]).name, sim.world.team(playoff_seeds1[0]).postseason_wins, sim.world.team(playoff_seeds1[1]).postseason_wins, sim.world.team(playoff_seeds1[1]).name);
    println!("Championship: {} {}-{} {}", sim.world.team(playoff_seeds2[0]).name, sim.world.team(playoff_seeds2[0]).postseason_wins, sim.world.team(playoff_seeds2[1]).postseason_wins, sim.world.team(playoff_seeds2[1]).name);

    playoff_seeds1.retain(|&t| {
        let wins = sim.world.team(t).postseason_wins;
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        wins > losses
    });
    playoff_seeds2.retain(|&t| {
        let wins = sim.world.team(t).postseason_wins;
        let losses = sim.world.team(t).postseason_losses;
        sim.world.team_mut(t).postseason_wins = 0;
        sim.world.team_mut(t).postseason_losses = 0;
        wins > losses
    });
    
    let higher_seed = 
        if sim.world.team(playoff_seeds1[0]).wins > sim.world.team(playoff_seeds2[0]).wins 
        || sim.world.team(playoff_seeds1[0]).wins == sim.world.team(playoff_seeds2[0]).wins && sim.world.team(playoff_seeds1[0]).fate < sim.world.team(playoff_seeds2[0]).fate { 
            playoff_seeds1[0] 
        } else {
            playoff_seeds2[0]
        };
    let lower_seed = 
        if sim.world.team(playoff_seeds1[0]).wins > sim.world.team(playoff_seeds2[0]).wins 
        || sim.world.team(playoff_seeds1[0]).wins == sim.world.team(playoff_seeds2[0]).wins && sim.world.team(playoff_seeds1[0]).fate < sim.world.team(playoff_seeds2[0]).fate { 
            playoff_seeds2[0] 
        } else {
            playoff_seeds1[0]
        };
    for i in 0..5 {
        let mut wins = [0; 2];
        wins[0] = sim.world.team(higher_seed).postseason_wins;
        wins[1] = sim.world.team(lower_seed).postseason_wins;
        let higher_seed_hosts = i % 2 == 0;
        if wins[0] < 3 && wins[1] < 3 || wins[0] == wins[1] {
            let mut game = generate_game(
                if higher_seed_hosts { higher_seed } else { lower_seed },
                if higher_seed_hosts { lower_seed } else { higher_seed },
                112 + i as usize,
                sim.rng,
                sim.world
            );
            loop {
                let evt = sim.next(&game);
                evt.apply(&mut game, sim.world);
                if let Event::GameOver = evt {
                    break;
                }
            }
        }
    }

    println!("Internet Series: {} {}-{} {}", sim.world.team(playoff_seeds1[0]).name, sim.world.team(playoff_seeds1[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).name);
    
    // println!("Hello, world!");
}
