use reqwest::blocking::get;
use serde::Deserialize;
use serde_json;
use uuid::Uuid;
use std::fs;
use std::path::Path;

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
    let ignore_cache = false;
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

#[derive(Deserialize, Debug)]
pub struct ChronItem<D> {
    validFrom: String, //todo: these are timestamps
    validTo: Option<String>,
    pub data: D
}

#[derive(Deserialize, Debug)]
pub struct ChronDivision {
    pub id: Uuid,
    pub name: String,
    pub teams: Vec<Uuid>
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

#[derive(Deserialize, Debug, Clone)]
pub struct ChronPlayer {
    pub id: Uuid,
    pub name: String,
    pub leagueTeamId: Uuid,
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

