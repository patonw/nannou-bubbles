use nannou::{rand, prelude::*};
use palette::named;
use structopt::StructOpt;
use lazy_static::lazy_static;
use log::*;
use typed_builder::TypedBuilder;
use glam::f32::Vec2;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value="1.0")]
    rate: f32,

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
    fn update(&mut self);
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
    dest: Point,
    #[builder(default=10.0)]
    radius: f32,
    #[builder(default=200.0)]
    max_radius: f32,
    #[builder(default=OPTS.rate)]
    speed: f32,
    #[builder(default=OPTS.rate)]
    growth_rate: f32,
}

impl Nannou for Dot {
    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .color(self.color)
            .w(self.radius)
            .h(self.radius)
            .x_y(self.origin.x, self.origin.y);
    }

    fn update(&mut self) {
        if self.radius < self.max_radius {
            self.radius += self.growth_rate;
        }

        let delta = self.dest - self.origin;
        if delta.length() > self.speed {
            self.origin += delta.normalize() * self.speed;
        }
        else {
            self.origin = self.dest;
        }
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

    fn update(&mut self) {
        self.dots.iter_mut().for_each(|d| d.update());
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
            dots: (0..OPTS.num_dots)
                .map(|i| i as f32)
                .map(|_| Dot::builder()
                     .color(random_color())
                     .origin(rand_point())
                     .dest(rand_point())
                     .max_radius(rand::random_range(20.0, 500.0))
                     .speed(rand::random_range(1.0, 20.0).ln())
                     .growth_rate(rand::random_range(1.0, 20.0).ln())
                     .build())
                .collect(),
        }
    }
}

fn model(_app: &App) -> Model {
    Model::default()
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    model.display(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    pretty_env_logger::init();
    info!("Hello world!");
    nannou::app(model).update(update).simple_window(view).run();
}

