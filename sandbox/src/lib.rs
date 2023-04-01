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
    Sun2,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub weather: Weather,

    pub top: bool,
    pub inning: i16, // 1-indexed
    pub balls: i16,
    pub strikes: i16,
    pub outs: i16,

    pub home_team: GameTeam,
    pub away_team: GameTeam,

    pub runners: Baserunners,
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
                batting_team.score += 1.0;
            }
        }
        self.runners = new_runners;
    }

    fn pick_fielder(&self, world: &World, roll: f64) -> Uuid {
        let pitching_team = world.team(self.pitching_team().id);

        let idx = (roll * (pitching_team.lineup.len() as f64)).floor() as usize;
        pitching_team.lineup[idx]
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
