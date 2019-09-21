use legion::prelude::*;
use quicksilver::prelude::*;
use std::time::Instant;

struct Game {
    view: Rectangle,
    universe: Universe,
    world: World,
    last_draw: Instant,
    last_update: Instant,
}

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

impl State for Game {
    fn new() -> Result<Self> {
        let universe = Universe::new();
        let mut world = universe.create_world();
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
        Ok(Game {
            view: Rectangle::new_sized((800, 600)),
            last_draw: Instant::now(),
            last_update: Instant::now(),
            universe,
            world,
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
        let draw_time = Instant::now();
        let delta_t = draw_time - self.last_draw;
        self.last_draw = draw_time;

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
