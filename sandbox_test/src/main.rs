use crate::schedule::{generate_game, generate_games, generate_schedule};
use sandbox::{
    entities::{LegendaryItem, NameGen, World},
    events::Event,
    rng::Rng,
    sim::Sim,
    mods::{Mod, ModLifetime},
    Game, Weather
};
use uuid::Uuid;

mod schedule;
mod postseason;

fn main() {
    //edit seed
    //let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    //let mut rng = Rng::new(37, 396396396396);
    let mut rng = Rng::new(1923746321473263448, 2938897239474837483);

    let mut world = World::new(11); //0-indexed season number
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
        vec!["Baltimore Crabs", "Breckenridge Jazz Hands", "Chicago Firefighters", "Hades Tigers", "Mexico City Wild Wings",
        "Boston Flowers", "Hellmouth Sunbeams", "Houston Spies", "Miami Dale", "Unlimited Tacos",
        "Dallas Steaks", "Hades Tigers", "New York Millennials", "Philly Pies", "Seattle Garages", 
        "Canada Moist Talkers", "Charleston Shoe Thieves", "Hawai'i Fridays", "Kansas City Breath Mints", "Yellowstone Magic"]
            .iter()
            .map(|s| teams[team_names.binary_search(s).expect("team not found")])
            .collect();

    //edit mods and legendary items
    //world.team_mut(team_a).mods.add(Mod::FourthStrike, ModLifetime::Season);
    //world.team_name_mut(String::from("Hades Tigers")).mods.add(Mod::HomeFieldAdvantage, ModLifetime::Season);
    //world.player_mut(world.team_name(String::from("Miami Dale")).lineup[4]).mods.add(Mod::Electric, ModLifetime::Game);
    //world.player_mut(world.team(team_a).lineup[0]).add_legendary_item(LegendaryItem::TheIffeyJr);
    //world.player_mut(world.team_name(String::from("Charleston Shoe Thieves")).rotation[0]).mods.add(Mod::Mild, ModLifetime::Permanent);
    
    let mut fate_pool: Vec<usize> = (0..20).collect();
    for i in 0..20 {
        let fate_roll = fate_pool[rng.index(20 - i)];
        world.team_mut(teams[i]).fate = fate_roll;
        //println!("{} {}", world.team(teams[i]).name, world.team(teams[i]).fate);
        fate_pool.retain(|&j| j != fate_roll);
    }
    let mut sim = Sim::new(&mut world, &mut rng);
    let season_mode = false;
    if season_mode {
        let days_in_season = 99;
        let games = generate_games(generate_schedule(days_in_season, &divisions, sim.rng), sim.world, sim.rng);
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
            sim.world.clear_game();
            if day % 9 == 8 {
                sim.world.clear_weekly();
            }
        }

        //do standings and fates need to be a vec actually
        let mut standings: Vec<i16> = Vec::new();
        let mut fates: Vec<usize> = Vec::new();
        
        for &t in divisions.iter() {
            let team = sim.world.team(t);
            standings.push(team.wins);
            fates.push(team.fate);
            println!("{}: {}-{}", team.name, team.wins, team.losses);
        }

        let (mut playoff_seeds1, mut playoff_seeds2) = postseason::generate_seeding(&divisions, &standings, &fates, sim.rng);

        for team in playoff_seeds1.iter() {
            println!("{}", sim.world.team(*team).name);
        }
        for team in playoff_seeds2.iter() {
            println!("{}", sim.world.team(*team).name);
        }

        //todo: make this a method with sim either as instance or parameter
        for i in 0..3 {
            let mut games_active: Vec<Game> = postseason::generate_wildcard(&playoff_seeds1, &playoff_seeds2, i, sim.world, sim.rng);
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
            let mut games_active: Vec<Game> = postseason::generate_divisional(&playoff_seeds1, &playoff_seeds2, i, sim.world, sim.rng);
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
            let mut games_active: Vec<Game> = postseason::generate_championship(&playoff_seeds1, &playoff_seeds2, i, sim.world, sim.rng);
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
            }
        }

        println!("Internet Series: {} {}-{} {}", sim.world.team(playoff_seeds1[0]).name, sim.world.team(playoff_seeds1[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).postseason_wins, sim.world.team(playoff_seeds2[0]).name);
    
        sim.world.clear_season();
    } else {
        //todo: id by name function
        let mut game = generate_game(divisions[0], divisions[16], 0, Some(Weather::Birds), sim.world, sim.rng); 
        println!("{} at {}, {:?}",
            sim.world.team(game.away_team.id).name,
            sim.world.team(game.home_team.id).name,
            game.weather
        );
        loop {
            let evt = sim.next(&game);
            evt.apply(&mut game, sim.world);

            if let Event::GameOver = evt {
                println!(
                    "game over! {}: {}, {}: {}",
                    sim.world.team(game.away_team.id).name,
                    game.away_team.score,
                    sim.world.team(game.home_team.id).name,
                    game.home_team.score
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
            );
        }
    }
    // println!("Hello, world!");
}
