use bases::Baserunners;
use entities::World;
use uuid::Uuid;
use sim::Event;

pub mod bases;
pub mod entities;
pub mod formulas;
pub mod mods;
pub mod rng;
pub mod sim;

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
    //Coffee
    //Coffee 2
    //Coffee 3
    Salmon,
    //Glitter
    PolarityPlus,
    PolarityMinus,
    SunPointOne,
    SumSun,
    //Jazz
    Night
}

#[derive(Clone, Debug)]
pub struct Game {
    pub weather: Weather,

    pub top: bool,
    pub inning: i16, // 1-indexed
    pub balls: i16,
    pub strikes: i16,
    pub outs: i16,

    pub events_inning: u8,
    pub polarity: bool, //false for positive, true for negative
    pub scoring_plays_inning: u8,
    pub salmon_resets_inning: i16,
    pub last_salmon_inning: i16,

    //pub events: Events,

    pub home_team: GameTeam,
    pub away_team: GameTeam,

    pub runners: Baserunners,

    pub linescore_home: Vec<f64>, //for salmon purposes
    pub linescore_away: Vec<f64>, //the first element is the total score
}

#[derive(Clone, Debug)]
pub struct Events {
    event_list: Vec<Event>
}

#[derive(Clone, Debug)]
pub struct GameTeam {
    pub id: Uuid,
    pub pitcher: Uuid,
    pub batter: Option<Uuid>,
    pub batter_index: usize,
    pub score: f64, // sigh
}

// todo: how much of this stuff really belongs on Game?
// can we get rid of passing rng/world around everywhere in a nice way?
// can we extract as much &logic as possible out and do all the &mut logic separately?
// like have `tick` not actually make any changes to the game state but instead apply that based on the EventData
impl Game {
    fn base_sweep(&mut self) {
        let mut new_runners = Baserunners::new();
        let mut scoring_play = false;
        for runner in self.runners.iter() {
            // todo: baserunner code is bad
            if runner.base < 3 {
                new_runners.add(runner.base, runner.id);
            } else {
                scoring_play = true;
                let run_value = self.get_run_value();
                let batting_team = if self.top {
                    &mut self.away_team
                } else {
                    &mut self.home_team
                };
                if self.outs < 3 {
                    batting_team.score += run_value;
                }
            }
        }
        if scoring_play {
            self.scoring_plays_inning += 1;
        }
        self.runners = new_runners;
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
            eligible_players.push(home_team.lineup[i].clone());
        }
        for i in 0..away_team.lineup.len() {
            eligible_players.push(away_team.lineup[i].clone());
        }
        if only_current {
            eligible_players.push(self.home_team.pitcher.clone());
            eligible_players.push(self.away_team.pitcher.clone());
        } else {
            for i in 0..home_team.rotation.len() {
                eligible_players.push(home_team.rotation[i].clone());
            }
            for i in 0..away_team.rotation.len() {
                eligible_players.push(away_team.rotation[i].clone());
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

/*impl Events {
    fn new() -> Events {
        Events {
            event_list: Vec::new()
        }
    }
    fn add(&mut self, event: &Event) {
        self.event_list.push(event.clone());
    }
    fn last(&self) -> &Event {
        self.event_list.last().unwrap()
    }
    fn last_and(&self, allowed: Vec<Event>) -> Option<Event> {
        let last = self.event_list.last().unwrap();
        if allowed.contains(last) {
            Some(last.clone())
        } else {
            None
        }
    }
}*/
