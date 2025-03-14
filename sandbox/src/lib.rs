use bases::Baserunners;
use entities::World;
use mods::{Mod, Mods};
use rng::Rng;
use uuid::Uuid;
use events::Events;

pub mod bases;
pub mod entities;
pub mod formulas;
pub mod mods;
pub mod rng;
pub mod sim;
pub mod events;

#[derive(Clone, Debug)]
pub enum Weather {
    Sun,
    Eclipse,
    Peanuts,
    Birds,
    Feedback,
    Reverb,
    Blooddrain,
    Sun2,
    BlackHole,
    Coffee,
    Coffee2,
    Coffee3,
    Flooding,
    Salmon,
    //Glitter
    PolarityPlus,
    PolarityMinus,
    SunPointOne,
    SumSun,
    //Jazz
    Night
}

impl Weather {
    pub fn generate(rng: &mut Rng, season_ruleset: u8, day: usize) -> Weather {
        //todo: actually implement this
        let weights = match season_ruleset {
            11 => {
                if day < 72 {
                    vec![50, 20, 20, 35, 20, 20, 20, 50, 2, 2, 1] //todo: idk this just feels wrong
                } else {
                    vec![50, 20, 20, 35, 20, 20, 20, 50, 2, 2, 1, 200]
                }
            },
            _ => todo!(),
        };
        let weight_sum = match season_ruleset {
            11 => if day < 72 { 240 } else { 440 },
            _ => todo!(),
        };
        let weathers = [
            Weather::Sun2, 
            Weather::Eclipse, 
            Weather::Blooddrain, 
            Weather::Peanuts, 
            Weather::Birds, 
            Weather::Feedback,
            Weather::Reverb,
            Weather::BlackHole,
            Weather::Coffee,
            Weather::Coffee2,
            Weather::Coffee3,
            //Weather::Glitter,
            Weather::Flooding,
            Weather::Salmon,
            Weather::PolarityPlus,
            Weather::SunPointOne,
            Weather::SumSun
        ];
        let roll = rng.index(weight_sum);
        let mut slider = 0;
        for i in 0..weights.len() {
            let weight = weights[i];
            slider += weight;
            let weather = weathers[i].clone();
            if roll < slider {
                return weather;
            }
        }
        return Weather::Sun;
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    pub id: Uuid,
    pub weather: Weather,
    pub day: usize,

    pub inning: i16, // 1-indexed
    pub balls: i16,
    pub strikes: i16,
    pub outs: i16,

    pub polarity: bool, //false for positive, true for negative
    pub scoring_plays_inning: u8,
    pub salmon_resets_inning: i16,

    pub events: Events,
    pub started: bool,

    pub scoreboard: Scoreboard,

    pub runners: Baserunners,

    pub linescore_home: Vec<f64>, //for salmon purposes
    pub linescore_away: Vec<f64>, //the first element is the total score
}

#[derive(Clone, Debug)]
pub struct Scoreboard {
    pub home_team: GameTeam,
    pub away_team: GameTeam,
    pub top: bool
}

#[derive(Clone, Debug)]
pub struct GameTeam {
    pub id: Uuid,
    pub pitcher: Uuid,
    pub batter: Option<Uuid>,
    pub batter_index: usize,
    pub score: f64, // sigh
}

//stealing this from Astrid
pub struct MultiplierData {
    batting_team_mods: Mods,
    pitching_team_mods: Mods,
    weather: Weather,
    day: usize,
    runners_empty: bool,
    top: bool,
    maximum_blaseball: bool,
    at_bats: i32
}


// todo: how much of this stuff really belongs on Game?
// can we get rid of passing rng/world around everywhere in a nice way?
// can we extract as much &logic as possible out and do all the &mut logic separately?
// like have `tick` not actually make any changes to the game state but instead apply that based on the EventData
impl Game {
    pub fn new(team_a: Uuid, team_b: Uuid, day: usize, weather_override: Option<Weather>, world: &World, rng: &mut Rng) -> Game {
        Game {
            id: Uuid::new_v4(),
            weather: if weather_override.is_some() { weather_override.unwrap() } else { Weather::generate(rng, world.season_ruleset, day) },
            day,
            inning: 1,
            balls: 0,
            strikes: 0,
            outs: 0,
            polarity: false,
            scoring_plays_inning: 0,
            salmon_resets_inning: 0,
            events: Events::new(),
            started: false,
            scoreboard: Scoreboard {
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
                top: true,
            },
            runners: Baserunners::new(if world.team(team_b).mods.has(Mod::FifthBase) { 5 } else { 4 }),
            linescore_home: vec![if world.team(team_a).mods.has(Mod::HomeFieldAdvantage) { 1.0 } else { 0.0 }],
            linescore_away: vec![0.0],
        }
    }

    fn base_sweep(&mut self) {
        let mut new_runners = Baserunners::new(self.runners.base_number);
        let mut scoring_play = false;
        for runner in self.runners.iter() {
            // todo: baserunner code is bad
            if runner.base < self.runners.base_number - 1 {
                new_runners.add(runner.base, runner.id);
            } else {
                scoring_play = true;
            }
        }
        if scoring_play {
            self.scoring_plays_inning += 1;
        }
        self.runners = new_runners;
    }

    //note that this is only for runs scored on a regular event
    fn score(&mut self, world: &mut World) {
        if self.outs < 3 {
            let mut runs_scored = 0.0;
            for runner in self.runners.iter() {
                if runner.base >= self.runners.base_number - 1 {
                    runs_scored += self.get_run_value();
                    runs_scored += world.player(runner.id).get_run_value();
                    if world.player(runner.id).mods.has(Mod::FreeRefill) {
                        self.outs -= 1;
                        self.outs = self.outs.max(0); //can players refill the in with 0 outs
                                                      //or was that a bug?
                        world.player_mut(runner.id).mods.remove(Mod::FreeRefill);
                    }
                }
            }
            //run multipliers and sun wackiness here
            self.scoreboard.batting_team_mut().score += runs_scored;
        }
    }
    
    fn end_pa(&mut self) {
        let bt = self.scoreboard.batting_team_mut();
        bt.batter = None;
        bt.batter_index += 1;
        self.balls = 0;
        self.strikes = 0;
    }

    fn pick_fielder(&self, world: &World, roll: f64) -> Uuid {
        let pitching_team = world.team(self.scoreboard.pitching_team().id);

        let idx = (roll * (pitching_team.lineup.len() as f64)).floor() as usize;
        pitching_team.lineup[idx]
    }

    //might turn this into a more general function later
    //in place of an official incin target algorithm this might do
    fn pick_player_weighted(&self, world: &World, roll: f64, weight: impl Fn(&Uuid) -> bool, only_current: bool) -> Uuid {
        let home_team = world.team(self.scoreboard.home_team.id);
        let away_team = world.team(self.scoreboard.away_team.id);

        let mut eligible_players = Vec::new();
        for i in 0..home_team.lineup.len() {
            eligible_players.push(home_team.lineup[i]);
        }
        for i in 0..away_team.lineup.len() {
            eligible_players.push(away_team.lineup[i]);
        }
        if only_current {
            eligible_players.push(self.scoreboard.home_team.pitcher);
            eligible_players.push(self.scoreboard.away_team.pitcher);
        } else {
            for i in 0..home_team.rotation.len() {
                eligible_players.push(home_team.rotation[i]);
            }
            for i in 0..away_team.rotation.len() {
                eligible_players.push(away_team.rotation[i]);
            }
        }

        eligible_players.retain(weight);
        let idx = (roll * eligible_players.len() as f64).floor() as usize;
        eligible_players[idx]
    }

    pub fn get_run_value(&self) -> f64 {
        let polarity_coeff = if self.polarity { -1.0 } else { 1.0 };
        let sun_point_one_coeff = if let Weather::SunPointOne = self.weather { (self.inning as f64) / 10.0 } else { 1.0 };
        let sum_sun_coeff = if let Weather::SumSun = self.weather { self.scoring_plays_inning as f64 } else { 0.0 };
        1.0 * polarity_coeff * sun_point_one_coeff + sum_sun_coeff
    }

    //todo: just pass in a mods vec
    pub fn get_max_strikes(&self, world: &World) -> i16 {
        let batter = world.player(self.scoreboard.batting_team().batter.unwrap());
        let team = world.team(self.scoreboard.batting_team().id);
        if batter.mods.has(Mod::FourthStrike) || team.mods.has(Mod::FourthStrike) {
            4
        } else {
            3
        }
    }

    pub fn get_max_balls(&self, world: &World) -> i16 {
        let batter = world.player(self.scoreboard.batting_team().batter.unwrap());
        let team = world.team(self.scoreboard.batting_team().id);
        if batter.mods.has(Mod::WalkInThePark) || team.mods.has(Mod::WalkInThePark) {
            3
        } else {
            4
        }
    }

    pub fn get_bases(&self, world: &World) -> u8 {
        if world.team(self.scoreboard.batting_team().id).mods.has(Mod::FifthBase) {
            5
        } else {
            4
        }
    }

    pub fn batter(&self) -> Option<Uuid> {
        self.scoreboard.batting_team().batter
    }

    pub fn assign_batter(&mut self, new: Uuid) {
        self.scoreboard.batting_team_mut().batter = Some(new);
    }

    pub fn pitcher(&self) -> Uuid {
        self.scoreboard.pitching_team().pitcher
    }

    pub fn assign_pitcher(&mut self, new: Uuid) {
        self.scoreboard.pitching_team_mut().pitcher = new;
    }

    pub fn compute_multiplier_data(&self, world: &World) -> MultiplierData {
        MultiplierData {
            //someone who knows about lifetimes more than me can probably
            //make this code more efficient
            batting_team_mods: world.team(self.scoreboard.batting_team().id).mods.clone(),
            pitching_team_mods: world.team(self.scoreboard.pitching_team().id).mods.clone(), 
            weather: self.weather.clone(),
            day: self.day,
            runners_empty: self.runners.empty(),
            top: self.scoreboard.top,
            maximum_blaseball: self.runners.iter().count() == 3, //todo: kid named fifth base
            at_bats: 0, //todo
        }
    }
}

impl Scoreboard {
    pub fn pitching_team(&self) -> &GameTeam {
        if self.top {
            &self.home_team
        } else {
            &self.away_team
        }
    }

    pub fn batting_team(&self) -> &GameTeam {
        if self.top {
            &self.away_team
        } else {
            &self.home_team
        }
    }

    pub fn pitching_team_mut(&mut self) -> &mut GameTeam {
        if self.top {
            &mut self.home_team
        } else {
            &mut self.away_team
        }
    }

    pub fn batting_team_mut(&mut self) -> &mut GameTeam {
        if self.top {
            &mut self.away_team
        } else {
            &mut self.home_team
        }
    }
}
