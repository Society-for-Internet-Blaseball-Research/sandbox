pub struct Rng {
    s0: u64,
    s1: u64,
}

impl Rng {
    pub fn new(s0: u64, s1: u64) -> Rng {
        Rng { s0, s1 }
    }

    fn step(&mut self) {
        let mut s1 = self.s0;
        let s0 = self.s1;
        s1 ^= s1 << 23;
        s1 ^= s1 >> 17;
        s1 ^= s0;
        s1 ^= s0 >> 26;
        self.s0 = self.s1;
        self.s1 = s1;
    }

    pub fn next(&mut self) -> f64 {
        self.step();

        f64::from_bits((self.s0 >> 12) | 0x3FF0000000000000) - 1.0
    }

    pub fn index(&mut self, len: usize) -> usize {
        (self.next() * len as f64).floor() as usize
    }
}
