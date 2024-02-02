use std::time::Duration;
use nannou::{rand, prelude::*};
use palette::named;
use structopt::StructOpt;
use lazy_static::lazy_static;
use log::*;
use typed_builder::TypedBuilder;
use rand_distr::{Distribution, Gamma};
use histo::Histogram;

use nannou_egui::{self, egui, Egui};
use egui_plot::{Plot, Bar, BarChart};

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
    fn update(&mut self, update: &Update);
}

fn as_nn(c: palette::rgb::Rgb<palette::encoding::Srgb, u8>) -> Rgb {
    rgb(c.red, c.green, c.blue)
}

fn from_str(name: &str) -> Option<Rgb> {
    named::from_str(name).map(|v| as_nn(v))
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black,
    DarkGray,
    DimGray,
    Honeydew,
    SteelBlue,
    SlateGray,
    Silver,
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
    Point::new(rand::random_range(-500.0, 500.0), rand::random_range(-500.0, 500.0))
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

    fn update(&mut self, update: &Update) {
        let delta = update.since_last;
        self.ttl = self.ttl.checked_sub(delta).unwrap_or(Duration::ZERO);

        let delta = delta.as_secs_f32();
        if self.radius < self.max_radius {
            self.radius += self.growth_rate * delta;
        }

        let offset = self.origin - self.pivot;
        let step = self.speed * delta;
        self.origin = self.pivot + offset.rotate(step);
    }
}

#[derive(Debug, Copy, Clone)]
struct Settings {
    paused: bool,
    bg_color: Rgb,
    max_count: u8,
    max_speed: f32,
    max_rate: f32,
    scale: f32,
    shape: f32,
}

struct Model {
    egui: Egui,
    settings: Settings,
    dots: Vec<Dot>,
    x_limit: u64,
}

impl Nannou for Model {
    fn display(&self, draw: &Draw) {
        draw.background()
            .color(self.settings.bg_color);

        self.dots.iter().for_each(|d| d.display(draw));
    }

    fn update(&mut self, update: &Update) {
        let egui = &mut self.egui;
        let settings = &mut self.settings;

        egui.set_elapsed_time(update.since_start);

        let ctx = egui.begin_frame();
        let mut dump = false;

        egui::Window::new("Settings")
            .anchor(egui::Align2::LEFT_TOP, (0.0, 0.0))
            .show(&ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    dump = ui.button("Dump").clicked();

                    if ui.button("Clear").clicked() {
                        self.dots.clear();
                    }

                    let paused = settings.paused;
                    ui.toggle_value(&mut settings.paused, if paused {"Resume" } else {"Pause"});
                });

                ui.label("Max Dots:");
                ui.add(egui::Slider::new(&mut settings.max_count, 1..=255));

                ui.label("Max Speed:");
                ui.add(egui::Slider::new(&mut settings.max_speed, 0.0..=10.0));

                ui.label("Growth Rate:");
                ui.add(egui::Slider::new(&mut settings.max_rate, 0.0..=1000.0));

                ui.add_space(16.0);
                ui.heading("Radius Distribution");

                ui.label("Shape");
                ui.add(egui::Slider::new(&mut settings.shape, 1.0..=500.0)
                       .logarithmic(true));

                ui.label("Scale");
                ui.add(egui::Slider::new(&mut settings.scale, 1.0..=500.0)
                       .logarithmic(true));
            });


        egui::Window::new("Speed")
            .anchor(egui::Align2::RIGHT_TOP, (0.0, 0.0))
            .show(&ctx, |ui| {
                Plot::new("Dist")
                    .view_aspect(1.5)
                    .include_x(0.0)
                    .include_x(settings.max_speed)
                    .include_y(20.0)
                    .y_axis_width(2)
                    .show(ui, |plt| {
                        const SCALE: f64 = 1000.0;
                        let mut hist = Histogram::with_buckets(10);
                        for d in self.dots.iter() {
                            let speed = d.speed.abs() as f64 * SCALE;
                            hist.add(speed as u64);
                        }

                        let bars = hist.buckets().map(|b| {
                            let center = (b.start() + b.end()) / 2;
                            let width = b.end() - b.start();

                            let center = center as f64 / SCALE;
                            let width = width as f64 / SCALE;


                            Bar::new(center as f64, b.count() as f64)
                                .width(0.5 * width as f64)
                        }).collect::<Vec<_>>();

                        let chart1 = BarChart::new(bars)
                            .name("Current");
                        plt.bar_chart(chart1);
                    });
            });

        let x_limit = self.x_limit as f64 * 0.995;
        egui::Window::new("Radius")
            .anchor(egui::Align2::RIGHT_BOTTOM, (0.0, 0.0))
            .show(&ctx, |ui| {
                Plot::new("Dist")
                    .legend(Default::default())
                    .view_aspect(1.5)
                    .include_x(0.0)
                    .include_x(x_limit)
                    .include_y(50.0)
                    .y_axis_width(2)
                    .show(ui, |plt| {
                        let mut hist = Histogram::with_buckets(10);
                        for d in self.dots.iter() {
                            hist.add(d.radius as u64);
                        }

                        let x_max1 = hist.buckets().map(|b| b.end()).max().unwrap_or(0);


                        let bars = hist.buckets().map(|b| {
                            let center = (b.start() + b.end()) / 2;
                            let width = b.end() - b.start();

                            Bar::new(center as f64, b.count() as f64)
                                .width(0.5 * width as f64)
                        }).collect::<Vec<_>>();

                        let chart1 = BarChart::new(bars)
                            .name("Current");
                        plt.bar_chart(chart1);

                        let mut hist = Histogram::with_buckets(10);
                        for d in self.dots.iter() {
                            hist.add(d.max_radius as u64);
                        }

                        let x_max2 = hist.buckets().map(|b| b.end()).max().unwrap_or(0);

                        let bars = hist.buckets().map(|b| {
                            let center = (b.start() + b.end()) / 2;
                            let width = b.end() - b.start();
                            Bar::new(center as f64, b.count() as f64)
                                .width(0.5 * width as f64)
                        }).collect::<Vec<_>>();

                        let chart1 = BarChart::new(bars)
                            .name("Maximum");
                        plt.bar_chart(chart1);

                        self.x_limit = vec![x_limit as u64, x_max1, x_max2].into_iter().max().unwrap_or(100);
                    });
            });

        if dump {
            // Some debugging data
            dbg!(&settings, &self.dots);
        }

        if settings.paused {
            return
        }

        self.dots.iter_mut().for_each(|d| d.update(update));
        self.dots.retain(|d| d.ttl > Duration::ZERO && d.radius < d.max_radius);

        let radius_dist = Gamma::new(settings.shape, settings.scale).unwrap();
        let max_radius: f32 = radius_dist.sample(&mut rand::thread_rng());
        let max_radius = max_radius.clamp(0.0, 512.0);

        if self.dots.len() < settings.max_count.into() {
            self.dots.push(
                Dot::builder()
                .color(random_color())
                .origin(rand_point())
                .pivot(rand_point())
                .max_radius(max_radius)
                .speed(rand::random_range(-settings.max_speed, settings.max_speed))
                .growth_rate(rand::random_range(1.0, settings.max_rate))
                .ttl(Duration::from_secs_f32(rand::random_range(1.0, 10.0)))
                .build());
        }
    }
}

fn random_color() -> Rgba {
    rgba(
        rand::random_range(0, 128),
        rand::random_range(0, 255),
        rand::random_range(0, 255),
        rand::random_range(128, 255),
    )
}

fn model(app: &App) -> Model {
    let wid = app.new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(wid).unwrap();
    let egui = Egui::from_window(&window);

    let settings = Settings {
        bg_color: Color::DimGray.into(),
        paused: false,
        max_count: OPTS.num_dots.into(),
        max_speed: OPTS.speed,
        max_rate: OPTS.rate,
        scale: 10.0,
        shape: 10.0,
    };

    Model {
        egui,
        settings,
        dots: Vec::new(),
        x_limit: 100,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.update(&update);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    model.display(&draw);
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn main() {
    pretty_env_logger::init();
    info!("Options: {:?}", *OPTS);
    nannou::app(model)
        .update(update)
        .run();
}

