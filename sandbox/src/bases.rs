use std::iter::zip;

use itertools::Itertools;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Baserunner {
    pub id: Uuid,
    pub base: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct BaserunnerRef<'a> {
    pub id: &'a Uuid,
    pub base: &'a u8,
}

#[derive(Debug)]
pub struct BaserunnerRefMut<'a> {
    pub id: &'a mut Uuid,
    pub base: &'a mut u8,
}

/// Stores where all the baserunners are
#[derive(Debug, Clone)]
pub struct Baserunners {
    pub ids: Vec<Uuid>,
    pub bases: Vec<u8>,
}

impl Baserunners {
    pub fn new() -> Baserunners {
        Baserunners {
            ids: Vec::new(),
            bases: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.bases.clear();
        self.ids.clear();
    }

    pub fn occupied(&self, base: u8) -> bool {
        self.bases.iter().any(|&x| x == base)
    }

    pub fn can_advance(&self, base: u8) -> bool {
        !self.occupied(base + 1)
    }

    pub fn at(&self, base: u8) -> Option<Uuid> {
        self.iter().find(|x| x.base == base).map(|x| x.id)
    }

    pub fn advance(&mut self, base: u8) {
        for &mut r in self.bases.iter_mut() {
            if r == base {
                r += 1;
            }
        }
    }

    pub fn remove(&mut self, base: u8) -> Option<Uuid> {
        let idx = self
            .bases
            .iter()
            .enumerate()
            .find(|(_, &x)| x == base)
            .map(|x| x.0);

        if let Some(idx) = idx {
            self.bases.remove(idx);
            let runner = self.ids.remove(idx);
            Some(runner)
        } else {
            None
        }
    }

    pub fn advance_all(&mut self, amount: u8) {
        for &mut r in self.bases.iter_mut() {
            r += amount;
        }
    }

    pub fn walk(&mut self) {
        // TODO: This code is even worse somehow
        let sorted = self.bases.iter().sorted_by(|a, b| Ord::cmp(a, b));
        let mut index = 0;
        for &i in sorted {
            if i == index {
                index += 1;
            }
        }

        for i in (0..index).rev() {
            self.advance(i);
        }
    }

    pub fn advance_if(&mut self, mut f: impl FnMut(&Baserunner) -> bool) {
        for i in self.iter() {
            if (self.can_advance(i.base) && f(i)) {
                self.advance(i.base);
            }
        }
    }

    pub fn push(&mut self, runner: Baserunner) {
        self.ids.push(runner.id);
        self.bases.push(runner.base);
    }

    pub fn iter(&self) -> impl Iterator<Item = BaserunnerRef> {
        zip(self.ids.iter(), self.bases.iter()).map(|x| BaserunnerRef { id: x.0, base: x.1 })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = BaserunnerRefMut> {
        zip(self.ids.iter_mut(), self.bases.iter_mut())
            .map(|x| BaserunnerRefMut { id: x.0, base: x.1 })
    }
}
