use legion::prelude::*;
use nalgebra::{Isometry2, Matrix2, Point2, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::bounding_volume::BoundingSphere;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Ball, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use quicksilver::prelude::*;
use std::ops::{Mul, MulAssign};

type BoundingBox = AABB<f32>;
type Position = Point2<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity(Vector2<f32>);

impl Velocity {
    fn new(x: f32, y: f32) -> Velocity {
        Velocity(Vector2::new(x, y))
    }
}

impl Mul<Matrix2<f32>> for Velocity {
    type Output = Self;

    fn mul(self, rhs: Matrix2<f32>) -> Self::Output {
        let Velocity(vec) = self;
        Velocity(rhs.mul(vec))
    }
}
impl MulAssign<Matrix2<f32>> for Velocity {
    fn mul_assign(&mut self, rhs: Matrix2<f32>) {
        let Velocity(vec) = self;
        *self = Velocity(rhs.mul(*vec));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mob;

impl Mob {
    const HALF_EXTENT: f32 = 10.;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bullet;

impl Bullet {
    const RADIUS: f32 = 5.;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Player;

impl Player {
    const RADIUS: f32 = 15.;
}

struct Game {
    _view: Rectangle,
    _universe: Universe,
    world: World,
    collision_world: CollisionWorld<f32, Entity>,
    ships: CollisionGroups,
    bullets: CollisionGroups,
}

impl Game {
    fn spawn_mob(&mut self, pos: impl Into<Position>) {
        let half_extent = 10.;
        self.world
            .insert(((), Mob), Some((pos.into(), Velocity::new(0.3, 0.1))));
    }

    fn spawn_player(&mut self) {
        let center = Position::new(200., 200.);
        let bounding_sphere = BoundingSphere::new(center, Player::RADIUS);
        self.world.insert(
            ((), Player),
            Some((center, bounding_sphere, Velocity::new(0., 0.))),
        );
    }

    fn spawn_bullet(&mut self) {
        let pos = <(Tagged<Player>, Read<Position>)>::query()
            .iter(&self.world)
            .map(|(_player, pos)| *pos)
            .nth(0)
            .unwrap();

        let bounding_sphere = BoundingSphere::new(pos, Bullet::RADIUS);

        let entity = self.world.insert(
            ((), Bullet),
            Some((bounding_sphere, pos, Velocity::new(5., 0.))),
        )[0];

        //        let query_type = GeometricQueryType::Proximity(0.0);
        //        let shape = ShapeHandle::new(ball);
        //        self.collision_world.add(ball.), shape, self.bullets, query_type,  entity);
    }

    fn handle_event(&mut self, event: &Event) {
        self.handle_move_event(event);
        self.handle_shoot_event(event);
    }

    fn handle_shoot_event(&mut self, event: &Event) {
        if let Event::Key(Key::Space, ButtonState::Pressed) = event {
            self.spawn_bullet()
        }
    }

    fn handle_move_event(&mut self, event: &Event) {
        if let Event::Key(key, state) = event {
            match state {
                ButtonState::Pressed => Some(1.),
                ButtonState::Released => Some(0.),
                _ => None,
            }
            .and_then(|multiplier| {
                <(Tagged<Player>, Write<Velocity>)>::query().for_each(
                    &self.world,
                    |(_player, mut vel)| match key {
                        Key::W => *vel *= Matrix2::new(1., 0., -1. * multiplier, 0.),
                        Key::A => *vel *= Matrix2::new(-1. * multiplier, 0., 1., 0.),
                        Key::S => *vel *= Matrix2::new(1., 0., 1. * multiplier, 0.),
                        Key::D => *vel *= Matrix2::new(1. * multiplier, 0., 1., 0.),
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

        let collision_world = CollisionWorld::new(0.02);
        let mut ships = CollisionGroups::new();
        ships.set_membership(&[0]);
        let mut bullets = CollisionGroups::new();
        bullets.set_membership(&[1]);

        let mut game = Game {
            _view: Rectangle::new_sized((800, 600)),
            _universe: universe,
            world,
            collision_world,
            ships,
            bullets,
        };

        game.spawn_player();
        game.spawn_mob(Position::new(10., 10.));
        Ok(game)
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        let mut query = <(Write<Position>, Read<Velocity>)>::query();

        for (entity, (mut pos, vel)) in query.iter_entities(&self.world) {
            let Velocity(vel) = *vel;
            *pos += vel;
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        self.handle_event(event);

        match event {
            Event::MouseButton(MouseButton::Left, ButtonState::Pressed) => {
                let pos = window.mouse().pos();
                self.spawn_mob(pos.into_point());
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        for (_player, pos) in <(Tagged<Player>, Read<Position>)>::query().iter(&self.world) {
            window.draw(
                &Circle::new((pos.x, pos.y), Player::RADIUS),
                Col(Color::GREEN),
            );
        }

        for (_bullet, pos) in <(Tagged<Bullet>, Read<Position>)>::query().iter(&self.world) {
            window.draw(
                &Circle::new((pos.x, pos.y), Bullet::RADIUS),
                Col(Color::GREEN),
            );
        }

        for (_mob, pos) in <(Tagged<Mob>, Read<Position>)>::query().iter(&self.world) {
            window.draw(
                &Rectangle::new(
                    (pos.x, pos.y),
                    (Mob::HALF_EXTENT * 2., Mob::HALF_EXTENT * 2.),
                ),
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
