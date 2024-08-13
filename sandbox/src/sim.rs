use uuid::Uuid;

use crate::{entities::{World, Player}, events::Event, formulas, mods::Mod, rng::Rng, Game, Weather};

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
                Box::new(ModPlugin),
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
        let max_strikes = get_max_strikes(game, world);
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

//can we make this a Game instance method? probably not
pub fn get_max_strikes(game: &Game, world: &World) -> i16 {
    let batter = world.player(game.batting_team().batter.unwrap());
    if batter.mods.has(Mod::FourthStrike) {
        4
    } else {
        3
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
            let first_batter = if game.events.len() == 0 {
                true
            } else if idx == 0 && game.inning == 1 && game.events.last() == "inningSwitch" {
                true
            } else {
                false
            };
            let inning_begin = !first_batter && game.events.last() == "inningSwitch";
            let prev = if first_batter { team.lineup[0].clone() } else { team.lineup[(idx - 1) % team.lineup.len()].clone() };
            //todo: improve this
            if !first_batter && !inning_begin && world.player(prev).mods.has(Mod::Reverberating) && rng.next() < 0.2 { //rough estimate
                return Some(Event::BatterUp { batter: prev, reverberating: true });
            }
            let batter = team.lineup[idx % team.lineup.len()].clone();
            if world.player(batter).mods.has(Mod::Shelled) {
                return Some(Event::Shelled { batter });
            }
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

fn poll_for_mod(game: &Game, world: &World, a_mod: Mod, only_current: bool) -> Vec<Uuid> {
    let home_team = &game.home_team;
    let away_team = &game.away_team;

    let home_lineup = world.team(home_team.id).lineup.clone();
    let home_pitcher = if only_current { vec![home_team.pitcher.clone()] } else { world.team(home_team.id).rotation.clone() };
    let away_lineup = world.team(away_team.id).lineup.clone();
    let away_pitcher = if only_current { vec![away_team.pitcher.clone()] } else { world.team(away_team.id).rotation.clone() };

    let mut players = vec![home_lineup, home_pitcher, away_lineup, away_pitcher].concat();

    players.retain(|player| world.player(*player).mods.has(a_mod));

    players
}

struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        match game.weather {
            Weather::Sun => None,
            Weather::Eclipse => {
                //todo: add fortification
                let incin_roll = rng.next();
                let target = game.pick_player_weighted(world, rng.next(), |uuid| if game.runners.contains(uuid) { 0.0 } else { 1.0 }, true);
                if world.player(target).mods.has(Mod::Unstable) && incin_roll < 0.002 { //estimate
                    if world.player(target).mods.has(Mod::Fireproof) {
                        return Some(Event::Fireproof { target });
                    }
                    let chain_target = game.pick_player_weighted(world, rng.next(), |uuid| if world.player(uuid).team.unwrap() == world.player(target).team.unwrap() {
                        0.0
                    } else {
                        1.0
                    }, false);
                    let replacement = Player::new(rng); 
                    let chain = if world.player(chain_target).mods.has(Mod::Stable) { None } else { Some(chain_target) };//assumption
                    Some(Event::IncinerationWithChain { 
                        target,
                        replacement,
                        chain
                    })
                } else if incin_roll < 0.00045 {
                    if world.player(target).mods.has(Mod::Fireproof) {
                        return Some(Event::Fireproof { target });
                    }
                    let replacement = Player::new(rng);
                    Some(Event::Incineration { 
                        target,
                        replacement
                    })
                } else {
                    None
                }
            },
            Weather::Peanuts => {
                if rng.next() < 0.0006 {
                    //idk if runners can have a reaction
                    //but this is assuming it's the same as incins
                    let target = game.pick_player_weighted(world, rng.next(), |uuid| if game.runners.contains(uuid) { 0.0 } else { 1.0 }, true);
                    Some(Event::Peanut {
                        target,
                        yummy: false
                    })
                } else {
                    None
                }
            },
            Weather::Birds => {
                //rough estimate
                if rng.next() < 0.003 {
                    return Some(Event::Birds);
                } //todo: figure out what order these events go in
                
                let shelled_players = poll_for_mod(game, world, Mod::Shelled, false);
                for player in shelled_players {
                    //estimate, not sure how accurate this is
                    if rng.next() < 0.00015 {
                        return Some(Event::PeckedFree { player });
                    }
                }
                None
            },
            Weather::Feedback => {
                let is_batter = rng.next() < (9.0 / 14.0);
                let feedback_roll = rng.next();
                let batter = game.batting_team().batter.unwrap();
                let pitcher = game.pitching_team().pitcher;

                let mut target1_opt = None;
                let mut target2_opt = None;

                if is_batter && world.player(batter).mods.has(Mod::Flickering) && feedback_roll < 0.02 {
                    let target2_raw = game.pick_fielder(world, rng.next());
                    
                    target1_opt = Some(batter);
                    target2_opt = Some(target2_raw);
                } else if !is_batter && world.player(pitcher).mods.has(Mod::Flickering) && feedback_roll < 0.02 {
                    let batting_team = world.team(game.batting_team().id);
                    let idx = (rng.next() * (batting_team.rotation.len() as f64)).floor() as usize;
                    let target2_raw = batting_team.rotation[idx];

                    target1_opt = Some(pitcher);
                    target2_opt = Some(target2_raw);
                } else if feedback_roll < 0.0001 {
                    if is_batter {
                        let target2_raw = game.pick_fielder(world, rng.next());
                        
                        target1_opt = Some(batter);
                        target2_opt = Some(target2_raw);
                    } else {
                        let batting_team = world.team(game.batting_team().id);
                        let idx = (rng.next() * (batting_team.rotation.len() as f64)).floor() as usize;
                        let target2_raw = batting_team.rotation[idx];
                        
                        target1_opt = Some(pitcher);
                        target2_opt = Some(target2_raw);
                    }
                }
                if target1_opt.is_some() {
                    let target1 = target1_opt.unwrap();
                    let target2 = target2_opt.unwrap();
                    if world.player(target1).mods.has(Mod::Soundproof) {
                        let decreases = roll_random_boosts(rng, -0.05);
                        Some(Event::Soundproof {
                            resists: target1,
                            tangled: target2,
                            decreases
                        })
                    } else if world.player(target2).mods.has(Mod::Soundproof) {
                        let decreases = roll_random_boosts(rng, -0.05);
                        Some(Event::Soundproof {
                            resists: target2,
                            tangled: target1,
                            decreases
                        })
                    } else {
                        Some(Event::Feedback {
                            target1,
                            target2
                        })
                    }
                } else {
                    None
                }
            },
            Weather::Reverb => {
                //estimate
                if rng.next() < 0.00003 {
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

                    let mut gravity_players: Vec<usize> = vec![];

                    let team = world.team(team_id.clone());

                    for i in 0..team.lineup.len() {
                        if world.player(team.lineup[i]).mods.has(Mod::Gravity) {
                            gravity_players.push(i);
                        }
                    }
                    for i in 0..team.rotation.len() {
                        if world.player(team.rotation[i]).mods.has(Mod::Gravity) {
                            gravity_players.push(i + team.lineup.len());
                        }
                    } //todo: make this prettier

                    let changes = team.roll_reverb_changes(rng, reverb_type, &gravity_players);
                    
                    Some(Event::Reverb {
                        reverb_type,
                        team: team_id,
                        changes
                    })
                } else {
                    None
                }
            },
            Weather::Blooddrain => {
                if rng.next() < 0.00065 {
                    let fielding_team_drains = rng.next() < 0.5;
                    let is_atbat = rng.next() < 0.5;
                    if is_atbat {
                        Some(Event::Blooddrain {
                            drainer: if fielding_team_drains { game.pitching_team().pitcher } else { game.batting_team().batter.unwrap() },
                            target: if fielding_team_drains { game.batting_team().batter.unwrap() } else { game.pitching_team().pitcher },
                            stat: (rng.next() * 4.0).floor() as u8,
                            siphon: false,
                            siphon_effect: -1
                        })
                    } else {
                        let fielder_roll = rng.next();
                        let fielder = game.pick_fielder(world, fielder_roll);
                        let hitter = if game.runners.empty() {
                            game.batting_team().batter.unwrap()
                        } else {
                            game.pick_player_weighted(world, rng.next(), |uuid| if uuid == game.batting_team().batter.unwrap() || game.runners.contains(uuid) { 1.0 } else { 0.0 }, true)
                        };
                        Some(Event::Blooddrain {
                            drainer: if fielding_team_drains { fielder } else { hitter },
                            target: if fielding_team_drains { hitter } else { fielder },
                            stat: (rng.next() * 4.0).floor() as u8,
                            siphon: false,
                            siphon_effect: -1
                        })
                    }
                } else {
                    None
                }
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
                    Some(Event::PolaritySwitch)
                } else {
                    None
                }
            },
            Weather::SunPointOne | Weather::SumSun => None,
            Weather::Night => {
                if rng.next() < 0.01 { //estimate
                    let batter = rng.next() < 0.5;
                    let shadows = if batter { &world.team(game.batting_team().id).shadows } else { &world.team(game.pitching_team().id).shadows };
                    let replacement_idx = (rng.next() * shadows.len() as f64).floor() as usize;
                    let replacement = shadows[replacement_idx as usize];
                    let boosts = roll_random_boosts(rng, 0.2);
                    Some(Event::NightShift { batter, replacement, replacement_idx, boosts })
                } else {
                    None
                }
            }
        }
    }
}

fn roll_random_boosts(rng: &mut Rng, threshold: f64) -> Vec<f64> {
    let mut boosts: Vec<f64> = Vec::new();
    for _ in 0..26 {
        boosts.push(rng.next() * threshold);
    }
    boosts
}

struct ExtraWeatherPlugin;
impl Plugin for ExtraWeatherPlugin {
    fn tick(&self, game: &Game, _world: &World, rng: &mut Rng) -> Option<Event> {
        if let Weather::Salmon = game.weather {
            let away_team_scored = game.linescore_away.last().unwrap().abs() > 0.01;
            let home_team_scored = if !game.top { false } else { game.linescore_home.last().unwrap().abs() > 0.01 };
            if game.events.len() > 0 && game.events.last() == "inningSwitch" && (away_team_scored || home_team_scored) {
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

struct ModPlugin;
impl Plugin for ModPlugin {
    fn tick(&self, game: &Game, world: &World, rng: &mut Rng) -> Option<Event> {
        let batter = game.batting_team().batter.unwrap();
        let batter_mods = &world.player(batter).mods;
        let pitcher = game.pitching_team().pitcher;
        let pitcher_mods = &world.player(pitcher).mods;
        if batter_mods.has(Mod::Electric) && game.strikes > 0 && rng.next() < 0.2 {
            Some(Event::Zap { batter: true })
        } else if pitcher_mods.has(Mod::Electric) && game.balls > 0 && rng.next() < 0.2 {
            Some(Event::Zap { batter: false })
        } else if pitcher_mods.has(Mod::DebtU) && !batter_mods.has(Mod::Unstable) && rng.next() < 0.02 { //estimate
            Some(Event::HitByPitch { target: batter, hbp_type: 0 })
        } else if pitcher_mods.has(Mod::RefinancedDebt) && !batter_mods.has(Mod::Flickering) && rng.next() < 0.02 { //estimate
            Some(Event::HitByPitch { target: batter, hbp_type: 1 })
        } else {
            None
        }
    }
}
