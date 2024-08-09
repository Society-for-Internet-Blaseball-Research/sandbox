use uuid::Uuid;

use crate::{bases::Baserunners, entities::{Player, World}, mods::{Mod, ModLifetime}, Game, Weather};

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
    Birds,
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
        tangled: Uuid,
        decreases: Vec<f64>
    },
    Shelled {
        batter: Uuid
    },
    HitByPitch {
        target: Uuid,
        hbp_type: u8
    },
    IncinerationWithChain {
        target: Uuid,
        replacement: Player,
        chain: Uuid
    },
    PeckedFree {
        player: Uuid
    },
    Zap {
        batter: bool
    }
}

impl Event {
    pub fn apply(&self, game: &mut Game, world: &mut World) {
        let repr = self.repr();
        game.events.add(repr.clone());
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
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                game.outs += 1;
                game.end_pa();
            }
            Event::Walk => {
                // maybe we should put batter in the event
                // todo: make a function that returns the current batter
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                game.runners.walk();
                game.runners.add(0, game.batting_team().batter.unwrap());
                game.base_sweep();
                game.end_pa();
            }
            Event::HomeRun => {
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                upgrade_spicy(game, world);
                let no_runners_on = game.runners.empty();
                game.runners.advance_all(4);
                game.batting_team_mut().score += game.get_run_value();
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
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                upgrade_spicy(game, world);
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
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                downgrade_spicy(game, world);
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::Flyout {
                fielder: _fielder,
                ref runners_after,
            } => {
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                downgrade_spicy(game, world);
                game.outs += 1;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::DoublePlay { ref runners_after } => {
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                downgrade_spicy(game, world);
                game.outs += 2;
                game.runners = runners_after.clone();
                game.base_sweep();
                game.end_pa();
            }
            Event::FieldersChoice { ref runners_after } => {
                world.player_mut(game.batting_team().batter.unwrap()).feed.add(repr.clone());
                downgrade_spicy(game, world);
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
            Event::Birds => {},
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
                if !game.events.has(String::from("salmon"), if game.top { 3 } else { 2 }) {
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
            Event::Soundproof { resists: _resists, tangled, ref decreases } => {
                world.player_mut(tangled).boost(decreases);
            },
            Event::Shelled { batter: _batter } => {
                let bt = game.batting_team_mut();
                bt.batter_index += 1;
            },
            Event::HitByPitch { target, hbp_type } => {
                let effect = match hbp_type {
                    0 => Some(Mod::Unstable),
                    1 => Some(Mod::Flickering),
                    _ => None
                };
                world.player_mut(target).mods.add(effect.unwrap(), ModLifetime::Week);
                game.runners.walk();
                game.runners.add(0, game.batting_team().batter.unwrap());
                game.base_sweep();
                game.end_pa();
            },
            Event::IncinerationWithChain { target, ref replacement, chain } => {
                let replacement_id = world.add_rolled_player(replacement.clone(), world.player(target).team.unwrap());
                if let Some(batter) = game.batting_team().batter {
                    if batter == target {
                        game.batting_team_mut().batter = Some(replacement_id);
                    }
                } else if target == game.pitching_team().pitcher {
                    game.pitching_team_mut().pitcher = replacement_id;
                }
                world.replace_player(target, replacement_id);
                world.player_mut(chain).mods.add(Mod::Unstable, ModLifetime::Week);
            },
            Event::PeckedFree { player } => {
                world.player_mut(player).mods.remove(Mod::Shelled);
                world.player_mut(player).mods.add(Mod::Superallergic, ModLifetime::Permanent);
            },
            Event::Zap { batter } => {
                if batter {
                    game.strikes -= 1;
                } else {
                    game.balls -= 1;
                }
            }
        }
    }

    //todo: might merge this with a possible future print function
    //btw these don't need to be growable but static lifetimes
    //are annoying
    fn repr(&self) -> String {
        let ev = match *self {
            Event::BatterUp { .. } => "batterUp",
            Event::InningSwitch { .. } => "inningSwitch",
            Event::GameOver => "gameOver",
            Event::Ball => "ball",
            Event::Strike => "strike",
            Event::Foul => "foul",
            Event::Strikeout => "strikeOut",
            Event::Walk => "walk",
            Event::HomeRun => "homeRun",
            Event::BaseHit { .. } => "baseHit",
            Event::GroundOut { .. } => "groundOut",
            Event::Flyout { .. } => "flyout",
            Event::DoublePlay { .. } => "doublePlay",
            Event::FieldersChoice { .. } => "fieldersChoice",
            Event::BaseSteal { .. } => "baseSteal",
            Event::CaughtStealing { .. } => "caughtStealing",
            Event::Incineration { .. } => "incineration",
            Event::Peanut { .. } => "peanut",
            Event::Birds => "birds",
            Event::Feedback { .. } => "feedback",
            Event::Reverb { .. } => "reverb",
            Event::Blooddrain { .. } => "blooddrain",
            //todo: add win manipulation when we actually have wins
            Event::Sun2 { .. } => "sun2",
            Event::BlackHole { .. } => "blackHole",
            Event::Salmon { .. } => "salmon",
            Event::PolaritySwitch => "polaritySwitch",
            Event::NightShift { .. } => "nightShift",
            Event::Fireproof { .. } => "fireproof",
            Event::Soundproof { .. } => "soundproof",
            Event::Shelled { .. } => "shelled",
            Event::HitByPitch { .. } => "hitByPitch",
            Event::IncinerationWithChain { .. } => "incinerationWithChain",
            Event::PeckedFree { .. } => "peckedFree",
            Event::Zap { .. } => "zap"
        };
        String::from(ev)
    }
}


fn upgrade_spicy(game: &mut Game, world: &mut World) {
    let batter = world.player_mut(game.batting_team().batter.unwrap());
    if batter.mods.has(Mod::Spicy) && batter.feed.streak_multiple(vec![String::from("baseHit"), String::from("homeRun")], -1) == 1 {
        batter.mods.add(Mod::HeatingUp, ModLifetime::Permanent);
    } else if batter.mods.has(Mod::HeatingUp) {
        batter.mods.remove(Mod::HeatingUp);
        batter.mods.add(Mod::RedHot, ModLifetime::Permanent);
    }
}

fn downgrade_spicy(game: &mut Game, world: &mut World) {
     let batter = world.player_mut(game.batting_team().batter.unwrap());
     if batter.mods.has(Mod::RedHot) {
         batter.mods.remove(Mod::RedHot);
     } else if batter.mods.has(Mod::HeatingUp) {
         batter.mods.remove(Mod::HeatingUp);
     }
}

#[derive(Clone, Debug)]
pub struct Events {
    events: Vec<String>
}

impl Events {
    pub fn new() -> Events {
        Events {
            events: Vec::new()
        }
    }
    pub fn add(&mut self, repr: String) {
        self.events.push(repr);
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn last(&self) -> &String {
        if self.events.len() == 0 {
            panic!("don't call this when the game begins");
        }
        self.events.last().unwrap()
    }
    pub fn has(&self, s: String, limit: i16) -> bool {
        let mut half_innings = 0i16;
        for ev in self.events.iter().rev() {
            if *ev == s {
                return true;
            } else if limit != -1 && *ev == "inningSwitch" {
                if half_innings < limit {
                    half_innings += 1;
                } else {
                    return false;
                }
            }
        }
        false
    }
    pub fn count(&self, s: String, limit: i16) -> u8 {
        let mut half_innings = 0i16;
        let mut counter = 0u8;
        for ev in self.events.iter().rev() {
            if *ev == s {
                counter += 1;
            } else if *ev == "inningSwitch" && limit != -1 {
                if half_innings < limit {
                    half_innings += 1;
                } else {
                    return counter;
                }
            }
        }
        counter
    }
    pub fn streak_multiple(&self, strvec: Vec<String>, limit: i16) -> u8 {
        let mut half_innings = 0i16;
        let mut counter = 0u8;
        for ev in self.events.iter().rev() {
            if *ev == "inningSwitch" && limit != -1 {
                if half_innings < limit {
                    half_innings += 1;
                } else {
                    return counter;
                }
            } else {
		//contains doesn't work
		for s in &strvec {
		    if *ev == *s {
			counter += 1;
		    }
		}
	    }
        }
        counter
    }
}
