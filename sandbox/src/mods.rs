#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// todo: repr u16 for compactness?
pub enum Mod {
    Flinch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModLifetime {
    Game,
    Week,
    Season,
    Permanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModWithLifetime {
    lifetime: ModLifetime,
    the_mod: Mod, // mod is a keyword lmao
}

#[derive(Debug, Clone)]
pub struct Mods {
    // todo: store this as a set? or a smallvec?
    // we only have <10 entries so i think searching a vec might be faster anyway
    mods: Vec<ModWithLifetime>,
}

impl Mods {
    pub fn new() -> Mods {
        Mods { mods: Vec::new() }
    }

    pub fn has(&self, m: Mod) -> bool {
        self.mods.iter().any(|x| x.the_mod == m)
    }

    pub fn add(&mut self, m: Mod, lifetime: ModLifetime) {
        let ml = ModWithLifetime {
            the_mod: m,
            lifetime: lifetime,
        };
        if !self.mods.contains(&ml) {
            self.mods.push(ml);
        }
    }

    pub fn remove(&mut self, m: Mod) {
        self.mods.retain(|x| x.the_mod == m)
    }

    pub fn clear_game(&mut self) {
        self.mods.retain(|x| x.lifetime != ModLifetime::Game);
    }

    pub fn clear_weekly(&mut self) {
        self.mods.retain(|x| x.lifetime != ModLifetime::Week);
    }

    pub fn clear_season(&mut self) {
        self.mods.retain(|x| x.lifetime != ModLifetime::Season);
    }
}
