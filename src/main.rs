mod prelude;

use fnv::FnvHashMap;
use legion::prelude::*;
use nalgebra::Vector2;
use ncollide2d::pipeline::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType};
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use prelude::*;
use quicksilver::prelude::*;

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
    mobs: CollisionGroups,
    bullets: CollisionGroups,
    collision_object_handles: FnvHashMap<Entity, CollisionObjectSlabHandle>,
}

impl Game {
    fn spawn_mob(&mut self, pos: impl Into<Position>) {
        let pos = pos.into();
        let entity = self
            .world
            .insert(((), Mob), Some((pos, Velocity::new(0.3, 0.1))))[0];

        let query_type = GeometricQueryType::Proximity(0.0);
        let cuboid = Cuboid::new(Vector2::new(Mob::HALF_EXTENT, Mob::HALF_EXTENT));
        let shape = ShapeHandle::new(cuboid);
        let (handle, _) =
            self.collision_world
                .add(pos.isometry(), shape, self.mobs, query_type, entity);
        self.collision_object_handles.insert(entity, handle);
    }

    fn spawn_player(&mut self) {
        let center = Position::new(200., 200.);
        self.world
            .insert(((), Player), Some((center, Velocity::new(0., 0.))));
    }

    fn spawn_bullet(&mut self) {
        let pos = <(Tagged<Player>, Read<Position>)>::query()
            .iter(&self.world)
            .map(|(_player, pos)| *pos)
            .nth(0)
            .unwrap();

        let entity = self
            .world
            .insert(((), Bullet), Some((pos, Velocity::new(5., 0.))))[0];

        let query_type = GeometricQueryType::Proximity(0.0);
        let ball = Ball::new(Bullet::RADIUS);
        let shape = ShapeHandle::new(ball);
        let (handle, _) =
            self.collision_world
                .add(pos.isometry(), shape, self.bullets, query_type, entity);
        self.collision_object_handles.insert(entity, handle);
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
                        Key::W => *vel += Velocity::new(0., -1. * multiplier),
                        Key::A => *vel += Velocity::new(-1. * multiplier, 0.),
                        Key::S => *vel += Velocity::new(0., 1. * multiplier),
                        Key::D => *vel += Velocity::new(1. * multiplier, 0.),
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
        let world = universe.create_world();

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
            mobs: ships,
            bullets,
            collision_object_handles: FnvHashMap::default(),
        };

        game.spawn_player();
        game.spawn_mob(Position::new(10., 10.));
        Ok(game)
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        let mut query = <(Write<Position>, Read<Velocity>)>::query();

        for (entity, (mut pos, vel)) in query.iter_entities(&self.world) {
            *pos += *vel;
            if let Some(handle) = self.collision_object_handles.get(&entity) {
                if let Some(object) = self.collision_world.objects.get_mut(*handle) {
                    object.set_position(pos.isometry())
                }
            }
        }
        self.collision_world.update();

        let mut bang_count = 0;
        for e in self.collision_world.proximity_events() {
            bang_count += 1;
            println!("bang {}!", bang_count);
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
            window.draw(&Circle::new(*pos, Player::RADIUS), Col(Color::GREEN));
        }

        for (_bullet, pos) in <(Tagged<Bullet>, Read<Position>)>::query().iter(&self.world) {
            window.draw(&Circle::new(*pos, Bullet::RADIUS), Col(Color::GREEN));
        }

        for (_mob, pos) in <(Tagged<Mob>, Read<Position>)>::query().iter(&self.world) {
            window.draw(
                &Rectangle::new(*pos, (Mob::HALF_EXTENT * 2., Mob::HALF_EXTENT * 2.)),
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
