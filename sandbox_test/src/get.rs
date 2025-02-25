use reqwest::blocking::get;
use serde::Deserialize;
use serde_json;
use uuid::Uuid;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use sandbox::{entities::{Player, Team, World}, mods::{Mods, Mod, ModLifetime}, events::Events};

pub fn fill(season: u8) {

}

//todo: unwrap -> ?, println! -> panic!
pub fn divisions(season: u8) -> Option<ChronArray<ChronDivision>> { 
    let timestamp = match season {
        11 => "2021-03-01T15:00:00Z",
        _ => todo!(),
    };
    let mut result: ChronArray<ChronDivision>;
    let string = format!("json/s{}divisions.json", season);
    let path = Path::new(&string);
    let cached = path.exists();
    let ignore_cache = false;
    if cached && !ignore_cache {
        let file = fs::read(path);
        if file.is_err() {
            println!("cache error: {}", file.unwrap_err());
            return None;
        }
        result = serde_json::from_slice(file.unwrap().as_slice()).unwrap();
    } else {
        let url = format!("https://api.sibr.dev/chronicler/v2/entities?type=division&at={}", timestamp);
        let res = get(url);
        if res.is_err() {
            println!("request error: {}", res.unwrap_err());
            return None;
        }
        let bytes = res.unwrap().bytes().unwrap();
        if !Path::new("json").exists() {
            fs::create_dir("json");
        }
        let data = serde_json::from_slice(&bytes);
        if data.is_ok() {
            fs::write(path, bytes);
        } //this looks stupid but it has to to compile
        if data.is_err() {
            println!("json error: {}", data.unwrap_err());
            return None;
        }
        result = data.unwrap();
    }
    //println!("{:?}", data.unwrap());
    if season == 11 {
        result.items.retain(|item| item.validTo.is_some());
    }

    return Some(result);
}

pub fn hall(season: u8) -> Option<ChronHall> {
    let timestamp = match season {
        11 => "2021-03-01T15:00:00Z",
        _ => todo!(),
    };
    let mut result: ChronArray<ChronHall>;
    let string = format!("json/s{}hall.json", season);
    let path = Path::new(&string);
    let cached = path.exists();
    let ignore_cache = true;
    if cached && !ignore_cache {
        let file = fs::read(path);
        if file.is_err() {
            println!("cache error: {}", file.unwrap_err());
            return None;
        }
        result = serde_json::from_slice(file.unwrap().as_slice()).unwrap();
    } else {
        let url = format!("https://api.sibr.dev/chronicler/v2/entities?type=tributes&at={}", timestamp);
        let res = get(url);
        if res.is_err() {
            println!("request error: {}", res.unwrap_err());
            return None;
        }
        let bytes = res.unwrap().bytes().unwrap();
        if !Path::new("json").exists() {
            fs::create_dir("json");
        }
        let data = serde_json::from_slice(&bytes);
        if data.is_ok() {
            fs::write(path, bytes);
        } //this looks stupid but it has to to compile
        if data.is_err() {
            println!("json error: {}", data.unwrap_err());
            return None;
        }
        result = data.unwrap();
    }
    //println!("{:?}", data.unwrap());

    return Some(result.items[0].data.clone());
}

pub fn team(id: Uuid, season: u8) -> Option<ChronTeam> {
    let timestamp = match season {
        11 => "2021-03-01T15:00:00Z",
        _ => todo!(),
    };
    let mut result: ChronArray<ChronTeam>;
    let string = format!("json/s{}teams_{}.json", season, id);
    let path = Path::new(&string);
    let cached = path.exists();
    let ignore_cache = false;
    if cached && !ignore_cache {
        let file = fs::read(path);
        if file.is_err() {
            println!("cache error: {}", file.unwrap_err());
            return None;
        }
        result = serde_json::from_slice(file.unwrap().as_slice()).unwrap();
    } else {
        let url = format!("https://api.sibr.dev/chronicler/v2/entities?type=team&id={}&at={}", id, timestamp);
        let res = get(url);
        if res.is_err() {
            println!("request error: {}", res.unwrap_err());
            return None;
        }
        let bytes = res.unwrap().bytes().unwrap();
        if !Path::new("json").exists() {
            fs::create_dir("json");
        }
        let data = serde_json::from_slice(&bytes);
        fs::write(path, bytes);
        if data.is_err() {
            println!("json error: {}", data.unwrap_err());
            return None;
        }
        result = data.unwrap();
    }
    //println!("{:?}", data.unwrap());
    return Some(result.items[0].data.clone());
}

pub fn player(id: Uuid, season: u8) -> Option<ChronPlayer> {
    let timestamp = match season {
        11 => "2021-03-01T15:00:00Z",
        _ => todo!(),
    };
    let mut result: ChronArray<ChronPlayer>;
    let string = format!("json/s{}players_{}.json", season, id);
    let path = Path::new(&string);
    let cached = path.exists();
    let ignore_cache = true;
    if cached && !ignore_cache {
        let file = fs::read(path);
        if file.is_err() {
            println!("cache error: {}", file.unwrap_err());
            return None;
        }
        result = serde_json::from_slice(file.unwrap().as_slice()).unwrap();
    } else {
        let url = format!("https://api.sibr.dev/chronicler/v2/entities?type=player&id={}&at={}", id, timestamp);
        let res = get(url);
        if res.is_err() {
            println!("request error: {}", res.unwrap_err());
            return None;
        }
        let bytes = res.unwrap().bytes().unwrap();
        if !Path::new("json").exists() {
            fs::create_dir("json");
        }
        let data = serde_json::from_slice(&bytes);
        fs::write(path, bytes);
        if data.is_err() {
            println!("json error: {}", data.unwrap_err());
            return None;
        }
        result = data.unwrap();
    }
    //println!("{:?}", data.unwrap());
    return Some(result.items[0].data.clone());
}

#[derive(Deserialize, Debug)]
pub struct ChronArray<D> {
    pub items: Vec<ChronItem<D>>
}

impl ChronArray<ChronDivision> {
    pub fn convert(self) -> Vec<Uuid> {
        let mut divs: Vec<ChronDivision> = self.items.iter().map(|item| item.data.clone()).collect();
        divs.sort_by(|d1, d2| d1.name.cmp(&d2.name));
        divs.iter()
            .flat_map(|d| d.teams.clone())
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub struct ChronItem<D> {
    validFrom: String, //todo: these are timestamps
    validTo: Option<String>,
    pub data: D
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChronDivision {
    pub id: Uuid,
    pub name: String,
    pub teams: Vec<Uuid>
}

pub type ChronHall = Vec<ChronTribute>;

#[derive(Deserialize, Debug, Clone)]
pub struct ChronTribute {
    pub peanuts: u64,
    pub playerId: Uuid,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChronTeam {
    pub id: Uuid,
    pub fullName: String,
    pub emoji: String,

    pub lineup: Vec<Uuid>,
    pub rotation: Vec<Uuid>,
    pub bench: Vec<Uuid>,
    pub bullpen: Vec<Uuid>,

    pub stadium: Option<Uuid>,

    pub permAttr: Vec<String>,
    pub seasAttr: Vec<String>,
    pub weekAttr: Vec<String>,
    pub gameAttr: Vec<String>,
}

impl ChronTeam {
    pub fn convert(self) -> Team {
        Team {
            id: self.id,
            name: self.fullName,
            emoji: self.emoji,

            lineup: self.lineup,
            rotation: self.rotation,
            shadows: [self.bench, self.bullpen].concat(),

            wins: 0,
            losses: 0,
            postseason_wins: 0,
            postseason_losses: 0,
            fate: 0, //todo: fate is assigned later each season. retrieve it from chron instead

            mods: modconvert(&[self.permAttr, self.seasAttr, self.weekAttr, self.gameAttr])
        }
    }
}

fn modconvert(mods: &[Vec<String>]) -> Mods {
    let mut smods = Mods::new();
    for i in 0..4 {
        for m in mods[i].iter() {
            let lifetime = match i {
                0 => ModLifetime::Permanent,
                1 => ModLifetime::Season,
                2 => ModLifetime::Week,
                3 => ModLifetime::Game,
                _ => {
                    panic!("Invalid parsed lifetime, somehow")
                }
            };
            let smod = Mod::from_str(m.as_str());
            if smod.is_err() {
                println!("Couldn't parse mod {}", m);
                continue;
            }
            smods.add(smod.unwrap(), lifetime);
        }
    }
    smods
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChronPlayer {
    pub id: Uuid,
    pub name: String,
    pub leagueTeamId: Option<Uuid>,
    pub deceased: bool,

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

    pub baseThirst: f64,
    pub continuation: f64,
    pub groundFriction: f64,
    pub indulgence: f64,
    pub laserlikeness: f64,

    pub anticapitalism: f64,
    pub chasiness: f64,
    pub omniscience: f64,
    pub tenaciousness: f64,
    pub watchfulness: f64,

    pub pressurization: f64,
    pub cinnamon: f64,

    pub permAttr: Vec<String>,
    pub seasAttr: Vec<String>,
    pub weekAttr: Vec<String>,
    pub gameAttr: Vec<String>
}

impl ChronPlayer {
    pub fn convert(self) -> Player {
        Player {
            id: self.id,
            name: self.name,
            mods: modconvert(&[self.permAttr, self.seasAttr, self.weekAttr, self.gameAttr]),
            legendary_item: None,
            team: self.leagueTeamId,

            feed: Events::new(),

            buoyancy: self.buoyancy,
            divinity: self.divinity,
            martyrdom: self.martyrdom,
            moxie: self.moxie,
            musclitude: self.musclitude,
            patheticism: self.patheticism,
            thwackability: self.thwackability,
            tragicness: self.tragicness,
            
            coldness: self.coldness,
            overpowerment: self.overpowerment,
            ruthlessness: self.ruthlessness,
            shakespearianism: self.shakespearianism,
            suppression: self.suppression,
            unthwackability: self.unthwackability,

            base_thirst: self.baseThirst,
            continuation: self.continuation,
            ground_friction: self.groundFriction,
            indulgence: self.indulgence,
            laserlikeness: self.laserlikeness,

            anticapitalism: self.anticapitalism,
            chasiness: self.chasiness,
            omniscience: self.omniscience,
            tenaciousness: self.tenaciousness,
            watchfulness: self.watchfulness,

            pressurization: self.pressurization,
            cinnamon: self.cinnamon,
        }
    }
}
