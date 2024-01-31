use std::time::Duration;
use nannou::{rand, prelude::*};
use palette::named;
use structopt::StructOpt;
use lazy_static::lazy_static;
use log::*;
use typed_builder::TypedBuilder;
use glam::f32::Vec2;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// Maximum angular velocity in radians per second
    #[structopt(short, long, default_value="1.0")]
    speed: f32,

    /// Maximum bubble growth rate in pixels per second
    #[structopt(short, long, default_value="100.0")]
    rate: f32,

    /// Maximum bubbles to render simultaneously
    #[structopt(short, long, default_value="1")]
    num_dots: u8,
}

lazy_static! {
    pub static ref OPTS: Opts = Opts::from_args();
}

type Rgb = Srgb<u8>;
type Rgba = Srgba<u8>;

trait Nannou {
    fn display(&self, draw: &Draw);
    fn update(&mut self, delta: &Duration);
}

fn as_nn(c: palette::rgb::Rgb<palette::encoding::Srgb, u8>) -> Rgb {
    rgb(c.red, c.green, c.blue)
}

fn from_str(name: &str) -> Option<Rgb> {
    named::from_str(name).map(|v| as_nn(v))
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Honeydew,
    SteelBlue,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

impl From<Color> for Rgb {
    fn from(c: Color) -> Self {
        from_str(&c.to_string()).expect("Unknown color {}")
    }
}

type Point = Vec2;

fn rand_point() -> Point {
    Point {
        x: rand::random_range(-500.0, 500.0),
        y: rand::random_range(-500.0, 500.0),
    }
}

#[derive(Debug, Clone, Copy, TypedBuilder)]
struct Dot {
    #[builder(setter(into))]
    color: Rgba,
    #[builder(setter(into), default)]
    origin: Point,
    #[builder(setter(into), default)]
    pivot: Point,
    #[builder(default=10.0)]
    radius: f32,
    #[builder(default=200.0)]
    max_radius: f32,
    #[builder(default=OPTS.rate)]
    speed: f32,
    #[builder(default=OPTS.rate)]
    growth_rate: f32,
    #[builder(default=Duration::from_secs(10))]
    ttl: Duration,
}

impl Nannou for Dot {
    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .color(self.color)
            .w(self.radius)
            .h(self.radius)
            .x_y(self.origin.x, self.origin.y);
    }

    fn update(&mut self, delta: &Duration) {
        self.ttl = self.ttl.checked_sub(*delta).unwrap_or(Duration::ZERO);

        let delta = delta.as_secs_f32();
        if self.radius < self.max_radius {
            self.radius += self.growth_rate * delta;
        }

        let offset = self.pivot - self.origin;
        let step = self.speed * delta;
        self.origin = self.pivot + Vec2::from_angle(step).rotate(-offset);
    }
}

struct Model {
    bg_color: Rgb,
    dots: Vec<Dot>,
}

impl Nannou for Model {
    fn display(&self, draw: &Draw) {
        draw.background()
            .color(self.bg_color);

        self.dots.iter().for_each(|d| d.display(draw));
    }

    fn update(&mut self, delta: &Duration) {
        self.dots.iter_mut().for_each(|d| d.update(delta));
        self.dots.retain(|d| d.ttl > Duration::ZERO && d.radius < d.max_radius);

        if self.dots.len() < OPTS.num_dots.into() {
            self.dots.push(
                Dot::builder()
                .color(random_color())
                .origin(rand_point())
                .pivot(rand_point())
                .max_radius(rand::random_range(20.0, 500.0))
                .speed(rand::random_range(-OPTS.speed, OPTS.speed))
                .growth_rate(rand::random_range(1.0, OPTS.rate))
                .ttl(Duration::from_secs_f32(rand::random_range(1.0, 10.0)))
                .build());
        }
    }
}

fn random_color() -> Rgba {
    rgba(
        rand::random_range(0, 255),
        rand::random_range(0, 255),
        rand::random_range(0, 255),
        rand::random_range(0, 255),
    )
}

impl Default for Model {
    fn default() -> Self {
        Model {
            bg_color: Color::Honeydew.into(),
            dots: Vec::new(),
        }
    }
}

fn model(_app: &App) -> Model {
    Model::default()
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.update(&app.duration.since_prev_update);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    model.display(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    pretty_env_logger::init();
    info!("Options: {:?}", *OPTS);
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

