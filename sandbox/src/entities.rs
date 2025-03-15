use std::{collections::BTreeMap, f64::consts::PI};

use uuid::Uuid;

use crate::{events::Events, mods::{Mod, ModLifetime, Mods}, rng::Rng};

pub struct World {
    pub players: BTreeMap<Uuid, Player>,
    pub teams: BTreeMap<Uuid, Team>,
    pub stadiums: BTreeMap<Uuid, Stadium>,
    pub hall: Vec<Uuid>, //think of this as a view into a section of players
    pub season_ruleset: u8,
}

impl World {
    pub fn new(season_ruleset: u8) -> World {
        World {
            players: BTreeMap::new(),
            teams: BTreeMap::new(),
            stadiums: BTreeMap::new(),
            hall: Vec::new(),
            season_ruleset
        }
    }
    pub fn player(&self, id: Uuid) -> &Player {
        self.players.get(&id).unwrap()
    }

    pub fn team(&self, id: Uuid) -> &Team {
        self.teams.get(&id).unwrap()
    }

    pub fn team_name(&self, name: String) -> &Team {
        for (_, team) in self.teams.iter() {
            if *team.name == name {
                return team;
            }
        }
        panic!("team name not found");
    }

    pub fn player_mut(&mut self, id: Uuid) -> &mut Player {
        self.players.get_mut(&id).unwrap()
    }

    pub fn team_mut(&mut self, id: Uuid) -> &mut Team {
        self.teams.get_mut(&id).unwrap()
    }

    pub fn team_name_mut(&mut self, name: String) -> &mut Team {
        for (_, team) in self.teams.iter_mut() {
            if *team.name == name {
                return team;
            }
        }
        panic!("team name not found");
    }

    pub fn insert_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn insert_team(&mut self, team: Team) {
        self.teams.insert(team.id, team);
    }

    pub fn replace_player(&mut self, player_id: Uuid, new_player_id: Uuid) {
        let player = self.player_mut(player_id);
        let team_id = player.team.unwrap();
        player.team = None;
        self.hall.push(player_id);
        let team = self.team_mut(team_id);
        team.replace_player(player_id, new_player_id);
    }

    pub fn swap(&mut self, player1_id: Uuid, player2_id: Uuid) {
        let team_id_1 = self.player(player1_id).team.unwrap();
        let team_id_2 = self.player(player2_id).team.unwrap();
        let player1 = self.player_mut(player1_id);
        player1.team = Some(team_id_2);
        let player2 = self.player_mut(player2_id);
        player2.team = Some(team_id_1);
        let team1 = self.team_mut(team_id_1);
        team1.replace_player(player1_id, player2_id);
        let team2 = self.team_mut(team_id_2);
        team2.replace_player(player2_id, player1_id);
    }

    pub fn swap_hall(&mut self, player1_id: Uuid, player2_id: Uuid) {
        let team_id_1 = self.player(player1_id).team.unwrap();
        let player1 = self.player_mut(player1_id);
        player1.team = None;
        let player2 = self.player_mut(player2_id);
        player2.team = Some(team_id_1);
        let team1 = self.team_mut(team_id_1);
        team1.replace_player(player1_id, player2_id);
    }

    pub fn gen_team(&mut self, rng: &mut Rng, name: String, emoji: String) -> Uuid {
        let id = Uuid::new_v4();
        let mut team = Team {
            id,
            emoji,
            lineup: Vec::new(),
            rotation: Vec::new(),
            shadows: Vec::new(),
            name,
            wins: 0,
            losses: 0,
            postseason_wins: 0,
            postseason_losses: 0,
            partying: false,
            fate: 100,
            mods: Mods::new(),
        };

        for _ in 0..9 {
            team.lineup.push(self.gen_player(rng, id));
        }

        for _ in 0..5 {
            team.rotation.push(self.gen_player(rng, id));
        }

        for _ in 0..11 {
            team.shadows.push(self.gen_player(rng, id));
        }

        self.insert_team(team);
        id
    }

    pub fn gen_player(&mut self, rng: &mut Rng, team: Uuid) -> Uuid {
        let interview_rolls = 6 + 2; //soul, allergy, fate, ritual, blood, coffee + names
        let mut player = Player::new(rng);
        let id = player.id;
        player.name = format!("Player {}", &(player.id).to_string()[..8]);
        for _ in 0..interview_rolls {
            rng.next(); //to make the rng align
        }
        player.team = Some(team.clone());
        self.insert_player(player);
        id
    }

    pub fn add_rolled_player(&mut self, mut player: Player, team: Uuid) -> Uuid {
        let id = player.id;
        player.name = format!("Player {}", &(player.id).to_string()[..8]);
        player.team = Some(team.clone());
        self.insert_player(player);
        id
    }

    pub fn random_hall_player(&self, rng: &mut Rng) -> Uuid {
        let index = rng.index(self.hall.len());
        self.hall[index]
    }

    pub fn clear_game(&mut self) {
        for (_, player) in self.players.iter_mut() {
            player.mods.clear_game();
        }
    }
    
    pub fn clear_weekly(&mut self) {
        for (_, player) in self.players.iter_mut() {
            player.mods.clear_weekly();
        }
    }

    pub fn clear_season(&mut self) {
        for (_, player) in self.players.iter_mut() {
            player.mods.clear_season();
        }
    }
}

pub struct NameGen<'a> {
    first_names: Vec<&'a str>,
    last_names: Vec<&'a str>,
    first_name_length: u16,
    last_name_length: u16,
}

impl<'a> NameGen<'a> {
    pub fn new() -> NameGen<'a> {
        //todo: season rulesets
        NameGen {
            first_names: include_str!("firstnames.txt").split_whitespace().collect(),
            last_names: include_str!("lastnames.txt").split_whitespace().collect(),
            first_name_length: 532,
            last_name_length: 538,
        }
    }
    pub fn generate(&self, rng: &mut Rng) -> String {
        let first_name_index = (rng.next() * self.first_name_length as f64).floor() as usize;
        let last_name_index = (rng.next() * self.last_name_length as f64).floor() as usize;
        let mut name = self.first_names[first_name_index].to_string();
        name.push_str(" ");
        name.push_str(self.last_names[last_name_index]);
        name
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

impl PlayerAttr {
    pub fn discr(&self) -> u8 {
        *self as u8
    }
    pub fn is_batting(&self) -> bool {
        let discr = *self as u8;
        discr < 8
    }
    pub fn is_pitching(&self) -> bool {
        let discr = *self as u8;
        discr > 7 && discr < 14
    }
    pub fn is_running(&self) -> bool {
        let discr = *self as u8;
        discr > 13 && discr < 19
    }
    pub fn is_defense(&self) -> bool {
        let discr = *self as u8;
        discr > 18 && discr < 24
    }
    pub fn is_vibes(&self) -> bool {
        let discr = *self as u8;
        discr > 23
    }
    pub fn is_negative(&self) -> bool {
        if let PlayerAttr::Patheticism = *self {
            true
        } else if let PlayerAttr::Tragicness = *self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub mods: Mods,
    pub legendary_item: Option<LegendaryItem>,
    pub team: Option<Uuid>, //ig
    
    pub feed: Events,
    pub swept_on: Option<usize>,
    pub scattered_letters: u8,

    // stats??
    // todo: maybe represent stats with an array
    // to make rolling new players less awkward
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
    pub fn new(rng: &mut Rng) -> Player {
        let id = Uuid::new_v4();

        Player {
            id,
            name: "".to_string(), //todo: name gen
            mods: Mods::new(),
            legendary_item: None,
            team: None,

            feed: Events::new(),
            swept_on: None,
            scattered_letters: 0,

            // NOW it's rng order compatible
            thwackability: rng.next(),
            moxie: rng.next(),
            divinity: rng.next(),
            musclitude: rng.next(),
            patheticism: rng.next(),
            buoyancy: rng.next(),
            base_thirst: rng.next(),
            laserlikeness: rng.next(),
            ground_friction: rng.next(),
            continuation: rng.next(),
            indulgence: rng.next(),
            martyrdom: rng.next(),
            tragicness: rng.next(),
            shakespearianism: rng.next(),
            suppression: rng.next(),
            unthwackability: rng.next(),
            coldness: rng.next(),
            overpowerment: rng.next(),
            ruthlessness: rng.next(),
            omniscience: rng.next(),
            tenaciousness: rng.next(),
            watchfulness: rng.next(),
            anticapitalism: rng.next(),
            chasiness: rng.next(),
            pressurization: rng.next(),
            cinnamon: rng.next(),
        }
    }
    pub fn vibes(&self, day: usize) -> f64 {
        if self.scattered_letters > 0 {
            0.0
        } else {
            let frequency = 6.0 + (10.0 * self.buoyancy).round();
            // todo: sin table? do we care that much?
            let sin_phase = (PI * ((2.0 / frequency) * (day as f64) + 0.5)).sin();
            0.5 * ((sin_phase - 1.0) * self.pressurization + (sin_phase + 1.0) * self.cinnamon)
        }
    }
    pub fn boost(&mut self, boosts: &Vec<f64>) {
        //todo: implement custom boost order
        self.buoyancy += boosts[0];
        self.divinity += boosts[1];
        self.martyrdom += boosts[2];
        self.moxie += boosts[3];
        self.musclitude += boosts[4];
        self.patheticism -= boosts[5];
        self.thwackability += boosts[6];
        self.tragicness -= boosts[7];
                
        self.coldness += boosts[8];
        self.overpowerment += boosts[9];
        self.ruthlessness += boosts[10];
        self.shakespearianism += boosts[11];
        self.suppression += boosts[12];
        self.unthwackability += boosts[13];
                
        self.base_thirst += boosts[14];
        self.continuation += boosts[15];
        self.ground_friction += boosts[16];
        self.indulgence += boosts[17];
        self.laserlikeness += boosts[18];
                
        self.anticapitalism += boosts[19];
        self.chasiness += boosts[20];
        self.omniscience += boosts[21];
        self.tenaciousness += boosts[22];
        self.watchfulness += boosts[23];
        
        if boosts.len() == 25 {
            self.cinnamon += boosts[24];
        } else {
            self.pressurization += boosts[24];
            self.cinnamon += boosts[25];
        }
    }
    pub fn add_legendary_item(&mut self, item: LegendaryItem) {
        if let LegendaryItem::NightVisionGoggles = item {
            self.mods.add(Mod::NightVision, ModLifetime::LegendaryItem);
        } else if let LegendaryItem::TheIffeyJr = item {
            self.mods.add(Mod::Minimized, ModLifetime::LegendaryItem);
        } else if let LegendaryItem::ActualAirplane = item {
            self.mods.add(Mod::Blaserunning, ModLifetime::LegendaryItem);
        }
        self.legendary_item = Some(item);
    }
    pub fn remove_legendary_item(&mut self) {
        self.mods.clear_legendary_item();
        self.legendary_item = None;
    }
    pub fn get_run_value(&self) -> f64 {
        if self.mods.has(Mod::Wired) {
            0.5
        } else if self.mods.has(Mod::Tired) {
            -0.5
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug)]
pub enum LegendaryItem {
    DialTone,
    LiteralArmCannon,
    VibeCheck,
    BangersAndSmash,
    GrapplingHook,
    Mushroom,
    NightVisionGoggles,
    ShrinkRay,
    TheIffeyJr,
    ActualAirplane
}

#[derive(Clone, Debug)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub emoji: String,

    pub lineup: Vec<Uuid>,
    pub rotation: Vec<Uuid>,
    pub shadows: Vec<Uuid>,

    pub wins: i16,
    pub losses: i16,
    pub postseason_wins: i16,
    pub postseason_losses: i16,
    pub partying: bool,
    pub fate: usize,

    pub mods: Mods,
}

impl Team {
    fn replace_player(&mut self, id: Uuid, new_id: Uuid) {
        //todo: write this code with return
        if let Some(idx) = self.lineup.iter().position(|x| *x == id) {
            self.lineup[idx] = new_id;
        } else if let Some(idx) = self.rotation.iter().position(|x| *x == id) {
            self.rotation[idx] = new_id;
        } else if let Some(idx) = self.shadows.iter().position(|x| *x == id) {
            self.shadows[idx] = new_id;
        } else {
            panic!("player not found");
        }
    }

    //if reverb type is 1 (partial), returns pairs of players to be swapped
    //if not, returns indexes of old slots (lineup lower) in rotation-lineup order
    pub fn roll_reverb_changes(&self, rng: &mut Rng, reverb_type: u8, gravity_players: &Vec<usize>) -> Vec<usize> {
        let mut reverb_changes = Vec::new();
        let lineup_length = self.lineup.len();
        let rotation_length = self.rotation.len();
        let length = lineup_length + rotation_length;
        match reverb_type {
            0 => {
                let mut players_rem: Vec<usize> = Vec::new(); //tracks players still unsorted
                for i in 0..length {
                    if !gravity_players.contains(&(i as usize)) {
                        players_rem.push(i as usize);
                    }
                }

                for i in 0..length {
                    let old_i: usize = if i < rotation_length { i + lineup_length } else { i - rotation_length };
                    if gravity_players.contains(&old_i) {
                        if i < lineup_length {
                            reverb_changes.push(i + rotation_length);
                        } else {
                            reverb_changes.push(i - lineup_length);
                        }
                    } else {
                        let rem_idx = (rng.next() * (players_rem.len() as f64)).floor() as usize;
                        let idx = players_rem[rem_idx];
                        players_rem.retain(|j| *j != idx);
                        reverb_changes.push(idx);
                    }
                }
            },
            //everything regarding gravity past this line is an assumption
            1 => {
                for _ in 0..3 {
                    let roll1 = (rng.next() * (length as f64)).floor() as usize;
                    let roll2 = (rng.next() * (length as f64)).floor() as usize;
                    let idx1 = if roll1 < rotation_length { lineup_length + roll1 } else { roll1 - rotation_length };
                    let idx2 = if roll2 < rotation_length { lineup_length + roll2 } else { roll2 - rotation_length };
                    if !gravity_players.contains(&idx1) && !gravity_players.contains(&idx2) {
                        reverb_changes.push(idx1);
                        reverb_changes.push(idx2);
                    }
                }
            },
            2 => {
                let mut players_rem: Vec<usize> = Vec::new();
                for i in 0..lineup_length {
                    if !gravity_players.contains(&(i as usize)) {
                        players_rem.push(i as usize);
                    }
                }

                for i in 0..lineup_length {
                    if gravity_players.contains(&(i as usize)) {
                        reverb_changes.push(i);
                    } else {
                        let rem_idx = (rng.next() * (players_rem.len() as f64)).floor() as usize;
                        let idx = players_rem[rem_idx];
                        players_rem.retain(|j| *j != idx);
                        reverb_changes.push(idx);
                    }
                }
            },
            3 => {
                let mut players_rem: Vec<usize> = Vec::new();
                for i in 0..rotation_length {
                    if !gravity_players.contains(&(i + lineup_length)) {
                        players_rem.push(i as usize);
                    }
                }

                for i in 0..rotation_length {
                    if gravity_players.contains(&((i + lineup_length) as usize)) {
                        reverb_changes.push(i);
                    } else {
                        let rem_idx = (rng.next() * (players_rem.len() as f64)).floor() as usize;
                        let idx = players_rem[rem_idx];
                        players_rem.retain(|j| *j != idx);
                        reverb_changes.push(idx);
                    }
                }
            },
            _ => {
                panic!("wrong reverb type");
            }
        }
        reverb_changes
    }

    pub fn apply_reverb_changes(&mut self, reverb_type: u8, changes: &Vec<usize>) {
        let mut result: Vec<Uuid> = Vec::new();
        let lineup_length = self.lineup.len();
        let rotation_length = self.rotation.len();
        let length = lineup_length + rotation_length;
        match reverb_type {
            0 => {
                for i in rotation_length..length {
                    let player_slot = changes[i];
                    if player_slot < lineup_length {
                        result.push(self.lineup[changes[i]].clone());
                    } else {
                        result.push(self.rotation[changes[i] - lineup_length].clone());
                    }
                }
                for i in 0..rotation_length {
                    let player_slot = changes[i];
                    if player_slot < lineup_length {
                        result.push(self.lineup[changes[i]].clone());
                    } else {
                        result.push(self.rotation[changes[i] - lineup_length].clone());
                    }
                }
            },
            1 => {
                for i in 0..rotation_length {
                    result.push(self.rotation[i].clone());
                }
                for i in 0..lineup_length {
                    result.push(self.lineup[i].clone());
                }
                let mut change_idx = 0;
                while change_idx < changes.len() {
                    let slot1 = changes[change_idx];
                    let slot2 = changes[change_idx + 1];
                    result.swap(slot1, slot2);
                    change_idx += 2;
                }
            },
            2 => {
                for i in 0..lineup_length {
                    result.push(self.lineup[changes[i]].clone());
                }
                for i in 0..rotation_length {
                    result.push(self.rotation[i].clone());
                }
            },
            3 => {
                for i in 0..lineup_length {
                    result.push(self.lineup[i].clone());
                }
                for i in 0..rotation_length {
                    result.push(self.rotation[changes[i]].clone());
                }
            },
            _ => {
                panic!("wrong reverb type, somehow");
            }
        }
        for i in 0..length {
            if i < lineup_length {
                self.lineup[i] = result[i];
            } else {
                self.rotation[i - lineup_length] = result[i];
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stadium {
    pub id: Uuid,

    pub name: String,
    // todo: stats ig
}
