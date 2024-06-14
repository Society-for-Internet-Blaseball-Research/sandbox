use bases::Baserunners;
use entities::World;
use uuid::Uuid;

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
    Salmon
}

#[derive(Clone, Debug)]
pub struct Game {
    pub weather: Weather,

    pub top: bool,
    pub inning: i16, // 1-indexed
    pub balls: i16,
    pub strikes: i16,
    pub outs: i16,

    pub events_inning: i16,

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

// todo: how much of this stuff really belongs on Game?
// can we get rid of passing rng/world around everywhere in a nice way?
// can we extract as much &logic as possible out and do all the &mut logic separately?
// like have `tick` not actually make any changes to the game state but instead apply that based on the EventData
impl Game {
    fn base_sweep(&mut self) {
        let mut new_runners = Baserunners::new();
        for runner in self.runners.iter() {
            // todo: baserunner code is bad
            if runner.base < 3 {
                new_runners.add(runner.base, runner.id);
            } else {
                let batting_team = if self.top {
                    &mut self.away_team
                } else {
                    &mut self.home_team
                };
                if self.outs < 3 {
                    batting_team.score += 1.0;
                }
            }
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
        let chosen_weight = roll * weight_sum;

        let mut counter = 0.0;
        for idx in 0..weights.len() {
            if chosen_weight < counter {
                return eligible_players[idx];
            } else {
                counter += weights[idx];
            }
        }
        panic!("what");
    }

    // todo: all of these are kind of nasty and will borrow all of self and that's usually annoying
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
