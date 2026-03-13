use super::ScoreScene;
use crate::{context::Context, NeothesiaEvent};

pub(crate) fn render_score_ui(scene: &mut ScoreScene, ctx: &mut Context) {
    let win_w = ctx.window_state.logical_size.width;
    let win_h = ctx.window_state.logical_size.height;

    let panel_w = 700.0;
    let panel_h = 600.0;

    let mut ui = std::mem::replace(&mut scene.nuon, nuon::Ui::new());

    nuon::translate()
        .x((win_w - panel_w) / 2.0)
        .y((win_h - panel_h) / 2.0)
        .build(&mut ui, |ui| {
            nuon::quad()
                .size(panel_w, panel_h)
                .color([26, 26, 31])
                .border_radius([10.0; 4])
                .build(ui);

            let mut current_y = 20.0;

            nuon::translate().x(20.0).y(current_y).build(ui, |ui| {
                nuon::label()
                    .text("Song Complete!")
                    .font_size(40.0)
                    .font_family("Arial")
                    .color([1.0, 1.0, 1.0, 1.0])
                    .size(panel_w - 40.0, 50.0)
                    .text_justify(nuon::TextJustify::Center)
                    .build(ui);
            });

            current_y += 50.0;

            let grade_color = grade_color(&scene.score_data.grade);
            nuon::translate().x(20.0).y(current_y).build(ui, |ui| {
                nuon::label()
                    .text(&format!("Grade: {}", scene.score_data.grade))
                    .font_size(60.0)
                    .font_family("Arial")
                    .color(grade_color)
                    .size(panel_w - 40.0, 80.0)
                    .text_justify(nuon::TextJustify::Center)
                    .build(ui);
            });

            current_y += 80.0;

            nuon::translate().x(20.0).y(current_y).build(ui, |ui| {
                nuon::label()
                    .text(&format!(
                        "Accuracy: {:.0}% ({}/{})",
                        scene.score_data.accuracy,
                        scene.score_data.correct_notes,
                        scene.score_data.total_notes
                    ))
                    .font_size(24.0)
                    .font_family("Arial")
                    .color([0.9, 0.9, 0.9, 1.0])
                    .size(panel_w - 40.0, 30.0)
                    .text_justify(nuon::TextJustify::Center)
                    .build(ui);
            });

            current_y += 40.0;

            nuon::translate().x(20.0).y(current_y).build(ui, |ui| {
                nuon::quad()
                    .size(panel_w - 40.0, 2.0)
                    .color([77, 77, 77])
                    .border_radius([1.0; 4])
                    .build(ui);
            });

            current_y += 30.0;

            let stat_line_height = 35.0;

            nuon::translate().x(40.0).y(current_y).build(ui, |ui| {
                render_stat_line(
                    ui,
                    0.0,
                    0.0,
                    "On Time:",
                    &scene.score_data.on_time.to_string(),
                    [51, 204, 51],
                );
            });

            current_y += stat_line_height;

            nuon::translate().x(40.0).y(current_y).build(ui, |ui| {
                render_stat_line(
                    ui,
                    0.0,
                    0.0,
                    "Too Early:",
                    &scene.score_data.too_early.to_string(),
                    [255, 153, 0],
                );
            });

            current_y += stat_line_height;

            nuon::translate().x(40.0).y(current_y).build(ui, |ui| {
                render_stat_line(
                    ui,
                    0.0,
                    0.0,
                    "Too Late:",
                    &scene.score_data.too_late.to_string(),
                    [255, 77, 0],
                );
            });

            current_y += stat_line_height;

            nuon::translate().x(40.0).y(current_y).build(ui, |ui| {
                render_stat_line(
                    ui,
                    0.0,
                    0.0,
                    "Missed:",
                    &scene.score_data.missed_notes.to_string(),
                    [204, 51, 51],
                );
            });

            current_y += 50.0;

            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_gap = 20.0;

            let total_btn_w = btn_w * 2.0 + btn_gap;
            let button_x = 20.0 + (panel_w - 40.0 - total_btn_w) / 2.0;

            let song_clone = scene.song.clone();
            if render_button(
                ui,
                button_x,
                current_y,
                "Replay",
                btn_w,
                btn_h,
                [51, 153, 230],
                Some(crate::icons::repeat_icon()),
            ) {
                ctx.proxy.send_event(NeothesiaEvent::Play(song_clone)).ok();
            }

            let song_clone = scene.song.clone();
            if render_button(
                ui,
                button_x + btn_w + btn_gap,
                current_y,
                "Continue",
                btn_w,
                btn_h,
                [77, 77, 89],
                Some(crate::icons::right_arrow_icon()),
            ) {
                ctx.proxy
                    .send_event(NeothesiaEvent::MainMenu(Some(song_clone)))
                    .ok();
            }
        });

    scene.nuon = ui;
}

fn render_stat_line(ui: &mut nuon::Ui, x: f32, y: f32, label: &str, value: &str, color: [u8; 3]) {
    nuon::translate().x(x).y(y).build(ui, |ui| {
        nuon::label()
            .text(label)
            .font_size(20.0)
            .font_family("Arial")
            .color([0.7, 0.7, 0.7, 1.0])
            .size(200.0, 25.0)
            .text_justify(nuon::TextJustify::Left)
            .build(ui);

        nuon::translate().x(200.0).build(ui, |ui| {
            nuon::label()
                .text(value)
                .font_size(20.0)
                .font_family("Arial")
                .color(color)
                .size(100.0, 25.0)
                .text_justify(nuon::TextJustify::Left)
                .build(ui);
        });
    });
}

fn render_button(
    ui: &mut nuon::Ui,
    x: f32,
    y: f32,
    label: &str,
    width: f32,
    height: f32,
    color: [u8; 3],
    icon: Option<&str>,
) -> bool {
    let id = nuon::Id::hash(label);
    let event = nuon::click_area(id).size(width, height).x(x).y(y).build(ui);

    let bg_color = if event.is_hovered() || event.is_pressed() {
        let c = color.map(|x| x.saturating_add(30));
        [c[0], c[1], c[2]]
    } else {
        color
    };

    nuon::quad()
        .x(x)
        .y(y)
        .size(width, height)
        .color(bg_color)
        .border_radius([5.0; 4])
        .build(ui);

    if let Some(icon_str) = icon {
        // Button with icon + text
        let icon_size = 24.0;
        let icon_spacing = 10.0;
        let text_x = x + icon_size + icon_spacing;

        // Render icon on the left
        nuon::label()
            .icon(icon_str)
            .font_size(icon_size)
            .x(x + 10.0)
            .y(y)
            .color([1.0, 1.0, 1.0, 1.0])
            .size(icon_size, height)
            .text_justify(nuon::TextJustify::Center)
            .build(ui);

        // Render text on the right
        nuon::label()
            .text(label)
            .font_size(20.0)
            .font_family("Arial")
            .x(text_x)
            .y(y)
            .color([1.0, 1.0, 1.0, 1.0])
            .size(width - icon_size - icon_spacing - 10.0, height)
            .text_justify(nuon::TextJustify::Left)
            .build(ui);
    } else {
        // Button with text only
        nuon::label()
            .text(label)
            .font_size(20.0)
            .font_family("Arial")
            .x(x)
            .y(y)
            .color([1.0, 1.0, 1.0, 1.0])
            .size(width, height)
            .text_justify(nuon::TextJustify::Center)
            .build(ui);
    }

    event.is_clicked()
}

fn grade_color(grade: &str) -> [f32; 4] {
    match grade {
        "S" => [1.0, 0.9, 0.2, 1.0],
        "A" => [0.2, 0.8, 0.4, 1.0],
        "B" => [0.2, 0.6, 0.9, 1.0],
        "C" => [0.6, 0.4, 0.9, 1.0],
        "D" => [0.9, 0.6, 0.2, 1.0],
        "F" => [0.8, 0.2, 0.2, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}
