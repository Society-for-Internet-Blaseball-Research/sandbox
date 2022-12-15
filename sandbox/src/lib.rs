use std::cell::RefCell;

// whats baseball
const STRIKES_FOR_STRIKEOUT: u8 = 3;
const BALLS_FOR_WALK: u8  = 3;
const OUTS_FOR_SWITCH: u8  = 3;

struct Event {
    event: String
}

impl Event {
    pub fn new(inner: String) -> Event {
        Event {
            event: inner
        }
    }
}

struct GameTeam {
    batters: Vec<Player>,
    batter_index: RefCell<usize>,
    pitcher: Player,
    score: i32,
}

impl GameTeam {
    pub fn next_batter(&self) -> &Player {
        let player = &self.batters[*self.batter_index.borrow()];
        *self.batter_index.borrow_mut() += 1;
        if *self.batter_index.borrow() >= self.batters.len() {
            *self.batter_index.borrow_mut() = 0;
        }

        player
    }
}

struct Player {
    name: String,
    id: PlayerId
}

impl Player {
    pub fn in_strike_zone(&self, rng: &mut impl RandomSource) -> bool {
        rng.next_roll() >= 0.5
    }

    pub fn swing(&self, rng: &mut impl RandomSource, in_strike_zone: bool) -> bool {
        if in_strike_zone {
            rng.next_roll() >= 0.5
        } else {
            rng.next_roll() >= 0.5
        }
    }

    pub fn contact(&self, rng: &mut impl RandomSource, in_strike_zone: bool) -> bool {
        if in_strike_zone {
            rng.next_roll() >= 0.5
        } else {
            rng.next_roll() >= 0.5
        }
    }
}

type PlayerId = u32;

trait RandomSource {
    fn next_roll(&mut self) -> f64;
}

struct GameState<R: RandomSource> {
    out_count: u8,
    ball_count: u8,
    strike_count: u8,
    bases: [Option<PlayerId>; 4],
    home: GameTeam,
    away: GameTeam,
    inning: i32,
    top_half: bool,
    rng: R
}

impl<R: RandomSource> GameState<R> {
    pub fn tick(&mut self) -> Option<Event> {
        let batter = if self.top_half { self.away.next_batter() } else { self.home.next_batter() };
        let pitcher = if self.top_half { &self.away.pitcher } else { &self.home.pitcher };

        let in_strike_zone = pitcher.in_strike_zone(&mut self.rng);
        let batter_swung = batter.swing(&mut self.rng, in_strike_zone);

        if !batter_swung {
            if !in_strike_zone {
                self.ball_count += 1;
                if self.ball_count == BALLS_FOR_WALK {
                    return Some(Event::new("walkies".to_string()))
                } else {
                    return Some(Event::new("ball!".to_string()))
                }
            }

            self.strike_count += 1;
            if self.strike_count == STRIKES_FOR_STRIKEOUT {
                self.out_count += 1;
                return Some(Event::new("strikeout lookin'".to_string()))
            } else {
                return Some(Event::new("strike'".to_string()))
            }
        }

        let contact = batter.contact(&mut self.rng, in_strike_zone);
        if !contact {
            self.strike_count += 1;
            if self.strike_count == STRIKES_FOR_STRIKEOUT {
                self.out_count += 1;
                return Some(Event::new("strikeout lookin'".to_string()))
            } else {
                return Some(Event::new("strike swingin'".to_string()))
            }
        }


        
        todo!()
    }


}
