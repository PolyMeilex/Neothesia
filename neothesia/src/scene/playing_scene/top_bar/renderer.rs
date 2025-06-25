use iced_core::{
    alignment::Vertical,
    border::Radius,
    font::{Family, Weight},
    renderer::Quad,
    text::{Alignment, LineHeight, Renderer as _, Shaping, Wrapping},
    Background, Border, Color, Font, Pixels, Point, Rectangle, Renderer as _, Size, Text,
};

pub struct NuonRenderer<'a> {
    pub renderer: &'a mut iced_wgpu::Renderer,
}

impl nuon::Renderer for NuonRenderer<'_> {
    #[profiling::function]
    fn rounded_quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: nuon::Color,
        border_radius: [f32; 4],
    ) {
        self.renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x,
                    y,
                    width: w,
                    height: h,
                },
                border: Border {
                    radius: Radius {
                        top_left: border_radius[0],
                        top_right: border_radius[1],
                        bottom_right: border_radius[3],
                        bottom_left: border_radius[2],
                    },
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: iced_core::Shadow::default(),
            },
            Background::Color(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            }),
        );
    }

    #[profiling::function]
    fn icon(&mut self, x: f32, y: f32, size: f32, icon: &str) {
        self.renderer.fill_text(
            Text {
                content: icon.to_string(),
                bounds: Size::new(f32::MAX, f32::MAX),
                size: Pixels(size),
                line_height: LineHeight::Absolute(Pixels(size)),
                font: Font {
                    family: Family::Name("bootstrap-icons"),
                    ..Font::DEFAULT
                },
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            },
            Point::new(x, y),
            Color::WHITE,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: f32::MAX,
                height: f32::MAX,
            },
        );
    }

    #[profiling::function]
    fn centered_text(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text: &str) {
        self.renderer.fill_text(
            Text {
                content: text.to_string(),
                bounds: Size::new(f32::MAX, f32::MAX),
                size: Pixels(size),
                line_height: LineHeight::Absolute(Pixels(size)),
                font: Font {
                    family: Family::Name("Roboto"),
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                },
                align_x: Alignment::Center,
                align_y: Vertical::Center,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            },
            Point::new(x + w / 2.0, y + h / 2.0),
            Color::WHITE,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: f32::MAX,
                height: f32::MAX,
            },
        );
    }
}
