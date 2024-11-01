use crate::schedule::{generate_games, generate_schedule};
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
            games_active[i].home_team.pitcher = home_team.rotation[day % home_team.rotation.len()];
            let away_team = sim.world.team(games_active[i].away_team.id);
            games_active[i].away_team.pitcher = away_team.rotation[day % away_team.rotation.len()];
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
        let team = world.team(t);
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
    let mut division_playoffs: Vec<u8> = vec![0; 4];
    let mut playoff_seeds1: Vec<Uuid> = Vec::new();
    let mut playoff_seeds2: Vec<Uuid> = Vec::new();
    for &idx in indices.iter() {
        if idx < 10 {
            if playoff_seeds1.len() < 4 {
                playoff_seeds1.push(divisions[idx]);
                division_playoffs[idx / 5] += 1;
            }
        } else {
            if playoff_seeds2.len() < 4 {
                playoff_seeds2.push(divisions[idx]);
                division_playoffs[idx / 5] += 1;
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
            }
        }
        if division_playoffs.iter().copied().reduce(|acc, e| acc + e).unwrap() == 8 {
            break;
        }
    }

    for team in playoff_seeds1 {
        println!("{}", world.team(team).name);
    }
    for team in playoff_seeds2 {
        println!("{}", world.team(team).name);
    }

    // println!("Hello, world!");
}
