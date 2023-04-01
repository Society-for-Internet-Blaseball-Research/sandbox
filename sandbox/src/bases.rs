use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Baserunner {
    pub id: Uuid,
    pub base: u8,
}

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
        self.runners
            .iter()
            .find(|x| x.base == base)
            .map(|x| x.id.clone())
    }

    pub fn advance(&mut self, base: u8) {
        for r in self.runners.iter_mut() {
            if r.base == base {
                r.base += 1;
                return;
            }
        }
    }

    pub fn remove(&mut self, base: u8) -> Option<Uuid> {
        let idx = self
            .runners
            .iter()
            .enumerate()
            .find(|x| x.1.base == base)
            .map(|x| x.0);

        if let Some(idx) = idx {
            let runner = self.runners.remove(idx);
            Some(runner.id)
        } else {
            None
        }
    }

    pub fn advance_all(&mut self, amount: u8) {
        for r in self.runners.iter_mut() {
            r.base += amount;
        }
    }

    pub fn walk(&mut self) {
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

    pub fn advance_if(&mut self, mut f: impl FnMut(&Baserunner) -> bool) {
        for i in 0..self.runners.len() {
            if self.can_advance(self.runners[i].base) {
                if f(&self.runners[i]) {
                    self.runners[i].base += 1;
                }
            }
        }
    }

    pub fn add(&mut self, base: u8, id: Uuid) {
        self.runners.push(Baserunner { id, base });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Baserunner> {
        self.runners.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Baserunner> {
        self.runners.iter_mut()
    }
}
