use crate::{
    schedule::{generate_games, generate_schedule},
    get::{world, divisions, tiebreakers}
};
use sandbox::{
    entities::{LegendaryItem, NameGen, World},
    events::Event,
    rng::Rng,
    sim::Sim,
    mods::{Mod, ModLifetime},
    Game, Weather
};
use uuid::Uuid;
use clap::Parser;

mod schedule;
mod postseason;
mod get;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t=69)]
    s1: u64,
    #[arg(long, default_value_t=420)]
    s2: u64,
    #[arg(long, action)]
    prefill: bool,
    #[arg(long, default_value_t=11)]
    season: u8,
    #[arg(long, default_value_t=20)]
    teams: usize,
    #[arg(long, default_value_t=5)]
    divsize: usize,
    #[arg(long, action)]
    seasonmode: bool,
    #[arg(long, default_value_t=1)]
    loops: usize
}

fn main() {
    let args = Args::parse();

    //edit seed
    //let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    //let mut rng = Rng::new(37, 396396396396);
    //let mut rng = Rng::new(1923746321473263448, 2938897239474837483);
    
    //let mut rng = Rng::new(12933895067857275469, 10184511423779887981); //s12 seed

    let mut rng = Rng::new(args.s1, args.s2);
    //let name_gen = NameGen::new();
    let mut prefill = args.prefill;
    let mut world = if prefill {
        world(args.season)
    } else {
        World::new(args.season)
    }; //0-indexed season number

    let team_number: usize = args.teams;
    let div_size: usize = args.divsize;
    let mut teams: Vec<Uuid> = Vec::new();
    //todo: those go in args too
    let mut team_names: Vec<&str> = include_str!("teams.txt").trim().split(",").collect();
    let emojis: Vec<&str> = include_str!("emojis.txt").trim().split(" ").collect();
    if !prefill {
        for i in 0..team_number {
            teams.push(world.gen_team(&mut rng, team_names[i].to_string(), emojis[i].to_string()));
        }
    }
    //this is supposed to be editable so it's in human readable format
    //IMPORTANT: team names in teams.txt must be sorted alphabetically
    let divisions: Vec<Uuid> = 
        if prefill {
            divisions(args.season).unwrap().convert()
        } else {
            /*vec!["Baltimore Crabs", "Breckenridge Jazz Hands", "Chicago Firefighters", "Hades Tigers", "Mexico City Wild Wings",
            "Boston Flowers", "Hellmouth Sunbeams", "Houston Spies", "Miami Dale", "Unlimited Tacos",
            "Dallas Steaks", "New York Millennials", "Philly Pies", "San Francisco Lovers", "Seattle Garages", 
            "Canada Moist Talkers", "Charleston Shoe Thieves", "Hawai'i Fridays", "Kansas City Breath Mints", "Yellowstone Magic"]*/
            vec!["Atlantis Georgias", "Breckenridge Jazz Hands", "Chicago Firefighters", "Hades Tigers", "Mexico City Wild Wings", "Tokyo Lift",
            "Boston Flowers", "Hellmouth Sunbeams", "Houston Spies", "Miami Dale", "Ohio Worms", "Unlimited Tacos",
            "Core Mechanics", "Dallas Steaks", "New York Millennials", "Philly Pies", "San Francisco Lovers", "Seattle Garages", 
            "Baltimore Crabs", "Canada Moist Talkers", "Charleston Shoe Thieves", "Hawai'i Fridays", "Kansas City Breath Mints", "Yellowstone Magic"]
                .iter()
                .map(|s| teams[team_names.binary_search(s).unwrap_or_else(|_| panic!("team not found: {s}"))])
                .collect()
            /*let mut v = Vec::new();
            for i in 0..12 {
                v.push(teams[i]);
            }
            v*/
        };

    //edit mods and legendary items
    //world.team_mut(team_a).mods.add(Mod::FourthStrike, ModLifetime::Season);
    //world.team_name_mut(String::from("Hades Tigers")).mods.add(Mod::HomeFieldAdvantage, ModLifetime::Season);
    //world.player_mut(world.team_name(String::from("Kansas City Breath Mints")).lineup[2]).mods.add(Mod::Squiddish, ModLifetime::Permanent);
    //world.player_mut(world.team(team_a).lineup[0]).add_legendary_item(LegendaryItem::TheIffeyJr);
    //world.player_mut(world.team_name(String::from("Charleston Shoe Thieves")).rotation[0]).mods.add(Mod::Superyummy, ModLifetime::Permanent);
    
    let mut fate_vec: Vec<Uuid> = if prefill { tiebreakers(11).unwrap() } else { Vec::new() };
    let mut fate_pool: Vec<usize> = (0..team_number).collect();
    let mut fates: Vec<usize> = Vec::new();
    for i in 0..team_number {
        let fate_roll = if prefill { fate_vec.iter().position(|&id| id == divisions[i]).unwrap() } else { fate_pool[rng.index(team_number - i)] };
        world.team_mut(divisions[i]).fate = fate_roll;
        fates.push(fate_roll);
        //println!("{} {}", world.team(teams[i]).name, world.team(teams[i]).fate);
        if !prefill { fate_pool.retain(|&j| j != fate_roll) };
    }
    let season_mode = args.seasonmode;
    let loop_number = args.loops;
    for i in 0..loop_number {
        let mut og_world = world.clone();
        let mut sim = Sim::new(&mut og_world, &mut rng);
        if season_mode {
            let days_in_season = 99;
            let game_number = team_number / 2;
            let games = generate_games(generate_schedule(days_in_season, &divisions, sim.rng, team_number, div_size), sim.world, sim.rng);
            for day in 0..days_in_season {
                let mut games_active: Vec<Game> = Vec::new();
                for i in (day * game_number)..((day + 1) * game_number) {
                    games_active.push(games[i].clone());
                }
                for i in 0..game_number {
                    let home_team = sim.world.team(games_active[i].scoreboard.home_team.id);
                    let home_pitcher = home_team.rotation[day % home_team.rotation.len()];
                    games_active[i].scoreboard.home_team.pitcher = if sim.world.player(home_pitcher).mods.has(Mod::Shelled) { home_team.rotation[(day - 1) % home_team.rotation.len()] } else { home_pitcher };
                    let away_team = sim.world.team(games_active[i].scoreboard.away_team.id);
                    let away_pitcher = away_team.rotation[day % away_team.rotation.len()];
                    games_active[i].scoreboard.away_team.pitcher = if sim.world.player(away_pitcher).mods.has(Mod::Shelled) { away_team.rotation[(day - 1) % away_team.rotation.len()] } else { away_pitcher };
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
                            games_deactivated.push(game.id);
                        }
                    }
                    games_active.retain(|g| !games_deactivated.contains(&g.id));
                    if games_active.len() == 0 {
                        break;
                    }
                }
                sim.world.clear_game();
                if day % 9 == 8 {
                    sim.world.clear_weekly();
                }
                
                let mut party_standings: Vec<i16> = Vec::new();
                for &t in divisions.iter() {
                    let team = sim.world.team(t);
                    party_standings.push(team.losses);
                }
                postseason::update_party(&divisions, &party_standings, &fates, day, sim.world, sim.rng);
            }

            //do standings and fates need to be a vec actually
            let mut standings: Vec<i16> = Vec::new();
            
            for &t in divisions.iter() {
                let team = sim.world.team(t);
                standings.push(team.wins);
                println!("{}: {}-{}", team.name, team.wins, team.losses);
            }

            let (mut playoff_seeds1, mut playoff_seeds2) = postseason::generate_seeding(&divisions, &standings, &fates, sim.rng);

            for team in playoff_seeds1.iter() {
                println!("{}", sim.world.team(*team).name);
            }
            for team in playoff_seeds2.iter() {
                println!("{}", sim.world.team(*team).name);
            }

            for &team in divisions.iter() {
                sim.world.team_mut(team).partying = false;
            }

            let mut wc_days = 0;
            //todo: make this a method with sim either as instance or parameter
            loop {
                let mut games_active: Vec<Game> = postseason::generate_wildcard(&playoff_seeds1, &playoff_seeds2, wc_days, sim.world, sim.rng);
                if games_active.len() == 0 {
                    break;
                }
                let mut games_deactivated: Vec<Uuid> = Vec::new();
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
                wc_days += 1;
            }

            println!("Wildcard: {} {}-{} {}", sim.world.team(playoff_seeds1[3]).name, sim.world.team(playoff_seeds1[3]).postseason_wins, sim.world.team(playoff_seeds1[4]).postseason_wins, sim.world.team(playoff_seeds1[4]).name);
            println!("Wildcard: {} {}-{} {}", sim.world.team(playoff_seeds2[3]).name, sim.world.team(playoff_seeds2[3]).postseason_wins, sim.world.team(playoff_seeds2[4]).postseason_wins, sim.world.team(playoff_seeds2[4]).name);

            playoff_seeds1.retain(|&t| {
                let wins = sim.world.team(t).postseason_wins;
                let losses = sim.world.team(t).postseason_losses;
            sim.world.team_mut(t).postseason_wins = 0;
            sim.world.team_mut(t).postseason_losses = 0;
            wins + losses == 0 || wins > losses
            });
            playoff_seeds2.retain(|&t| {
                let wins = sim.world.team(t).postseason_wins;
                let losses = sim.world.team(t).postseason_losses;
                sim.world.team_mut(t).postseason_wins = 0;
                sim.world.team_mut(t).postseason_losses = 0;
                wins + losses == 0 || wins > losses
            });

            let mut div_days = 0;
            loop {
                let mut games_active: Vec<Game> = postseason::generate_divisional(&playoff_seeds1, &playoff_seeds2, div_days, sim.world, sim.rng);
                if games_active.len() == 0 {
                    break;
                }
                let mut games_deactivated: Vec<Uuid> = Vec::new();
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
                div_days += 1;
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
       
            let mut champ_days = 0;
            loop {
                let mut games_active: Vec<Game> = postseason::generate_championship(&playoff_seeds1, &playoff_seeds2, champ_days, sim.world, sim.rng);
                if games_active.len() == 0 {
                    break;
                }
                let mut games_deactivated: Vec<Uuid> = Vec::new();
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
                champ_days += 1;
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
            let higher_seed_in_league_one = sim.world.team(playoff_seeds1[0]).wins > sim.world.team(playoff_seeds2[0]).wins 
                || sim.world.team(playoff_seeds1[0]).wins == sim.world.team(playoff_seeds2[0]).wins && sim.world.team(playoff_seeds1[0]).fate < sim.world.team(playoff_seeds2[0]).fate; 

            let higher_seed = 
                if higher_seed_in_league_one {
                    playoff_seeds1[0] 
                } else {
                    playoff_seeds2[0]
                };
            //todo: be concise here
            let lower_seed = 
                if higher_seed_in_league_one {
                    playoff_seeds2[0] 
                } else {
                    playoff_seeds1[0]
                };
            let mut ilb_days = 0;
            loop {
                let mut wins = [0; 2];
                wins[0] = sim.world.team(higher_seed).postseason_wins;
                wins[1] = sim.world.team(lower_seed).postseason_wins;
                let higher_seed_hosts = ilb_days % 2 == 0;
                if wins[0] < 3 && wins[1] < 3 || wins[0] == wins[1] {
                    let mut game = Game::new(
                        if higher_seed_hosts { higher_seed } else { lower_seed },
                        if higher_seed_hosts { lower_seed } else { higher_seed },
                        112 + ilb_days as usize,
                        None,
                        sim.world,
                        sim.rng
                    );
                    loop {
                        let evt = sim.next(&game);
                        evt.apply(&mut game, sim.world);
                        if let Event::GameOver = evt {
                            break;
                        }
                    }
                } else {
                    break;
                }
                ilb_days += 1;
            }

            println!("Internet Series: {} {}-{} {}", sim.world.team(playoff_seeds1[0]).name, sim.world.team(playoff_seeds1[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).name);
        
            sim.world.clear_season();
        } else {
            //todo: id by name function
            /*let id = sim.world.gen_player(sim.rng, divisions[6]);
            println!("{}", id);
            sim.world.player_mut(id).team = None;
            sim.world.hall.push(id);*/
            let mut game = Game::new(divisions[15], divisions[22], 0, Some(Weather::Coffee), sim.world, sim.rng); 
            println!("{} at {}, {:?}",
                sim.world.team(game.scoreboard.away_team.id).name,
                sim.world.team(game.scoreboard.home_team.id).name,
                game.weather
            );
            loop {
                let evt = sim.next(&game);
                evt.apply(&mut game, sim.world);

                if let Event::GameOver = evt {
                    println!(
                        "game over! {}: {}, {}: {}",
                        sim.world.team(game.scoreboard.away_team.id).name,
                        game.scoreboard.away_team.score,
                        sim.world.team(game.scoreboard.home_team.id).name,
                        game.scoreboard.home_team.score
                    );
                    break;
                }
                let base = if game.runners.base_number == 5 {
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

                let away_score = (game.scoreboard.away_team.score * 10.0).round() / 10.0;
                let home_score = (game.scoreboard.home_team.score * 10.0).round() / 10.0; //floats

                println!(
                    "{}{} {}@{} ({}b/{}s/{}o) {} {:?}",
                    if game.scoreboard.top { "t" } else { "b" },
                    game.inning,
                    away_score,
                    home_score,
                    game.balls,
                    game.strikes,
                    game.outs,
                    base,
                    evt
                );
            }
        }
        // println!("Hello, world!");
    }
}
