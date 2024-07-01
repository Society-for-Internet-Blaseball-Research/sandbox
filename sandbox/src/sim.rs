use uuid::Uuid;

use crate::{bases::Baserunners, entities::{World, Player}, formulas, mods::Mod, rng::Rng, Game, Weather};

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
                Box::new(ExtraWeatherPlugin),
                Box::new(BatterStatePlugin),
                Box::new(WeatherPlugin),
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
        reverberating: bool
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
    DoublePlay {
        runners_after: Baserunners
    },
    FieldersChoice {
        runners_after: Baserunners
    },

    BaseSteal {
        runner: Uuid,
        base_from: u8,
        base_to: u8,
    },
    CaughtStealing {
        runner: Uuid,
        base_from: u8,
    },

    Incineration {
        target: Uuid,
        replacement: Player
    },
    Peanut {
        target: Uuid,
        yummy: bool
    },
    Feedback {
        target1: Uuid,
        target2: Uuid,
    },
    Reverb {
        reverb_type: u8,
        team: Uuid,
        changes: Vec<usize>
    },
    Blooddrain {
        drainer: Uuid,
        target: Uuid,
        stat: u8,
        siphon: bool,
        siphon_effect: i16
    },
    Sun2 {
        home_team: bool,
    },
    BlackHole {
        home_team: bool,
    },
    Salmon {
        home_runs_lost: bool,
        away_runs_lost: bool
    },
    PolaritySwitch,
    NightShift {
        batter: bool,
        replacement: Uuid,
        replacement_idx: usize,
        boosts: Vec<f64>
    },
    Fireproof {
        target: Uuid,
    },
    Soundproof {
        resists: Uuid,
        tangled: Uuid
    }
}

impl Event {
    pub fn apply(&self, game: &mut Game, world: &mut World) {
        match *self {
            Event::BatterUp { batter, reverberating } => {
                let bt = game.batting_team_mut();
                if reverberating {
                    bt.batter_index -= 1;
                }
                bt.batter = Some(batter);
            }
            Event::InningSwitch { inning, top } => {
                if let Weather::Salmon = game.weather {
                    if game.top {
                        let runs_away = game.away_team.score - game.linescore_away[0];
                        game.linescore_away.push(runs_away);
                        game.linescore_away[0] += runs_away;
                    } else {
                        let runs_home = game.home_team.score - game.linescore_home[0];
                        game.linescore_home.push(runs_home);
                        game.linescore_home[0] += runs_home;
                    }
                }
                game.inning = inning;
                game.top = top;
                game.outs = 0;
                game.balls = 0;
                game.strikes = 0;
                game.events_inning = 0;
                game.scoring_plays_inning = 0;
                game.runners = Baserunners::new();
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
                game.strikes = game.strikes.min(2); //todo: kid named fourth strike
            }
            Event::Strikeout => {
                game.outs += 1;
                game.end_pa();
            }
            Event::Walk => {
                // maybe we should put batter in the event
                // todo: make a function that returns the current batter
                game.runners.walk();
                game.runners.add(0, game.batting_team().batter.unwrap());
                game.base_sweep();
                game.end_pa();
            }
            Event::HomeRun => {
                let no_runners_on = game.runners.empty();
                game.runners.advance_all(4);
                game.batting_team_mut().score += game.get_run_value(); //lazy workaround to score the home run hitter
                game.base_sweep();
                if no_runners_on {
                    game.scoring_plays_inning += 1;
                } //this is to make sum sun not break
                game.end_pa();
            }
            Event::BaseHit {
                bases,
                ref runners_after,
            } => {
                game.runners = runners_after.clone();
                game.base_sweep();
                game.runners
                    .add(bases - 1, game.batting_team().batter.unwrap());
                game.end_pa();
            }
            Event::GroundOut {
                fielder: _fielder,
                ref runners_after,
            } => {
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::Flyout {
                fielder: _fielder,
                ref runners_after,
            } => {
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::DoublePlay { ref runners_after } => {
                game.outs += 2;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::FieldersChoice { ref runners_after } => {
                game.outs += 1;
                game.runners = runners_after.clone();
                game.runners.add(0, game.batting_team().batter.unwrap());
                game.base_sweep();
                game.end_pa();
            }
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
            },
            Event::Incineration { target, ref replacement } => {
                let replacement_id = world.add_rolled_player(replacement.clone(), world.player(target).team.unwrap());
                if let Some(batter) = game.batting_team().batter {
                    if batter == target {
                        game.batting_team_mut().batter = Some(replacement_id);
                    }
                } else if target == game.pitching_team().pitcher {
                    game.pitching_team_mut().pitcher = replacement_id;
                }
                world.replace_player(target, replacement_id);
            },
            Event::Peanut { target, yummy } => {
                let coeff = if yummy {
                    0.2
                } else {
                    -0.2
                };
                let boosts: Vec<f64> = vec![coeff; 26];
                let player = world.player_mut(target);
                player.boost(&boosts);
            },
            Event::Feedback { target1, target2 } => {
                if let Some(batter) = game.batting_team().batter {
                    if batter == target1 {
                        game.batting_team_mut().batter = Some(target2);
                    } else {
                        game.pitching_team_mut().pitcher = target2;
                    }
                }
                world.swap(target1, target2);
            },
            Event::Reverb { reverb_type, team, ref changes } => {
                world.team_mut(team).apply_reverb_changes(reverb_type, changes);
                if game.batting_team().id == team && reverb_type != 3 {
                    let idx = game.batting_team().batter_index;
                    let world_team = world.team(team);
                    let new_batter = world_team.lineup[idx % world_team.lineup.len()].clone();
                    game.batting_team_mut().batter = Some(new_batter);
                } else if game.pitching_team().id == team && reverb_type != 2 {
                    game.pitching_team_mut().pitcher = world.team(team).rotation[0].clone();
                } else if game.batting_team().id == team && reverb_type != 2 {
                    game.batting_team_mut().pitcher = world.team(team).rotation[0].clone();
                }
            },
            Event::Blooddrain { drainer, target, stat, .. } => {
                let drainer_mut = world.player_mut(drainer);
                let mut boosts: Vec<f64> = vec![0.0; 26];
                match stat {
                    0 => {
                        //pitching
                        for i in 8..14 {
                            boosts[i] = 0.1;
                        }
                    },
                    1 => {
                        //batting
                        for i in 0..8 {
                            boosts[i] = 0.1;
                        }
                    },
                    2 => {
                        //defense
                        for i in 19..24 {
                            boosts[i] = 0.1;
                        }
                    },
                    3 => {
                        //baserunning
                        for i in 14..19 {
                            boosts[i] = 0.1;
                        }
                    },
                    _ => {
                    }
                }
                drainer_mut.boost(&boosts);

                let target_mut = world.player_mut(target);
                let mut decreases: Vec<f64> = vec![0.0; 26];
                match stat {
                    0 => {
                        for i in 8..14 {
                            decreases[i] = -0.1;
                        }
                    },
                    1 => {
                        for i in 0..8 {
                            decreases[i] = -0.1;
                        }
                    },
                    2 => {
                        for i in 19..24 {
                            decreases[i] = -0.1;
                        }
                    },
                    3 => {
                        for i in 14..19 {
                            decreases[i] = -0.1;
                        }
                    },
                    _ => {
                    }
                }
                target_mut.boost(&decreases);
            },
            //todo: add win manipulation when we actually have wins
            Event::Sun2 { home_team } => {
                if home_team {
                    game.home_team.score -= 10.0;
                } else {
                    game.away_team.score -= 10.0;
                }
            }
            Event::BlackHole { home_team } => {
                if home_team {
                    game.home_team.score -= 10.0;
                } else {
                    game.away_team.score -= 10.0;
                }
            },
            Event::Salmon { home_runs_lost, away_runs_lost } => {
                if game.last_salmon_inning != game.inning - 1 {
                    game.salmon_resets_inning = 0;
                }
                if away_runs_lost {
                    //this whole exercise's goal is
                    //to find the first instance of the inning
                    game.away_team.score -= game.linescore_away[game.linescore_away.len() - 1 - (game.salmon_resets_inning as usize)];
                }
                if home_runs_lost {
                    game.home_team.score -= game.linescore_home[game.linescore_home.len() - 1 - (game.salmon_resets_inning as usize)];
                }
                if !game.top {
                    game.top = true
                } else {
                    game.inning -= 1;
                }
                game.salmon_resets_inning += 1;
                game.last_salmon_inning = game.inning;
            },
            Event::PolaritySwitch => {
                game.polarity = !game.polarity;
            },
            Event::NightShift { batter, replacement, replacement_idx, ref boosts } => {
                if batter {
                    let team = game.batting_team();
                    let active_batter = team.batter.unwrap();
                    let active_batter_order = team.batter_index % world.team(team.id).lineup.len();
                    world.team_mut(team.id).lineup[active_batter_order] = replacement;
                    world.team_mut(team.id).shadows[replacement_idx] = active_batter;
                    world.player_mut(replacement).boost(boosts);
                    let team_mut = game.batting_team_mut();
                    team_mut.batter = Some(replacement);
                } else {
                    let team = game.pitching_team();
                    let active_pitcher = team.pitcher;
                    let active_pitcher_idx = 0; //todo: this only works for one game
                    world.team_mut(team.id).rotation[active_pitcher_idx] = replacement;
                    world.team_mut(team.id).shadows[replacement_idx] = active_pitcher;
                    world.player_mut(replacement).boost(boosts);
                    let team_mut = game.pitching_team_mut();
                    team_mut.pitcher = replacement;
                }
            },
            Event::Fireproof { target: _target } => {},
            Event::Soundproof { resists: _resists, tangled: _tangled } => {} //todo: no idea how to handle tangled
        }
    }
}

enum PitchOutcome {
    Ball,
    StrikeSwinging,
    StrikeLooking,
    Foul,
    GroundOut {
        fielder: Uuid,
        advancing_runners: Vec<Uuid>
    },
    Flyout { 
        fielder: Uuid,
        advancing_runners: Vec<Uuid>
    },
    DoublePlay { runner_out: u8 },
    FieldersChoice { runner_out: u8 },
    HomeRun,
    Triple { advancing_runners: Vec<Uuid> },
    Double { advancing_runners: Vec<Uuid> },
    Single { advancing_runners: Vec<Uuid> },
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
            PitchOutcome::GroundOut { fielder, advancing_runners } => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_if(|runner| advancing_runners.contains(&runner.id));
                Event::GroundOut {
                    fielder,
                    runners_after: new_runners,
                }
            },
            PitchOutcome::Flyout { fielder, advancing_runners } => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_if(|runner| advancing_runners.contains(&runner.id));
                Event::Flyout {
                    fielder,
                    runners_after: new_runners,
                }
            },
            PitchOutcome::DoublePlay { runner_out } => {
                let mut new_runners = game.runners.clone();
                new_runners.remove(runner_out);
                new_runners.advance_all(1);
                Event::DoublePlay {
                    runners_after: new_runners
                }
            },
            PitchOutcome::FieldersChoice { runner_out } => {
                let mut new_runners = game.runners.clone();
                new_runners.remove(runner_out);
                new_runners.advance_all(1);
                Event::FieldersChoice {
                    runners_after: new_runners
                }
            },

            PitchOutcome::HomeRun => Event::HomeRun,

            // todo: there may be a subtle bug here since we don't sweep the runners after the forced advance
            // runner [1, 0], double, then we're at [3, 2], 3 *should* get swept and *then* 2 should get to advance to 3...
            PitchOutcome::Triple { advancing_runners }=> {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(3);
                new_runners.advance_if(|runner| advancing_runners.contains(&runner.id));
                Event::BaseHit {
                    bases: 3,
                    runners_after: new_runners,
                }
            }

            PitchOutcome::Double { advancing_runners } => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(2);
                new_runners.advance_if(|runner| advancing_runners.contains(&runner.id));
                Event::BaseHit {
                    bases: 2,
                    runners_after: new_runners,
                }
            }

            PitchOutcome::Single { advancing_runners } => {
                let mut new_runners = game.runners.clone();
                new_runners.advance_all(1);
                new_runners.advance_if(|runner| advancing_runners.contains(&runner.id));
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

    let is_out = rng.next() > formulas::out_threshold(pitcher, batter, out_defender);
    if is_out {
        let fly_defender_id = game.pick_fielder(world, rng.next());
        let fly_defender = world.player(out_defender_id); //is this correct?

        let is_fly = rng.next() < formulas::fly_threshold(fly_defender);
        if is_fly {
            let mut advancing_runners = Vec::new();
            for baserunner in game.runners.iter() {
                let base_from = baserunner.base;
                let runner_id = baserunner.id.clone();
                let runner = world.player(runner_id);

                if rng.next() < formulas::flyout_advancement_threshold(runner, base_from) {
                    advancing_runners.push(runner_id);
                }
            }
            return PitchOutcome::Flyout {
                fielder: fly_defender_id,
                advancing_runners
            };
        }

        let ground_defender_id = game.pick_fielder(world, rng.next());
        let mut advancing_runners = Vec::new();

        if !game.runners.empty() {
            let dp_roll = rng.next();
            if game.runners.occupied(0) {
                if game.outs < 2 && dp_roll < formulas::double_play_threshold(batter, pitcher, out_defender) {
                    return PitchOutcome::DoublePlay {
                        runner_out: game.runners.pick_runner(rng.next())
                    };
                } else {
                    let sac_roll = rng.next();
                    if sac_roll < formulas::groundout_sacrifice_threshold(batter) {
                        for baserunner in game.runners.iter() {
                            let runner_id = baserunner.id.clone();
                            let runner = world.player(runner_id);
                            if rng.next() < formulas::groundout_advancement_threshold(runner, out_defender) {
                                advancing_runners.push(runner_id);
                            }
                        }
                        return PitchOutcome::GroundOut {
                            fielder: ground_defender_id,
                            advancing_runners
                        };
                    } else {
                        return PitchOutcome::FieldersChoice {
                            runner_out: game.runners.pick_runner_fc()
                        }
                    }
                }
            }
            for baserunner in game.runners.iter() {
                let runner_id = baserunner.id.clone();
                let runner = world.player(runner_id);
                if rng.next() < formulas::groundout_advancement_threshold(runner, out_defender) {
                    advancing_runners.push(runner_id);
                }
            }
        }
        return PitchOutcome::GroundOut {
            fielder: ground_defender_id,
            advancing_runners
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

    let mut advancing_runners = Vec::new();
    for baserunner in game.runners.iter() {
        let runner_id = baserunner.id.clone();
        let runner = world.player(runner_id);

        if rng.next() < formulas::hit_advancement_threshold(runner, hit_defender) {
            advancing_runners.push(runner_id);
        }
    }

    if triple_roll < formulas::triple_threshold(pitcher, batter, hit_defender) {
        return PitchOutcome::Triple {
            advancing_runners
        };
    }
    if double_roll < formulas::double_threshold(pitcher, batter, hit_defender) {
        return PitchOutcome::Double {
            advancing_runners
        };
    }

    PitchOutcome::Single {
        advancing_runners
    }
}

struct BatterStatePlugin;
impl Plugin for BatterStatePlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        let batting_team = game.batting_team();
        if game.batting_team().batter.is_none() {
            let idx = batting_team.batter_index;
            let team = world.team(batting_team.id);
            let game_begin = idx == 0;
            let inning_begin = game.events_inning == 0;
            let prev = if game_begin { team.lineup[0].clone() } else { team.lineup[(idx - 1) % team.lineup.len()].clone() };
            if !inning_begin && world.player(prev).mods.has(Mod::Reverberating) && rng.next() < 0.2 { //rough estimate
                return Some(Event::BatterUp { batter: prev, reverberating: true });
            }
            let batter = team.lineup[idx % team.lineup.len()].clone();
            Some(Event::BatterUp { batter, reverberating: false })
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

        let lead = if (game.away_team.score - game.home_team.score).abs() < 0.01 {
            0
        } else if game.away_team.score > game.home_team.score {
            1
        } else {
            -1
        }; // lol floats
        if game.inning >= 9 && (lead == -1 || !game.top && lead == 1) {
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

struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        match game.weather {
            Weather::Sun => None,
            Weather::Eclipse => {
                //todo: add fortification
                if rng.next() < 0.00045 {
                    let target = game.pick_player_weighted(world, rng.next(), |uuid| if game.runners.contains(uuid) { 0.0 } else { 1.0 }, true);
                    if world.player(target).mods.has(Mod::Fireproof) {
                        return Some(Event::Fireproof { target });
                    }
                    let replacement = Player::new(rng);
                    return Some(Event::Incineration { 
                        target,
                        replacement
                    });
                }
                None
            },
            Weather::Peanuts => {
                if rng.next() < 0.0006 {
                    //idk if runners can have a reaction
                    //but this is assuming it's the same as incins
                    let target = game.pick_player_weighted(world, rng.next(), |uuid| if game.runners.contains(uuid) { 0.0 } else { 1.0 }, true);
                    return Some(Event::Peanut {
                        target,
                        yummy: false
                    });
                }
                None
            },
            Weather::Birds => None, //lol
            Weather::Feedback => {
                let is_batter = rng.next() < (9.0 / 14.0);
                if rng.next() < 0.0001 {
                    if is_batter {
                        let target1 = game.batting_team().batter.unwrap();
                        let target2 = game.pick_fielder(world, rng.next());
                        if world.player(target1).mods.has(Mod::Soundproof) {
                            return Some(Event::Soundproof {
                                resists: target1,
                                tangled: target2
                            });
                        } else if world.player(target2).mods.has(Mod::Soundproof) {
                            return Some(Event::Soundproof {
                                resists: target2,
                                tangled: target1
                            });
                        }
                        return Some(Event::Feedback {
                            target1: game.batting_team().batter.unwrap(),
                            target2
                        });
                    }

                    let target1 = game.pitching_team().pitcher;
                    let batting_team = world.team(game.batting_team().id);
                    let idx = (rng.next() * (batting_team.rotation.len() as f64)).floor() as usize;
                    let target2 = batting_team.rotation[idx];
                    if world.player(target1).mods.has(Mod::Soundproof) {
                        return Some(Event::Soundproof {
                            resists: target1,
                            tangled: target2
                        });
                    } else if world.player(target2).mods.has(Mod::Soundproof) {
                        return Some(Event::Soundproof {
                            resists: target2,
                            tangled: target1
                        });
                    }
                    return Some(Event::Feedback {
                        target1: game.pitching_team().pitcher,
                        target2
                    });
                }
                None
            },
            Weather::Reverb => {
                //estimate
                if rng.next() < 0.00008 {
                    let reverb_type_roll = rng.next();
                    let reverb_type = if reverb_type_roll < 0.09 {
                        0u8
                    } else if reverb_type_roll < 0.55 {
                        1u8
                    } else if reverb_type_roll < 0.95 {
                        2u8
                    } else {
                        3u8
                    };
                    let team_id = if rng.next() < 0.5 {
                        game.home_team.id
                    } else {
                        game.away_team.id
                    };

                    let changes = world.team(team_id.clone()).roll_reverb_changes(rng, reverb_type);
                    
                    return Some(Event::Reverb {
                        reverb_type,
                        team: team_id,
                        changes
                    });
                }
                None
            },
            Weather::Blooddrain => {
                if rng.next() < 0.00065 {
                    let fielding_team_drains = rng.next() < 0.5;
                    let is_atbat = rng.next() < 0.5;
                    if is_atbat {
                        if fielding_team_drains {
                            return Some(Event::Blooddrain {
                                drainer: game.pitching_team().pitcher,
                                target: game.batting_team().batter.unwrap(),
                                stat: (rng.next() * 4.0).floor() as u8,
                                siphon: false,
                                siphon_effect: -1
                            });
                        } else {
                            return Some(Event::Blooddrain {
                                drainer: game.batting_team().batter.unwrap(),
                                target: game.pitching_team().pitcher,
                                stat: (rng.next() * 4.0).floor() as u8,
                                siphon: false,
                                siphon_effect: -1
                            });
                        }
                    } else {
                        let fielder_roll = rng.next();
                        let fielder = game.pick_fielder(world, fielder_roll);
                        let hitter = if game.runners.empty() {
                            game.batting_team().batter.unwrap()
                        } else {
                            game.pick_player_weighted(world, rng.next(), |uuid| if uuid == game.batting_team().batter.unwrap() || game.runners.contains(uuid) { 1.0 } else { 0.0 }, true)
                        };
                        if fielding_team_drains {
                            return Some(Event::Blooddrain {
                                drainer: fielder,
                                target: hitter,
                                stat: (rng.next() * 4.0).floor() as u8,
                                siphon: false,
                                siphon_effect: -1
                            });
                        } else {
                            return Some(Event::Blooddrain {
                                drainer: hitter,
                                target: fielder,
                                stat: (rng.next() * 4.0).floor() as u8,
                                siphon: false,
                                siphon_effect: -1
                            });
                        }
                    }
                }
                None
            },
            Weather::Sun2 => {
                if game.home_team.score - 10.0 >= -0.001 { //ugh
                    Some(Event::Sun2 { home_team: true })
                } else if game.away_team.score - 10.0 >= -0.001 {
                    Some(Event::Sun2 { home_team: false })
                } else {
                    None
                }
            },
            Weather::BlackHole => {
                if game.home_team.score - 10.0 >= -0.001 {
                    Some(Event::BlackHole { home_team: true })
                } else if game.away_team.score - 10.0 >= -0.001 {
                    Some(Event::BlackHole { home_team: false })
                } else {
                    None
                }
            },
            Weather::Salmon => None,
            Weather::PolarityPlus | Weather::PolarityMinus => {
                if rng.next() < 0.035 {
                    return Some(Event::PolaritySwitch);
                }
                None
            },
            Weather::SunPointOne | Weather::SumSun => None,
            Weather::Night => {
                if rng.next() < 0.0035 { //estimate
                    if rng.next() < 0.5 {
                        let shadows = &world.team(game.batting_team().id).shadows;
                        let replacement_idx = (rng.next() * shadows.len() as f64).floor() as usize;
                        let replacement = shadows[replacement_idx as usize];
                        let mut boosts: Vec<f64> = Vec::new();
                        for _ in 0..26 {
                            boosts.push(rng.next() * 0.2);
                        } //I think this code might be better written
                        return Some(Event::NightShift { batter: true, replacement, replacement_idx, boosts });
                    }
                    let shadows = &world.team(game.pitching_team().id).shadows;
                    let replacement_idx = (rng.next() * shadows.len() as f64).floor() as usize;
                    let replacement = shadows[replacement_idx as usize];
                    let mut boosts: Vec<f64> = Vec::new();
                    for _ in 0..26 {
                        boosts.push(rng.next() * 0.2);
                    }
                    return Some(Event::NightShift { batter: false, replacement, replacement_idx, boosts });
                }
                None
            }
        }
    }
}

struct ExtraWeatherPlugin;
impl Plugin for ExtraWeatherPlugin {
    fn tick(&self, game: &Game, _world: &World, rng: &mut Rng) -> Option<Event> {
        if let Weather::Salmon = game.weather {
            let away_team_scored = game.linescore_away.last().unwrap().abs() > 0.01;
            let home_team_scored = if !game.top { false } else { game.linescore_home.last().unwrap().abs() > 0.01 };
            if game.events_inning == 0 && (away_team_scored || home_team_scored) {
                let salmon_activated = rng.next() < 0.1375;
                if salmon_activated {
                    let runs_lost = rng.next() < 0.675; //rough estimate
                    if runs_lost {
                        if away_team_scored && home_team_scored {
                            let double_runs_lost = rng.next() < 0.2; //VERY rough estimate
                            if double_runs_lost {
                                return Some(Event::Salmon { away_runs_lost: true, home_runs_lost: true });
                            }
                            let home_runs_lost = rng.next() < 0.5;
                            return Some(Event::Salmon { away_runs_lost: !home_runs_lost, home_runs_lost });
                        }
                        if away_team_scored {
                            return Some(Event::Salmon { away_runs_lost: true, home_runs_lost: false });
                        }
                        return Some(Event::Salmon { away_runs_lost: false, home_runs_lost: true });
                    }
                    return Some(Event::Salmon { away_runs_lost: false, home_runs_lost: false });
                }
            }
            return None;
        }
        None
    }
}
