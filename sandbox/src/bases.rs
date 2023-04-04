use itertools::Itertools;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Baserunner {
    pub id: Uuid,
    pub base: u8,
}

/// Stores where all the baserunners are
#[derive(Debug, Clone)]
pub struct Baserunners {
    pub runners: Vec<Baserunner>,
}

impl Baserunners {
    pub fn new() -> Baserunners {
        Baserunners {
            runners: Vec::new(),
        }
    }

    pub fn occupied(&self, base: u8) -> bool {
        self.runners.iter().any(|x| x.base == base)
    }

    pub fn can_advance(&self, base: u8) -> bool {
        !self.occupied(base + 1)
    }

    pub fn at(&self, base: u8) -> Option<Uuid> {
        self.runners.iter().find(|x| x.base == base).map(|x| x.id)
    }

    pub fn remove(&mut self, base: u8) -> Option<Uuid> {
        let idx = self
            .runners
            .iter()
            .enumerate()
            .find(|(_, x)| x.base == base)
            .map(|x| x.0);

        if let Some(idx) = idx {
            let runner = self.runners.remove(idx);
            Some(runner.id)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.runners.clear();
    }

    pub fn advance(&mut self, base: u8) {
        for r in self.runners.iter_mut() {
            if r.base == base {
                r.base += 1;
                return;
            }
        }
    }

    pub fn advance_all(&mut self, amount: u8) {
        for r in self.runners.iter_mut() {
            r.base += amount;
        }
    }

    pub fn walk(&mut self) {
        let mut i = 0;
        for r in self.runners.iter_mut().rev() {
            if r.base == i {
                r.base += 1;
                i += 1;
            }
        }
    }

    pub fn advance_if(&mut self, mut f: impl FnMut(&Baserunner) -> bool) {
        for i in 0..self.runners.len() {
            if self.can_advance(self.runners[i].base) && f(&self.runners[i]) {
                self.runners[i].base += 1;
            }
        }
    }

    pub fn add(&mut self, id: Uuid, base: u8) {
        self.runners.push(Baserunner { id, base })
    }

    pub fn push(&mut self, runner: Baserunner) {
        self.runners.push(runner);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Baserunner> {
        self.runners.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Baserunner> {
        self.runners.iter_mut()
    }
}
