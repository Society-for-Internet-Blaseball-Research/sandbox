use std::{iter::zip, borrow::Borrow};

use uuid::Uuid;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Baserunner {
    pub id: Uuid,
    pub base: u8,
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
        let sorted = self.iter_mut().sorted_by(|a, b| Ord::cmp(a.1, b.1));
        let mut index = 0;
        for i in sorted {
            if *i.base == index {
                index += 1;
            }
        }
        // todo: this code is also crap
        let mut num_occupied = 0;
        for i in 0..5 {
            if self.occupied(i) {
                num_occupied += 1;
            } else {
                break;
            }
        }

        for i in (0..num_occupied).rev() {
            self.advance(i);
        }
    }

    pub fn advance_if(&mut self, mut f: impl FnMut((&)) -> bool) {
        for i in 0..self.runners.len() {
            if self.can_advance(self.runners[i].base) {
                if f(&self.runners[i]) {
                    self.runners[i].base += 1;
                }
            }
        }
    }

    pub fn add(&mut self, base: u8, id: Uuid) {
        self.ids.push(id);
        self.bases.push(base);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Baserunner> {
        zip(self.ids.iter(), self.bases.iter()).map(|x| Baserunner{id: x.0, base: x.1})
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Baserunner> {
        zip(self.ids.iter_mut(), self.bases.iter_mut()).map(|x| Baserunner{id: x.0, base: x.1})
    }
}
