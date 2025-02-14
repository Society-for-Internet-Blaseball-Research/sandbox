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
    pub fn generate(rng: &mut Rng, _season_ruleset: u8) -> Weather {
        //todo: actually implement this
        /*let weights = match season_ruleset {
            11 -> [],
            12..24 -> todo!(),
        };*/
        let roll = rng.index(20);
        //todo: add season rulesets
        if roll < 1 {
            Weather::Eclipse
        } else if roll < 2 {
            Weather::Blooddrain
        } else if roll < 8 {
            Weather::Peanuts
        } else if roll < 11 {
            Weather::Birds
        } else if roll < 14 {
            Weather::Feedback
        } else {
            Weather::Reverb
        }
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    pub id: Uuid,
    pub weather: Weather,
    pub day: usize,

    pub top: bool,
    pub inning: i16, // 1-indexed
    pub balls: i16,
    pub strikes: i16,
    pub outs: i16,

    pub polarity: bool, //false for positive, true for negative
    pub scoring_plays_inning: u8,
    pub salmon_resets_inning: i16,

    pub events: Events,
    pub started: bool,

    pub home_team: GameTeam,
    pub away_team: GameTeam,

    pub runners: Baserunners,

    pub linescore_home: Vec<f64>, //for salmon purposes
    pub linescore_away: Vec<f64>, //the first element is the total score
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
            self.batting_team_mut().score += runs_scored;
        }
    }
    
    fn end_pa(&mut self) {
        let bt = self.batting_team_mut();
        bt.batter = None;
        bt.batter_index += 1;
        self.balls = 0;
        self.strikes = 0;
    }

    fn pick_fielder(&self, world: &World, roll: f64) -> Uuid {
        let pitching_team = world.team(self.pitching_team().id);

        let idx = (roll * (pitching_team.lineup.len() as f64)).floor() as usize;
        pitching_team.lineup[idx]
    }

    //might turn this into a more general function later
    //in place of an official incin target algorithm this might do
    fn pick_player_weighted(&self, world: &World, roll: f64, weight: impl Fn(Uuid) -> f64, only_current: bool) -> Uuid {
        let home_team = world.team(self.home_team.id);
        let away_team = world.team(self.away_team.id);

        let mut eligible_players = Vec::new();
        for i in 0..home_team.lineup.len() {
            eligible_players.push(home_team.lineup[i]);
        }
        for i in 0..away_team.lineup.len() {
            eligible_players.push(away_team.lineup[i]);
        }
        if only_current {
            eligible_players.push(self.home_team.pitcher);
            eligible_players.push(self.away_team.pitcher);
        } else {
            for i in 0..home_team.rotation.len() {
                eligible_players.push(home_team.rotation[i]);
            }
            for i in 0..away_team.rotation.len() {
                eligible_players.push(away_team.rotation[i]);
            }
        }

        let mut weights: Vec<f64> = Vec::new();
        for i in 0..eligible_players.len() {
            weights.push(weight(eligible_players[i]));
        }
        let weight_sum = weights.iter().sum::<f64>();
        if weight_sum == 0.0 {
            panic!("nobody can be chosen");
        }
        let last = weights.len() - 1 - weights.iter().rev().take_while(|x| **x == 0.0).collect::<Vec<&f64>>().len();
        let chosen_weight = roll * weight_sum;

        let mut counter = 0.0;
        for idx in 0..weights.len() {
            if chosen_weight < counter || idx == last {
                return eligible_players[idx];
            } else {
                counter += weights[idx];
            }
        }
        panic!("what")
    }

    pub fn get_run_value(&self) -> f64 {
        let polarity_coeff = if self.polarity { -1.0 } else { 1.0 };
        let sun_point_one_coeff = if let Weather::SunPointOne = self.weather { (self.inning as f64) / 10.0 } else { 1.0 };
        let sum_sun_coeff = if let Weather::SumSun = self.weather { self.scoring_plays_inning as f64 } else { 0.0 };
        1.0 * polarity_coeff * sun_point_one_coeff + sum_sun_coeff
    }

    //todo: just pass in a mods vec
    pub fn get_max_strikes(&self, world: &World) -> i16 {
        let batter = world.player(self.batting_team().batter.unwrap());
        let team = world.team(self.batting_team().id);
        if batter.mods.has(Mod::FourthStrike) || team.mods.has(Mod::FourthStrike) {
            4
        } else {
            3
        }
    }

    pub fn get_max_balls(&self, world: &World) -> i16 {
        let batter = world.player(self.batting_team().batter.unwrap());
        let team = world.team(self.batting_team().id);
        if batter.mods.has(Mod::WalkInThePark) || team.mods.has(Mod::WalkInThePark) {
            3
        } else {
            4
        }
    }

    pub fn get_bases(&self, world: &World) -> u8 {
        if world.team(self.batting_team().id).mods.has(Mod::FifthBase) {
            5
        } else {
            4
        }
    }

    pub fn compute_multiplier_data(&self, world: &World) -> MultiplierData {
        MultiplierData {
            //someone who knows about lifetimes more than me can probably
            //make this code more efficient
            batting_team_mods: world.team(self.batting_team().id).mods.clone(),
            pitching_team_mods: world.team(self.pitching_team().id).mods.clone(), 
            weather: self.weather.clone(),
            day: self.day,
            runners_empty: self.runners.empty(),
            top: self.top,
            maximum_blaseball: self.runners.iter().count() == 3, //todo: kid named fifth base
            at_bats: 0, //todo
        }
    }

    // todo: all of these are kind of nasty and will borrow all of self and that's usually annoying
    // note: the alternative is even more annoying
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
