use uuid::Uuid;

use crate::{bases::Baserunners, entities::World, formulas, mods::Mod, rng::Rng, Game};

pub trait Plugin {
    fn tick(&self, _game: &Game, _world: &World, _rng: &mut Rng) -> Option<Event> {
        None
    }
}

pub struct Sim<'a> {
    plugins: Vec<Box<dyn Plugin>>,
    world: &'a mut World,
    rng: &'a mut Rng,
}

impl<'a> Sim<'a> {
    pub fn new(world: &'a mut World, rng: &'a mut Rng) -> Sim<'a> {
        Sim {
            world,
            rng,
            plugins: vec![
                Box::new(InningStatePlugin),
                Box::new(BatterStatePlugin),
                Box::new(StealingPlugin),
                Box::new(BasePlugin),
            ],
        }
    }
    pub fn next(&mut self, game: &Game) -> Event {
        for plugin in self.plugins.iter() {
            if let Some(event) = plugin.tick(game, &self.world, &mut self.rng) {
                return event;
            }
        }

        panic!("uhhh")
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    BatterUp {
        batter: Uuid,
    },
    InningSwitch {
        inning: i16,
        top: bool,
    },
    GameOver,

    Ball,
    Strike,
    Foul,

    Strikeout,
    Walk,
    HomeRun,

    // todo: find a nicer way to encode runner advancement
    BaseHit {
        bases: u8,
        runners_after: Baserunners,
    },
    GroundOut {
        fielder: Uuid,
        runners_after: Baserunners,
    },
    Flyout {
        fielder: Uuid,
        runners_after: Baserunners,
    },
    DoublePlay,
    FieldersChoice,

    BaseSteal {
        runner: Uuid,
        base_from: u8,
        base_to: u8,
    },
    CaughtStealing {
        runner: Uuid,
        base_from: u8,
    },
}

impl Event {
    pub fn apply(&self, game: &mut Game, _world: &mut World) {
        match *self {
            Event::BatterUp { batter } => {
                let bt = game.batting_team_mut();
                bt.batter = Some(batter);
            }
            Event::InningSwitch { inning, top } => {
                game.inning = inning;
                game.top = top;
                game.outs = 0;
                game.balls = 0;
                game.strikes = 0;
            }
            Event::GameOver => {}
            Event::Ball => {
                game.balls += 1;
            }
            Event::Strike => {
                game.strikes += 1;
            }
            Event::Foul => {
                game.strikes += 1;
                game.strikes = game.strikes.min(2);
            }
            Event::Strikeout => {
                game.outs += 1;
                end_pa(game);
            }
            Event::Walk => {
                // maybe we should put batter in the event
                game.runners.add(0, game.batting_team().batter.unwrap());
                end_pa(game);
            }
            Event::HomeRun => {
                game.runners.advance_all(4);
                game.base_sweep();
                end_pa(game);
            }
            Event::BaseHit {
                bases,
                ref runners_after,
            } => {
                game.runners = runners_after.clone();
                game.base_sweep();
                game.runners
                    .add(bases - 1, game.batting_team().batter.unwrap());
                end_pa(game);
            }
            Event::GroundOut {
                fielder: _fielder,
                ref runners_after,
            } => {
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                end_pa(game);
            }
            Event::Flyout {
                fielder: _fielder,
                ref runners_after,
            } => {
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                end_pa(game);
            }
            Event::DoublePlay => todo!(),
            Event::FieldersChoice => todo!(),
            Event::BaseSteal {
                runner: _runner,
                base_from,
                base_to: _base_to,
            } => {
                game.runners.advance(base_from);
                game.base_sweep();
            }
            Event::CaughtStealing {
                runner: _runner,
                base_from,
            } => {
                game.runners.remove(base_from);
                game.outs += 1;
            }
        }
    }
}

// this should maybe be an instance method on Game idk
fn end_pa(game: &mut Game) {
    let bt = game.batting_team_mut();
    bt.batter = None;
    bt.batter_index += 1;
    game.balls = 0;
    game.strikes = 0;
}

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

struct BasePlugin;
impl Plugin for BasePlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        let max_balls = 4;
        let max_strikes = 3;
        // let max_outs = 3;

        let last_strike = (game.strikes + 1) >= max_strikes;

        Some(match do_pitch(world, game, rng) {
            PitchOutcome::Ball => {
                if (game.balls + 1) < max_balls {
                    Event::Ball
                } else {
                    Event::Walk
                }
            }
            PitchOutcome::StrikeSwinging => {
                if last_strike {
                    Event::Strikeout
                } else {
                    Event::Strike
                }
            }
            PitchOutcome::StrikeLooking => {
                if last_strike {
                    Event::Strikeout
                } else {
                    Event::Strike
                }
            }
            PitchOutcome::Foul => Event::Foul,
            PitchOutcome::GroundOut { fielder } => Event::GroundOut {
                fielder,
                runners_after: game.runners.clone(),
            },
            PitchOutcome::Flyout { fielder } => Event::Flyout {
                fielder,
                runners_after: game.runners.clone(),
            },
            PitchOutcome::HomeRun => Event::HomeRun,

            // todo: there may be a subtle bug here since we don't sweep the runners after the forced advance
            // runner [1, 0], double, then we're at [3, 2], 3 *should* get swept and *then* 2 should get to advance to 3...
            PitchOutcome::Triple => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(3);
                new_runners.advance_if(|_runner| rng.next() < 0.5);
                Event::BaseHit {
                    bases: 3,
                    runners_after: new_runners,
                }
            }

            PitchOutcome::Double => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(2);
                new_runners.advance_if(|_runner| rng.next() < 0.5);
                Event::BaseHit {
                    bases: 2,
                    runners_after: new_runners,
                }
            }

            PitchOutcome::Single => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(1);
                new_runners.advance_if(|_runner| rng.next() < 0.5);
                Event::BaseHit {
                    bases: 1,
                    runners_after: new_runners,
                }
            }
        })
    }
}

fn do_pitch(world: &World, game: &Game, rng: &mut Rng) -> PitchOutcome {
    let pitcher = world.player(game.pitching_team().pitcher);
    let batter = world.player(game.batting_team().batter.unwrap());

    let is_flinching = game.strikes == 0 && batter.mods.has(Mod::Flinch);

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

    let out_defender_id = game.pick_fielder(world, rng.next());
    let out_defender = world.player(out_defender_id);

    let is_out = rng.next() < formulas::out_threshold(pitcher, batter, out_defender);
    if is_out {
        let fly_defender_id = game.pick_fielder(world, rng.next());
        let fly_defender = world.player(out_defender_id);

        let is_fly = rng.next() < formulas::fly_threshold(fly_defender);
        if is_fly {
            return PitchOutcome::Flyout {
                fielder: fly_defender_id,
            };
        }

        let ground_defender_id = game.pick_fielder(world, rng.next());
        return PitchOutcome::GroundOut {
            fielder: ground_defender_id,
        };
    }

    let is_hr = rng.next() < formulas::hr_threshold(pitcher, batter);
    if is_hr {
        return PitchOutcome::HomeRun;
    }

    let hit_defender_id = game.pick_fielder(world, rng.next());
    let hit_defender = world.player(hit_defender_id);
    let double_roll = rng.next();
    let triple_roll = rng.next();

    if triple_roll < formulas::triple_threshold(pitcher, batter, hit_defender) {
        return PitchOutcome::Triple;
    }
    if double_roll < formulas::double_threshold(pitcher, batter, hit_defender) {
        return PitchOutcome::Double;
    }

    PitchOutcome::Single
}

struct BatterStatePlugin;
impl Plugin for BatterStatePlugin {
    fn tick(&self, game: &Game, world: &World, _rng: &mut Rng) -> Option<Event> {
        let batting_team = game.batting_team();
        if game.batting_team().batter.is_none() {
            let idx = batting_team.batter_index;
            let team = world.team(batting_team.id);
            let batter = team.lineup[idx % team.lineup.len()].clone();
            Some(Event::BatterUp { batter })
        } else {
            None
        }
    }
}

struct InningStatePlugin;
impl Plugin for InningStatePlugin {
    fn tick(&self, game: &Game, _world: &World, _rng: &mut Rng) -> Option<Event> {
        if game.outs < 3 {
            return None;
        }

        let tied = (game.away_team.score - game.home_team.score).abs() < 0.01; // lol floats
        if game.inning >= 9 && !tied {
            return Some(Event::GameOver);
        }

        if game.top {
            Some(Event::InningSwitch {
                inning: game.inning,
                top: false,
            })
        } else {
            Some(Event::InningSwitch {
                inning: game.inning + 1,
                top: true,
            })
        }
    }
}

struct StealingPlugin;
impl Plugin for StealingPlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        let steal_defender_id = game.pick_fielder(world, rng.next());
        let steal_defender = world.player(steal_defender_id);

        // todo: can we refactor `Baserunners` in a way where this sort of iteration is more natural
        for base in (0..4).rev() {
            if let Some(runner_id) = game.runners.at(base) {
                if game.runners.can_advance(base) {
                    let runner = world.player(runner_id);
                    let should_attempt =
                        rng.next() < formulas::steal_attempt_threshold(runner, steal_defender);
                    if should_attempt {
                        let success =
                            rng.next() < formulas::steal_success_threshold(runner, steal_defender);

                        if success {
                            return Some(Event::BaseSteal {
                                runner: runner_id,
                                base_from: base,
                                base_to: base + 1,
                            });
                        } else {
                            return Some(Event::CaughtStealing {
                                runner: runner_id,
                                base_from: base,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}
