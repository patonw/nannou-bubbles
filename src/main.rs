use nannou::{prelude::*};
use palette::named;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

fn as_nn(c: palette::rgb::Rgb<palette::encoding::Srgb, u8>) -> rgb::Rgb<nannou::color::encoding::Srgb, u8> {
    rgb(c.red, c.green, c.blue)
}

fn from_str(name: &str) -> Option<Rgb<u8>> {
    named::from_str(name).map(|v| as_nn(v))
}

struct Model {
    bg_color: Rgb<u8>,
    x: f32,
    y: f32,
    radius: f32,
}

fn model(_app: &App) -> Model {
    let bg_color = from_str("honeydew").expect("Unknown color");

    Model {
        bg_color: bg_color,
        x: 0.0,
        y: 0.0,
        radius: 10.0,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.radius < 500.0 {
        model.radius += 1.0;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background()
        .color(model.bg_color);
    draw.ellipse()
        .color(STEELBLUE)
        .w(model.radius)
        .h(model.radius)
        .x_y(model.x, model.y);
    draw.to_frame(app, &frame).unwrap();
}
