use super::Color;

pub trait Renderer {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.rounded_quad(x, y, w, h, color, [0.0; 4])
    }

    fn rounded_quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        border_radius: [f32; 4],
    );

    fn icon(&mut self, x: f32, y: f32, size: f32, icon: &str);

    fn centered_text_bold(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text: &str);
    fn centered_text(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text: &str);
}
