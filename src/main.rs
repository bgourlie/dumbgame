use legion::prelude::*;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::pipeline::CollisionGroups;
use ncollide2d::world::CollisionWorld;
use quicksilver::prelude::*;

struct Game {
    _view: Rectangle,
    _universe: Universe,
    world: World,
    collision_world: CollisionWorld<f32, Entity>,
    ships: CollisionGroups,
    bullets: CollisionGroups,
}

type BoundingBox = AABB<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MyCircle {
    radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MySquare {
    height: f32,
    width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bullet;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Player;

impl Player {
    fn handle_event(event: &Event, world: &mut World) {
        Self::handle_move_event(event, world);
        Self::handle_shoot_event(event, world);
    }

    fn handle_shoot_event(event: &Event, world: &mut World) {
        if let Event::Key(Key::Space, ButtonState::Pressed) = event {
            let (pos_x, pos_y) = <(Tagged<Player>, Read<Position>)>::query()
                .iter(&world)
                .nth(0)
                .map(|(_player, pos)| (pos.x, pos.y))
                .unwrap();

            world.insert(
                ((), Bullet),
                Some((
                    MySquare {
                        height: 5.,
                        width: 5.,
                    },
                    Position { x: pos_x, y: pos_y },
                    Velocity { dx: 5.0, dy: 0. },
                )),
            );
        }
    }

    fn handle_move_event(event: &Event, world: &World) {
        if let Event::Key(key, state) = event {
            match state {
                ButtonState::Pressed => Some(1.),
                ButtonState::Released => Some(0.),
                _ => None,
            }
            .and_then(|multiplier| {
                <(Tagged<Player>, Write<Velocity>)>::query().for_each(
                    world,
                    |(_player, mut vel)| match key {
                        Key::W => vel.dy = -1. * multiplier,
                        Key::A => vel.dx = -1. * multiplier,
                        Key::S => vel.dy = 1. * multiplier,
                        Key::D => vel.dx = 1. * multiplier,
                        _ => (),
                    },
                );
                Some(())
            })
            .unwrap_or(());
        }
    }
}

impl State for Game {
    fn new() -> Result<Self> {
        let universe = Universe::new();
        let mut world = universe.create_world();

        world.insert(
            ((), Player),
            Some((
                MyCircle { radius: 15. },
                Position { x: 200., y: 200. },
                Velocity { dx: 0., dy: 0. },
            )),
        );

        world.insert(
            (),
            Some((
                MyCircle { radius: 20.0 },
                Position { x: 100.0, y: 100.0 },
                Velocity { dx: 0.1, dy: 0.25 },
            )),
        );
        world.insert(
            (),
            Some((
                MySquare {
                    height: 20.0,
                    width: 20.0,
                },
                Position { x: 100.0, y: 100.0 },
                Velocity { dx: 0.3, dy: 0.1 },
            )),
        );

        let collision_world = CollisionWorld::new(0.02);
        let mut ships = CollisionGroups::new();
        ships.set_membership(&[0]);
        let mut bullets = CollisionGroups::new();
        bullets.set_membership(&[1]);

        Ok(Game {
            _view: Rectangle::new_sized((800, 600)),
            _universe: universe,
            world,
            collision_world,
            ships,
            bullets,
        })
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        let mut query = <(Write<Position>, Read<Velocity>)>::query();

        for (mut pos, vel) in query.iter(&self.world) {
            pos.x += vel.dx;
            pos.y += vel.dy;
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        Player::handle_event(event, &mut self.world);

        match event {
            Event::MouseButton(MouseButton::Left, ButtonState::Pressed) => {
                let pos = window.mouse().pos();

                self.world.insert(
                    (),
                    Some((
                        MySquare {
                            height: 20.0,
                            width: 20.0,
                        },
                        BoundingBox::from_half_extents(
                            Point2::new(pos.x, pos.y),
                            Vector2::new(10.0, 10.0),
                        ),
                        Position { x: pos.x, y: pos.y },
                        Velocity { dx: 0.3, dy: 0.1 },
                    )),
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let mut query = <(Read<Position>, Read<MyCircle>)>::query();
        // Remove any lingering artifacts from the previous frame
        window.clear(Color::BLACK)?;
        // Draw a rectangle with a top-left corner at (100, 100) and a width and height of 32 with
        // a blue background
        for (pos, circle) in query.iter(&self.world) {
            window.draw(
                &Circle::new((pos.x, pos.y), circle.radius),
                Col(Color::GREEN),
            );
        }

        let mut query = <(Read<Position>, Read<MySquare>)>::query();
        for (pos, square) in query.iter(&self.world) {
            window.draw(
                &Rectangle::new((pos.x, pos.y), (square.width, square.height)),
                Col(Color::GREEN),
            );
        }
        Ok(())
    }
}

fn main() {
    run::<Game>(
        "Dumb Game",
        Vector::new(800, 600),
        Settings {
            multisampling: Some(4),
            ..Settings::default()
        },
    );
}
