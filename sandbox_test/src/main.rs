use sandbox::{
    bases::Baserunners,
    entities::{Player, Team, World},
    mods::Mods,
    rng::Rng,
    sim::{Event, Sim},
    Game, GameTeam,
};
use uuid::Uuid;

fn main() {
    let mut rng = Rng::new(69, 420);

    let mut world = World::new();
    let team_a = gen_team(&mut world, &mut rng, "Team A".to_string(), "A".to_string());
    let team_b = gen_team(&mut world, &mut rng, "Team B".to_string(), "B".to_string());

    let mut game = Game {
        weather: sandbox::Weather::Sun2,
        top: true,
        inning: 1,
        home_team: GameTeam {
            id: team_a,
            pitcher: world.team(team_a).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        away_team: GameTeam {
            id: team_b,
            pitcher: world.team(team_b).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        balls: 0,
        strikes: 0,
        outs: 0,
        runners: Baserunners::new(),
    };

    loop {
        let mut sim = Sim::new(&mut world, &mut rng);
        let evt = sim.next(&game);

        // keeping sim outside the loop means it borrows world and we can't pass it as mut here, which might be fine...?
        evt.apply(&mut game, &mut world);

        if let Event::GameOver = evt {
            println!("game over!");
            break;
        }

        let base = format!(
            "[{}|{}|{}]",
            if game.runners.occupied(2) { "X" } else { " " },
            if game.runners.occupied(1) { "X" } else { " " },
            if game.runners.occupied(0) { "X" } else { " " }
        );

        println!(
            "{}{} {}@{} ({}b/{}s/{}o) {} {:?}",
            if game.top { "t" } else { "b" },
            game.inning,
            game.away_team.score,
            game.home_team.score,
            game.balls,
            game.strikes,
            game.outs,
            base,
            evt
        );
    }

    // println!("Hello, world!");
}

fn gen_team(world: &mut World, rng: &mut Rng, name: String, emoji: String) -> Uuid {
    let id = Uuid::new_v4();
    let mut team = Team {
        id,
        emoji: emoji,
        lineup: Vec::new(),
        rotation: Vec::new(),
        shadows: Vec::new(),
        name: name,
        mods: Mods::new(),
    };

    for _ in 0..9 {
        team.lineup.push(gen_player(world, rng));
    }

    for _ in 0..5 {
        team.rotation.push(gen_player(world, rng));
    }

    for _ in 0..11 {
        team.shadows.push(gen_player(world, rng));
    }

    world.insert_team(team);
    id
}

fn gen_player(world: &mut World, rng: &mut Rng) -> Uuid {
    let id = Uuid::new_v4();

    let name = format!("Player {}", &id.to_string()[..8]);
    let player = Player {
        id: id,
        name,
        mods: Mods::new(),

        // this is not rng order compatible
        buoyancy: rng.next(),
        divinity: rng.next(),
        martyrdom: rng.next(),
        moxie: rng.next(),
        musclitude: rng.next(),
        patheticism: rng.next(),
        thwackability: rng.next(),
        tragicness: rng.next(),
        coldness: rng.next(),
        overpowerment: rng.next(),
        ruthlessness: rng.next(),
        shakespearianism: rng.next(),
        suppression: rng.next(),
        unthwackability: rng.next(),
        base_thirst: rng.next(),
        continuation: rng.next(),
        ground_friction: rng.next(),
        indulgence: rng.next(),
        laserlikeness: rng.next(),
        anticapitalism: rng.next(),
        chasiness: rng.next(),
        omniscience: rng.next(),
        tenaciousness: rng.next(),
        watchfulness: rng.next(),
        pressurization: rng.next(),
        cinnamon: rng.next(),
    };
    world.insert_player(player);
    id
}
