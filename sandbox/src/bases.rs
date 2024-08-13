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

    pub fn contains(&self, id: Uuid) -> bool {
        self.runners.iter().any(|x| x.id == id)
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

    pub fn walk_instincts(&mut self, third: bool) {
        //todo: the runners who score end up on the wrong base
        //does this cause problems?
        if third {
            self.advance_all(3);
        } else {
            if self.occupied(0) {
                self.advance_all(2);
            } else if self.occupied(1) {
                self.advance_all(1);
            }
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

    pub fn forced_advance_if(&mut self, mut f: impl FnMut(&Baserunner) -> bool) {
        if self.occupied(0) && self.occupied(1) {
            for runner in self.runners.iter_mut() {
                runner.base += 1;
            }
        } else {
            for i in 0..self.runners.len() {
                if self.can_advance(self.runners[i].base) {
                    if f(&self.runners[i]) {
                        self.runners[i].base += 1;
                    } else if self.runners[i].base == 0 {
                        self.walk(); //this is a code crime
                    }
                }
            }
        }
    }

    pub fn add(&mut self, base: u8, id: Uuid) {
        self.runners.push(Baserunner { id, base });
    }

    pub fn empty(&self) -> bool {
        self.runners.len() == 0
    }

    pub fn pick_runner(&self, roll: f64) -> u8 {
        //todo: maybe rewrite this to use an if
        let len = self.runners.len();
        match len {
            0 => {
                panic!("this shouldn't be called");
            },
            1 => {
                return self.runners[0].base;
            },
            _ => {
                let idx = (roll * (len as f64)).floor() as usize;
                return self.runners[idx].base;
            }
        }
    }

    pub fn pick_runner_fc(&self) -> u8 {
        if self.occupied(1) {
            if self.occupied(2) {
                return 2;
            } else {
                return 1;
            }
        } else {
            return 0;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Baserunner> {
        self.runners.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Baserunner> {
        self.runners.iter_mut()
    }
}
