use std::collections::BTreeMap;

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
