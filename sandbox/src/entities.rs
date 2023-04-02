use std::{collections::BTreeMap, f64::consts::PI};

use uuid::Uuid;

use crate::mods::Mods;

pub struct World {
    pub players: BTreeMap<Uuid, Player>,
    pub teams: BTreeMap<Uuid, Team>,
    pub stadiums: BTreeMap<Uuid, Stadium>,
}

impl World {
    pub fn new() -> World {
        World {
            players: BTreeMap::new(),
            teams: BTreeMap::new(),
            stadiums: BTreeMap::new(),
        }
    }
    pub fn player(&self, id: Uuid) -> &Player {
        self.players.get(&id).unwrap()
    }

    pub fn team(&self, id: Uuid) -> &Team {
        self.teams.get(&id).unwrap()
    }

    pub fn insert_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn insert_team(&mut self, team: Team) {
        self.teams.insert(team.id, team);
    }
}

// use this for like multiplier calc or something
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAttr {
    Buoyancy,
    Divinity,
    Martyrdom,
    Moxie,
    Musclitude,
    Patheticism,
    Thwackability,
    Tragicness,
    Coldness,
    Overpowerment,
    Ruthlessness,
    Shakespearianism,
    Suppression,
    Unthwackability,
    BaseThirst,
    Continuation,
    GroundFriction,
    Indulgence,
    Laserlikeness,
    Anticapitalism,
    Chasiness,
    Omniscience,
    Tenaciousness,
    Watchfulness,
    Pressurization,
    Cinnamon,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub mods: Mods,

    // stats??
    pub buoyancy: f64,
    pub divinity: f64,
    pub martyrdom: f64,
    pub moxie: f64,
    pub musclitude: f64,
    pub patheticism: f64,
    pub thwackability: f64,
    pub tragicness: f64,

    pub coldness: f64,
    pub overpowerment: f64,
    pub ruthlessness: f64,
    pub shakespearianism: f64,
    pub suppression: f64,
    pub unthwackability: f64,

    pub base_thirst: f64,
    pub continuation: f64,
    pub ground_friction: f64,
    pub indulgence: f64,
    pub laserlikeness: f64,

    pub anticapitalism: f64,
    pub chasiness: f64,
    pub omniscience: f64,
    pub tenaciousness: f64,
    pub watchfulness: f64,

    pub pressurization: f64,
    pub cinnamon: f64,
}

impl Player {
    pub fn vibes(&self, day: usize) -> f64 {
        let frequency = 6.0 + (10.0 * self.buoyancy).round();
        // todo: sin table? do we care that much?
        let sin_phase = PI * ((2.0 / frequency) * (day as f64) + 0.5);
        0.5 * ((sin_phase - 1.0) * self.pressurization + (sin_phase + 1.0) * self.cinnamon)
    }
}

#[derive(Clone, Debug)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub emoji: String,

    pub lineup: Vec<Uuid>,
    pub rotation: Vec<Uuid>,
    pub shadows: Vec<Uuid>,

    pub mods: Mods,
}

#[derive(Clone, Debug)]
pub struct Stadium {
    pub id: Uuid,

    pub name: String,
    // todo: stats ig
}
