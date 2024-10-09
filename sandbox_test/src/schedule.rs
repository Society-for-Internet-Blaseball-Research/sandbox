use sandbox::{
    bases::Baserunners,
    entities::World,
    events::Events,
    rng::Rng,
    mods::Mod,
    Game, GameTeam,
};
use uuid::Uuid;

fn generate_game(team_a: Uuid, team_b: Uuid, day: usize, rng: &mut Rng, world: &World) -> Game {
    Game {
        id: Uuid::new_v4(),
        weather: sandbox::Weather::generate(rng),
        day,
        top: true,
        inning: 1,
        home_team: GameTeam {
            id: team_a,
            //todo: days
            pitcher: world.team(team_a).rotation[day % world.team(team_a).rotation.len()],
            batter: None,
            batter_index: 0,
            score: if world.team(team_a).mods.has(Mod::HomeFieldAdvantage) { 1.0 } else { 0.0 },
        },
        away_team: GameTeam {
            id: team_b,
            pitcher: world.team(team_b).rotation[day % world.team(team_b).rotation.len()],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        balls: 0,
        strikes: 0,
        outs: 0,
        polarity: false,
        scoring_plays_inning: 0,
        salmon_resets_inning: 0,
        events: Events::new(),
        runners: Baserunners::new(if world.team(team_b).mods.has(Mod::FifthBase) { 5 } else { 4 }),
        linescore_home: vec![if world.team(team_a).mods.has(Mod::HomeFieldAdvantage) { 1.0 } else { 0.0 }],
        linescore_away: vec![0.0],
    }
}

pub fn generate_schedule(days: usize,  divisions: &Vec<Uuid>, rng: &mut Rng) -> Vec<ScheduleGame> {
    let mut schedule: Vec<ScheduleGame> = Vec::new();
    let mut series_distr = vec![20, 130, 180]; //interleague, league, division, total
    for day in 0..days {
        if day % 3 == 0 {
            let mut daily_games: Vec<ScheduleGame> = Vec::new(); 
            //every number is an index of the team in divisions
            //debating whether to use this or a regular array with binary search
            //although a 2d array is probably needed anyway
            let mut remaining_teams: Vec<Vec<usize>> = vec![(0..5).collect(), (5..10).collect(), (10..15).collect(), (15..20).collect()];
            let mut orders: Vec<usize> = (0..10).collect();

            let mut interleague_games = 0;
            for _j in 0..10 {
                if rng.index(series_distr[0] + series_distr[1] + series_distr[2]) < series_distr[0] {
                    interleague_games += 1;
                }
            }
            //rounding down to even integer and checking for underflow
            interleague_games = (interleague_games / 2 * 2).min(series_distr[0]);
            for j in 0..interleague_games {
                let home_team_idx = rng.index(20 - 2 * j);
                let mut away_team_idx = rng.index(10 - j);
                if home_team_idx < 10 - j {
                    away_team_idx += 10 - j;
                }
                let home_team_num = remaining_teams[home_team_idx / 5][home_team_idx % 5];
                let away_team_num = remaining_teams[away_team_idx / 5][away_team_idx % 5];
                let order_num = rng.index(orders.len());
                let order = orders[order_num];
                daily_games.push(ScheduleGame {
                    day,
                    order,
                    home_team: divisions[home_team_num],
                    away_team: divisions[away_team_num]
                });
                for k in 0..4 {
                    remaining_teams[k].retain(|&n| n != home_team_num && n != away_team_num);
                }
                orders.retain(|&n| n != order);
            }
            println!("{} interleague games", interleague_games);
            series_distr[0] -= interleague_games;

            //1 if league games are mathematically forced, 0 if not
            let league1_forced_interdiv = remaining_teams[0].len() % 2;
            let league2_forced_interdiv = remaining_teams[2].len() % 2;
            let mut league1_interdiv_games = 0;
            let mut league2_interdiv_games = 0;
            for j in 0..10 {
                if rng.index(series_distr[1] + series_distr[2]) < series_distr[1] {
                    if j < 5 {
                        league1_interdiv_games += 1;
                    } else {
                        league2_interdiv_games += 1;
                    }
                }
            }
            league1_interdiv_games = (league1_interdiv_games / 2 * 2 + league1_forced_interdiv)
                .min(remaining_teams[0].len())
                .min(remaining_teams[1].len())
                .min(series_distr[1]);
            league2_interdiv_games = (league2_interdiv_games / 2 * 2 + league2_forced_interdiv)
                .min(remaining_teams[2].len())
                .min(remaining_teams[3].len())
                .min(series_distr[1]);
            //todo: this is a really nasty fix, and also it breaks if days are not divisible by 3
            /*if day == days - 3 {
                league2_interdiv_games = series_distr[1] - league1_interdiv_games;
            }*/
            for j in 0..league1_interdiv_games {
                let mut home_team_idx = rng.index(10 - interleague_games - 2 * j);
                let mut div1_hosts = true;
                if home_team_idx >= remaining_teams[0].len() {
                    div1_hosts = false;
                    home_team_idx -= remaining_teams[0].len();
                };
                let away_team_idx = if div1_hosts {
                    rng.index(remaining_teams[1].len() - j)
                } else {
                    rng.index(remaining_teams[0].len() - j)
                };
                let home_team_num = remaining_teams[if div1_hosts { 0 } else { 1 }][home_team_idx];
                let away_team_num = remaining_teams[if div1_hosts { 1 } else { 0 }][away_team_idx];
                let order_num = rng.index(orders.len());
                let order = orders[order_num];
                daily_games.push(ScheduleGame {
                    day,
                    order,
                    home_team: divisions[home_team_num],
                    away_team: divisions[away_team_num]
                });
                for k in 0..2 {
                    remaining_teams[k].retain(|&n| n != home_team_num && n != away_team_num);
                }
                orders.retain(|&n| n != order);
            }
            for j in 0..league2_interdiv_games {
                let mut home_team_idx = rng.index(10 - interleague_games - 2 * j);
                let mut div3_hosts = true;
                if home_team_idx >= remaining_teams[2].len() {
                    div3_hosts = false;
                    home_team_idx -= remaining_teams[2].len();
                };
                let away_team_idx = if div3_hosts {
                    rng.index(remaining_teams[3].len() - j)
                } else {
                    rng.index(remaining_teams[2].len() - j)
                };
                let home_team_num = remaining_teams[if div3_hosts { 2 } else { 3 }][home_team_idx];
                let away_team_num = remaining_teams[if div3_hosts { 3 } else { 2 }][away_team_idx];
                let order_num = rng.index(orders.len());
                let order = orders[order_num];
                daily_games.push(ScheduleGame {
                    day,
                    order,
                    home_team: divisions[home_team_num],
                    away_team: divisions[away_team_num]
                });
                for k in 2..4 {
                    remaining_teams[k].retain(|&n| n != home_team_num && n != away_team_num);
                }
                orders.retain(|&n| n != order);               
            }
            println!("{} league games", league1_interdiv_games + league2_interdiv_games);
            println!("{}", orders.len());
            series_distr[1] -= league1_interdiv_games + league2_interdiv_games;

            let div_games = 10 - interleague_games - league1_interdiv_games - league2_interdiv_games;
            for j in 0..4 {
                if remaining_teams[j].len() % 2 == 1 {
                    panic!("wrong amount of teams playing interleague/interdivision matchups");
                }
                for k in 0..(remaining_teams[j].len() / 2) {
                    let home_team_idx = rng.index(remaining_teams[j].len());
                    let mut away_team_idx = rng.index(remaining_teams[j].len() - 1);
                    //avoiding scenario of a team playing itself
                    if away_team_idx >= home_team_idx {
                        away_team_idx += 1;
                    }
                    let home_team_num = remaining_teams[j][home_team_idx];
                    let away_team_num = remaining_teams[j][away_team_idx];
                    let order_num = rng.index(orders.len());
                    let order = orders[order_num];
                    daily_games.push(ScheduleGame {
                        day,
                        order,
                        home_team: divisions[home_team_num],
                        away_team: divisions[away_team_num]
                    });
                    remaining_teams[j].retain(|&n| n != home_team_num && n != away_team_num);
                    orders.retain(|&n| n != order);
                }
            }
            println!("{} division games", div_games);
            series_distr[2] -= div_games;
            if series_distr[2] > 180 {
                panic!("total games do not add up");
            }
            if orders.len() > 0 {
                println!("{}", orders.len());
                panic!("not enough daily games");
            }
            if daily_games.len() > 10 {
                panic!("too much daily games");
            }
            daily_games.sort_by(|g1, g2| g1.order.cmp(&g2.order));
            schedule.append(&mut daily_games);
        } else {
            let mut daily_games: Vec<ScheduleGame> = Vec::new();
            for i in ((day - 1) * 10)..(day * 10) {
                daily_games.push(schedule[i].clone());
            }
            let mut orders: Vec<usize> = (0..10).collect();
            for game in daily_games.iter_mut() {
                game.day = day;
                let order = rng.index(orders.len());
                game.order = orders[order];
                orders.retain(|&n| n != order);
                let team = game.home_team;
                game.home_team = game.away_team;
                game.away_team = team;
            }
            daily_games.sort_by(|g1, g2| g1.order.cmp(&g2.order));
            schedule.append(&mut daily_games);
        }
    }
    schedule
}

pub fn generate_games(schedule: Vec<ScheduleGame>, world: &World, rng: &mut Rng) -> Vec<Game> {
    schedule.iter().map(|sg| generate_game(sg.home_team, sg.away_team, sg.day, rng, world)).collect()
}

#[derive(Debug, Clone)]
pub struct ScheduleGame {
    day: usize,
    order: usize,
    home_team: Uuid,
    away_team: Uuid
}
