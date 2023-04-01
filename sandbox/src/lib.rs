use bases::Baserunners;
use entities::World;
use mods::Mod;
use rng::Rng;
use uuid::Uuid;

pub mod bases;
pub mod entities;
pub mod formulas;
pub mod mods;
pub mod rng;

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

#[derive(Clone, Debug)]
pub enum EventData {
    BatterUp {
        id: Uuid,
    },
    InningSwitch {
        inning: i16,
        top: bool,
    },
    GameEnd {
        winner: Uuid,
    },

    Ball {
        walk: bool,
    },
    Strike {
        swing: bool,
        out: bool,
    },
    Foul,
    GroundOut {
        fielder: Uuid,
    },
    Flyout {
        fielder: Uuid,
    },

    HomeRun,

    // todo: should be one type with bases: u8?
    Triple,
    Double,
    Single,

    // todo: should be one type with success: bool?
    BaseSteal {
        player: Uuid,
        base_from: u8,
        base_to: u8,
    },
    CaughtStealing {
        player: Uuid,
        base_from: u8,
        base_to: u8,
    },
}

// todo: how much of this stuff really belongs on Game?
// can we get rid of passing rng/world around everywhere in a nice way?
// can we extract as much &logic as possible out and do all the &mut logic separately?
// like have `tick` not actually make any changes to the game state but instead apply that based on the EventData
impl Game {
    pub fn tick(&mut self, rng: &mut Rng, world: &mut World) -> EventData {
        // todo: elsewhere return
        // todo: weather
        // todo: consumers
        // todo: ballpark mods

        let max_balls = 4;
        let max_strikes = 3;
        let max_outs = 3;
        let max_innings = 99; // for testing

        // todo: put this here or set a game state flag? i think blaseball.com does the flag
        if self.outs >= max_outs {
            if self.top {
                self.top = false;
            } else {
                self.top = true;

                // todo: this logic is wrong(?) and also doesn't account for shame
                // also the 999 max innings is a placeholder
                if self.inning < max_innings || self.away_team.score == self.home_team.score {
                    self.inning += 1;
                } else {
                    return EventData::GameEnd {
                        winner: if self.away_team.score > self.home_team.score {
                            self.away_team.id
                        } else {
                            self.home_team.id
                        },
                    };
                }
            }

            self.balls = 0;
            self.strikes = 0;
            self.outs = 0;
            self.runners = Baserunners::new();
            return EventData::InningSwitch {
                inning: self.inning,
                top: self.top,
            };
        }

        let batting_team = self.batting_team_mut();
        let Some(batter_id) = batting_team.batter else {
            let new_idx = batting_team.batter_index + 1;
            let team = world.team(batting_team.id);
            let batter = team.lineup[new_idx % team.lineup.len()].clone();

            batting_team.batter_index = new_idx;
            batting_team.batter = Some(batter);
            return EventData::BatterUp { id: batter };
        };

        if let Some(outcome) = self.try_steals(rng, world) {
            if outcome.success {
                self.runners.advance(outcome.base_from);
                self.base_sweep();
                return EventData::BaseSteal {
                    player: outcome.runner_id,
                    base_from: outcome.base_from,
                    base_to: outcome.base_from + 1,
                };
            } else {
                self.runners.remove(outcome.base_from);
                return EventData::CaughtStealing {
                    player: outcome.runner_id,
                    base_from: outcome.base_from,
                    base_to: outcome.base_from + 1,
                };
            }
        }

        let event = match self.do_pitch(rng, world) {
            PitchOutcome::Ball => {
                self.balls += 1;

                let is_walk = self.balls >= max_balls;
                if is_walk {
                    self.runners.walk();
                    self.runners.add(0, batter_id);
                    self.end_pa();
                }

                EventData::Ball { walk: is_walk }
            }
            po @ (PitchOutcome::StrikeSwinging | PitchOutcome::StrikeLooking) => {
                self.strikes += 1;

                let is_strikeout = self.strikes >= max_strikes;
                if is_strikeout {
                    self.outs += 1;
                    self.end_pa();
                }

                // todo: this is ugly. maybe change to PitchOutcome::Strike in there too
                EventData::Strike {
                    swing: match po {
                        PitchOutcome::StrikeSwinging => true,
                        _ => false,
                    },
                    out: is_strikeout,
                }
            }
            PitchOutcome::Foul => {
                self.strikes = (self.strikes + 1).min(max_strikes - 1);

                EventData::Foul
            }
            PitchOutcome::GroundOut { fielder } => {
                self.outs += 1;
                // todo: adv

                self.end_pa();

                EventData::GroundOut { fielder }
            }
            PitchOutcome::Flyout { fielder } => {
                self.outs += 1;

                if self.outs < max_outs {
                    self.runners.advance_if(|_runner| rng.next() < 0.5);
                }

                self.end_pa();

                EventData::Flyout { fielder }
            }
            PitchOutcome::HomeRun => {
                self.runners.advance_all(4);
                self.runners.add(3, batter_id);

                self.end_pa();

                EventData::HomeRun
            }
            PitchOutcome::Triple => {
                self.runners.advance_all(3);
                self.runners
                    .advance_if(|runner| runner.base < 3 && rng.next() < 0.5);
                self.runners.add(2, batter_id);

                self.end_pa();

                EventData::Triple
            }
            PitchOutcome::Double => {
                self.runners.advance_all(2);
                self.runners
                    .advance_if(|runner| runner.base < 3 && rng.next() < 0.5);
                self.runners.add(1, batter_id);

                self.end_pa();

                EventData::Double
            }
            PitchOutcome::Single => {
                self.runners.advance_all(1);
                self.runners
                    .advance_if(|runner| runner.base < 3 && rng.next() < 0.5);
                self.runners.add(0, batter_id);

                self.end_pa();

                EventData::Single
            }
        };

        self.base_sweep();
        event
    }

    fn end_pa(&mut self) {
        self.batting_team_mut().batter = None;
        self.balls = 0;
        self.strikes = 0;
    }

    fn try_steals(&self, rng: &mut Rng, world: &World) -> Option<StealOutcome> {
        let steal_defender_id = self.pick_fielder(world, rng.next());
        let steal_defender = world.player(steal_defender_id);

        // todo: can we refactor `Baserunners` in a way where this sort of iteration is more natural
        for base in (0..4).rev() {
            if let Some(runner_id) = self.runners.at(base) {
                if self.runners.can_advance(base) {
                    let runner = world.player(runner_id);
                    let should_attempt =
                        rng.next() < formulas::steal_attempt_threshold(runner, steal_defender);
                    if should_attempt {
                        let success =
                            rng.next() < formulas::steal_success_threshold(runner, steal_defender);

                        return Some(StealOutcome {
                            runner_id,
                            base_from: base,
                            success,
                        });
                    }
                }
            }
        }

        None
    }

    fn do_pitch(&self, rng: &mut Rng, world: &World) -> PitchOutcome {
        let pitcher = world.player(self.pitching_team().pitcher);
        let batter = world.player(self.batting_team().batter.unwrap());

        let is_flinching = self.strikes == 0 && batter.mods.has(Mod::Flinch);

        let is_strike = rng.next() < formulas::strike_threshold(pitcher, batter, is_flinching);
        let does_swing = if !is_flinching {
            rng.next() < formulas::swing_threshold(pitcher, batter, is_strike)
        } else {
            false
        };

        if !does_swing {
            if is_strike {
                return PitchOutcome::StrikeLooking;
            } else {
                return PitchOutcome::Ball;
            }
        }

        let does_contact = rng.next() < formulas::contact_threshold(pitcher, batter, is_strike);
        if !does_contact {
            return PitchOutcome::StrikeSwinging;
        }

        let is_foul = rng.next() < formulas::foul_threshold(pitcher, batter);
        if is_foul {
            return PitchOutcome::Foul;
        }

        let out_defender_id = self.pick_fielder(world, rng.next());
        let out_defender = world.player(out_defender_id);

        let is_out = rng.next() < formulas::out_threshold(pitcher, batter, out_defender);
        if is_out {
            let fly_defender_id = self.pick_fielder(world, rng.next());
            let fly_defender = world.player(out_defender_id);

            let is_fly = rng.next() < formulas::fly_threshold(fly_defender);
            if is_fly {
                // ignore advancement for now
                return PitchOutcome::Flyout {
                    fielder: fly_defender_id,
                };
            }

            let ground_defender_id = self.pick_fielder(world, rng.next());
            return PitchOutcome::GroundOut {
                fielder: ground_defender_id,
            };
        }

        let is_hr = rng.next() < formulas::hr_threshold(pitcher, batter);
        if is_hr {
            return PitchOutcome::HomeRun;
        }

        let double_roll = rng.next();
        let triple_roll = rng.next();

        if triple_roll < formulas::triple_threshold(pitcher, batter) {
            return PitchOutcome::Triple;
        }
        if double_roll < formulas::double_threshold(pitcher, batter) {
            return PitchOutcome::Double;
        }

        PitchOutcome::Single
    }

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

// todo: do we need this?
// the idea is to separate out the basic "life of a pitch" rolls from the game state management and all the bonus side effects
// but it might just be annoying to "split" things like this
enum PitchOutcome {
    Ball,
    StrikeSwinging,
    StrikeLooking,
    Foul,
    GroundOut { fielder: Uuid },
    Flyout { fielder: Uuid },
    HomeRun,
    Triple,
    Double,
    Single,
    // todo: dp/fc
}

struct StealOutcome {
    runner_id: Uuid,
    base_from: u8,
    success: bool,
}
