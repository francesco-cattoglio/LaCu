#[macro_use]
extern crate enum_map;

use macroquad::prelude::*;

mod cubes;
use cubes::{Cube, create_default_cubemap, CubeMap, CubeInfo};

const GRID_SIZE: usize = 4;

enum CameraView {
    Front,
    Side,
    Top,
    Isometric
}

type CubeColumn = [Option<Cube>; GRID_SIZE];
type CubeGrid = [[[Option<Cube>; GRID_SIZE]; GRID_SIZE]; GRID_SIZE];

fn window_conf() -> Conf {
    Conf {
        window_title: "ColorCubes".to_owned(),
        sample_count: 8,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut camera_view = CameraView::Isometric;
    let mut camera = create_isometric_camera();

    let cubemap = create_default_cubemap();

    let mut show_paving: bool = true;
    let mut hide_builder: bool = false;
    let mut dragged_cube: Option<Cube> = None;

    let mut cubes : CubeGrid = CubeGrid::default();
    let mut saved_mouse_position: Vec2 = mouse_position().into();

    show_mouse(false);

    loop {
        let delta = get_frame_time();

        let mut additional_zoom = 0.0;
        clear_background(WHITE);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Whole egui UI
        let vertical_display = screen_height() > screen_width();
        egui_macroquad::ui(|egui_ctx| {
            let cube_builder_lambda = |ui: &mut egui::Ui| {
                let layout = if vertical_display {
                    egui::Layout::left_to_right()
                } else {
                    egui::Layout::top_down(egui::Align::Min)
                };
                ui.with_layout(layout, |ui| {
                    ui.vertical(|ui| {
                        // show the widget that is used to place the cubes
                        let colored_square_size = ui.spacing().interact_size.y * egui::vec2(2.5, 2.5);
                        let colored_square_spacing = ui.spacing().interact_size.y * egui::vec2(0.2, 0.2);
                        let shrink_amount = ui.spacing().interact_size.y * egui::vec2(0.25, 0.25);
                        ui.checkbox(&mut hide_builder, "Nascondi lo schema");
                        if hide_builder {
                            let total_size = colored_square_size * GRID_SIZE as f32 + colored_square_spacing * (GRID_SIZE - 1) as f32;
                            let (rect, _response) = ui.allocate_exact_size(total_size, egui::Sense::click());
                            let label = egui::widgets::Label::new(egui::RichText::new("?").color(egui::Color32::KHAKI).size(72.0));
                            ui.put(rect, label);
                        } else {
                            egui::Grid::new("cube_builder")
                                .spacing(colored_square_spacing)
                                .show(ui, |ui| {
                                    for x in 0..GRID_SIZE {
                                        for y in 0..GRID_SIZE {
                                            let (rect, response) = ui.allocate_exact_size(colored_square_size, egui::Sense::click());
                                            if response.double_clicked() {
                                                pop_top(&mut cubes, x, y);
                                            }
                                            if let Some(dropped_cube) = dragged_cube {
                                                if ui.input().pointer.any_released() && response.hovered() {
                                                    push_top(&mut cubes, x, y, dropped_cube);
                                                }
                                            }
                                            draw_column(&cubes[x][y], &cubemap, ui, rect, shrink_amount);
                                        }
                                        ui.end_row();
                                    }
                                });

                        }
                        if ui.input().pointer.any_released() {
                            dragged_cube = None;
                        }
                    });
                    ui.vertical(|ui| {
                        use itertools::Itertools;
                        ui.label("Cubi disponibili");
                        let square_size = ui.spacing().interact_size.y * egui::vec2(2.0, 2.0);
                        for line in &cubemap.iter().chunks(5) {
                            ui.horizontal(|ui| {
                                for entry in line {
                                    let cube: Cube = entry.0;
                                    let info: &CubeInfo = entry.1;
                                    if draw_drag_cube(ui, info.egui_shape.clone(), square_size) {
                                        dragged_cube = Some(cube);
                                    }
                                }
                            });
                            ui.add_space(2.0);
                        }
                        ui.checkbox(&mut show_paving, "Mostra la scacchiera");
                    });
                });
            };
            if vertical_display {
                egui::TopBottomPanel::top("cube_builder_panel")
                    .resizable(false)
                    .show(egui_ctx, cube_builder_lambda);
            } else {
                egui::SidePanel::right("cube_builder_panel")
                    .resizable(false)
                    .show(egui_ctx, cube_builder_lambda);
            }
            // BOT BAR: camera
            egui::TopBottomPanel::bottom("camera_panel").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let camera_button_size = ui.spacing().interact_size.y * egui::vec2(3.0, 3.0);
                    if draw_x_icon(ui, camera_button_size).clicked() {
                        camera = create_front_camera();
                        camera_view = CameraView::Front;
                    };
                    if draw_y_icon(ui, camera_button_size).clicked() {
                        camera = create_side_camera();
                        camera_view = CameraView::Side;
                    }
                    if draw_z_icon(ui, camera_button_size).clicked() {
                        camera = create_top_camera();
                        camera_view = CameraView::Top;
                    }
                    if draw_xyz_icon(ui, camera_button_size).clicked() {
                        camera = create_isometric_camera();
                        camera_view = CameraView::Isometric;
                    }

                    let rich_text = egui::RichText::new("🔎")
                        .size(32.0)
                        .color(egui::Color32::WHITE);
                    ui.label(rich_text);
                    let zoom_slider = egui::Slider::new(&mut additional_zoom, -2.0..=2.0)
                        .show_value(false);
                    ui.add_sized(ui.available_size(), zoom_slider);
                });
            });
        });

        let mut egui_mouse_requested = false;
        let mut viewport_area = (0, 0, 0, 0);

        egui_macroquad::cfg(|egui_ctx: &egui::Context| {
            egui_mouse_requested = egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();
            let free_area = egui_ctx.available_rect();
            viewport_area = (0, (screen_height() - free_area.max.y) as i32, free_area.size().x as i32, free_area.size().y as i32);
        });


        // camera control
        if !egui_mouse_requested {
            // TODO: check if this is needed for real, or everything still works even if we do not
            // separate the first click-to-save position
            if is_mouse_button_pressed(MouseButton::Left) {
                saved_mouse_position = mouse_position().into();
            } else if is_mouse_button_down(MouseButton::Left) {
                let mouse_position: Vec2 = mouse_position().into();
                let mouse_delta = mouse_position - saved_mouse_position;
                saved_mouse_position = mouse_position;
                // move camera, something different can happen depending on the active view
                let scale = 0.02;
                match camera_view {
                    CameraView::Top => {
                        camera.target.x -= scale * mouse_delta.y;
                        camera.position.x -= scale * mouse_delta.y;
                        camera.target.y -= scale * mouse_delta.x;
                        camera.position.y -= scale * mouse_delta.x;
                    }
                    CameraView::Front => {
                        camera.target.z += scale * mouse_delta.y;
                        camera.position.z += scale * mouse_delta.y;
                        camera.target.y -= scale * mouse_delta.x;
                        camera.position.y -= scale * mouse_delta.x;
                    }
                    CameraView::Side => {
                        camera.target.z += scale * mouse_delta.y;
                        camera.position.z += scale * mouse_delta.y;
                        camera.target.x -= scale * mouse_delta.x;
                        camera.position.x -= scale * mouse_delta.x;
                    }
                    CameraView::Isometric => {
                        let camera_dir = camera.target - camera.position;
                        let camera_left = camera.up.cross(camera_dir).normalize();
                        camera.target += scale * (mouse_delta.x*camera_left + mouse_delta.y * camera.up);
                        camera.position += scale * (mouse_delta.x*camera_left + mouse_delta.y * camera.up);
                    }
                }
            }
        }

        camera.viewport = Some(viewport_area);
        camera.aspect = Some(viewport_area.2 as f32 / viewport_area.3 as f32);
        camera.fovy -= additional_zoom * delta;
        set_camera(&camera);

        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if show_paving {
                    // add some cubes to be used as a "pavement"
                    let paving_color = if (x+y) % 2 == 0 {
                        GRAY
                    } else {
                        LIGHTGRAY
                    };
                    draw_cube(vec3(x as f32, y as f32, -0.6), vec3(1.0, 1.0, 0.2), None, paving_color);
                }
                for z in 0..GRID_SIZE {
                    if let Some(cube) = cubes[x][y][z] {
                        let model_matrix = Mat4::from_translation(vec3(x as f32, y as f32, z as f32));
                        let gl = unsafe { get_internal_gl().quad_gl };
                        gl.push_model_matrix(model_matrix);
                        draw_mesh(&cubemap[cube].mesh);
                        gl.pop_model_matrix();
                    }
                }
            }
        }

        // Back to screen space, render some text
        egui_macroquad::draw();
        next_frame().await
    }
}

fn push_top(grid: &mut CubeGrid, x: usize, y: usize, cube: Cube) {
    for z in 0..GRID_SIZE {
        if grid[x][y][z].is_none() {
            grid[x][y][z] = Some(cube);
            return;
        }
    }
}

fn pop_top(grid: &mut CubeGrid, x: usize, y: usize) -> Option<Cube> {
    for z in (0..GRID_SIZE).rev() {
        if let Some(cube) = grid[x][y][z] {
            grid[x as usize][y as usize][z] = None;
            return Some(cube);
        }
    }

    None
}

fn draw_column(col: &CubeColumn, cubemap: &CubeMap, ui: &mut egui::Ui, rect: egui::Rect, shrink: egui::Vec2) {
    if ui.is_rect_visible(rect) {
        // first, paint "the background"
        ui.painter().rect(rect, 0.0, egui::Color32::DARK_GRAY, egui::Stroke::default());
        // then, for each element, draw the cube, but shrink it every time we move "up"
        for z in 0..GRID_SIZE {
            if let Some(cube) = col[z] {
                let mut cube_shape = cubemap[cube].egui_shape.clone();
                let shrinked = rect.shrink2(shrink * z as f32);
                translate_scale_shape(&mut cube_shape, shrinked.min.to_vec2(), shrinked.size());
                ui.painter().add(cube_shape);
            }
        }
    }
}

fn draw_drag_cube(ui: &mut egui::Ui, mut shape: egui::Shape, size: egui::Vec2) -> bool {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::drag());
    translate_scale_shape(&mut shape, rect.min.to_vec2(), rect.size());
    ui.painter().add(shape);
    response.drag_started()
}

fn draw_x_icon(ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::LIGHT_GRAY);
    let points = compute_diamond_corners(rect);
    let face = vec![points[5], rect.center(), points[3], points[4]];
    let p_stroke =  egui::Stroke::new(4.0, egui::Color32::BLACK);
    let f_stroke =  egui::Stroke::new(2.0, egui::Color32::BLACK);
    ui.painter().line_segment([rect.center(), points[1]], f_stroke);
    let sil = egui::Shape::convex_polygon(points, egui::Color32::TRANSPARENT, p_stroke);
    ui.painter().add(sil);
    let fill = egui::Shape::convex_polygon(face, egui::Color32::DARK_RED, f_stroke);
    ui.painter().add(fill);

    response
}

fn draw_y_icon(ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::LIGHT_GRAY);
    let points = compute_diamond_corners(rect);
    let face = vec![points[1], points[2], points[3], rect.center()];
    let p_stroke =  egui::Stroke::new(4.0, egui::Color32::BLACK);
    let f_stroke =  egui::Stroke::new(2.0, egui::Color32::BLACK);
    ui.painter().line_segment([rect.center(), points[5]], f_stroke);
    let sil = egui::Shape::convex_polygon(points, egui::Color32::TRANSPARENT, p_stroke);
    ui.painter().add(sil);
    let fill = egui::Shape::convex_polygon(face, egui::Color32::DARK_GREEN, f_stroke);
    ui.painter().add(fill);

    response
}

fn draw_z_icon(ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::LIGHT_GRAY);
    let points = compute_diamond_corners(rect);
    let face = vec![points[0], points[1], rect.center(), points[5]];
    let p_stroke =  egui::Stroke::new(4.0, egui::Color32::BLACK);
    let f_stroke =  egui::Stroke::new(2.0, egui::Color32::BLACK);
    ui.painter().line_segment([rect.center(), points[3]], f_stroke);
    let sil = egui::Shape::convex_polygon(points, egui::Color32::TRANSPARENT, p_stroke);
    ui.painter().add(sil);
    let fill = egui::Shape::convex_polygon(face, egui::Color32::DARK_BLUE, f_stroke);
    ui.painter().add(fill);

    response
}

fn draw_xyz_icon(ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::LIGHT_GRAY);
    let points = compute_diamond_corners(rect);
    let face_x = vec![points[5], rect.center(), points[3], points[4]];
    let face_y = vec![points[1], points[2], points[3], rect.center()];
    let face_z = vec![points[0], points[1], rect.center(), points[5]];
    let p_stroke =  egui::Stroke::new(4.0, egui::Color32::BLACK);
    let f_stroke =  egui::Stroke::new(2.0, egui::Color32::BLACK);
    let sil = egui::Shape::convex_polygon(points, egui::Color32::TRANSPARENT, p_stroke);
    let fill_x = egui::Shape::convex_polygon(face_x, egui::Color32::DARK_RED, f_stroke);
    let fill_y = egui::Shape::convex_polygon(face_y, egui::Color32::DARK_GREEN, f_stroke);
    let fill_z = egui::Shape::convex_polygon(face_z, egui::Color32::DARK_BLUE, f_stroke);
    ui.painter().extend(vec![sil, fill_x, fill_y, fill_z]);

    response
}

fn compute_diamond_corners(mut rect: egui::Rect) -> Vec<egui::Pos2> {
    rect = rect.shrink(4.0);
    let tl = rect.min.to_vec2();
    let br = rect.max.to_vec2();
    let tr = egui::vec2(br.x, tl.y);
    let bl = egui::vec2(tl.x, br.y);
    let p0 = tl * 0.5 + tr * 0.5;
    let p1 = tr * 0.75 + br * 0.25;
    let p2 = tr * 0.25 + br * 0.75;
    let p3 = bl * 0.5 + br * 0.5;
    let p4 = tl * 0.25 + bl * 0.75;
    let p5 = tl * 0.75 + bl * 0.25;

    vec![
        p0.to_pos2(),
        p1.to_pos2(),
        p2.to_pos2(),
        p3.to_pos2(),
        p4.to_pos2(),
        p5.to_pos2(),
    ]
}

fn create_top_camera() -> Camera3D {
    let grid_mid = 0.5 * (GRID_SIZE as f32 - 1.0);
    Camera3D {
        position: vec3(grid_mid, grid_mid, 10.0),
        up: vec3(-1.0, 0.0, 0.0),
        projection: Projection::Orthographics,
        target: vec3(grid_mid, grid_mid, 0.0),
        fovy: 5.0,
        ..Default::default()
    }
}

fn create_front_camera() -> Camera3D {
    let grid_mid = 0.5 * (GRID_SIZE as f32 - 1.0);
    Camera3D {
        position: vec3(10.0, grid_mid, grid_mid),
        up: vec3(0.0, 0.0, 1.0),
        projection: Projection::Orthographics,
        target: vec3(0.0, grid_mid, grid_mid),
        fovy: 5.0,
        ..Default::default()
    }
}

fn create_side_camera() -> Camera3D {
    let grid_mid = 0.5 * (GRID_SIZE as f32 - 1.0);
    Camera3D {
        position: vec3(grid_mid, 10.0, grid_mid),
        up: vec3(0.0, 0.0, 1.0),
        projection: Projection::Orthographics,
        target: vec3(grid_mid, 0.0, grid_mid),
        fovy: 5.0,
        ..Default::default()
    }
}

fn create_isometric_camera() -> Camera3D {
    let grid_mid = 0.5 * (GRID_SIZE as f32 - 1.0);
    Camera3D {
        position: vec3(grid_mid + 10.0, grid_mid + 7.5, grid_mid + 5.0),
        up: vec3(0.0, 0.0, 1.0),
        projection: Projection::Orthographics,
        target: vec3(grid_mid, grid_mid, grid_mid),
        fovy: 6.0,
        ..Default::default()
    }
}

fn translate_scale_shape(shape: &mut egui::Shape, translation: egui::Vec2, size: egui::Vec2) {
    match shape {
        egui::Shape::Rect(ref mut rect_shape) => {
            rect_shape.rect.max = size.to_pos2();
        }
        egui::Shape::Vec(ref mut vec_shape) => {
            for elem in vec_shape.iter_mut() {
                translate_scale_shape(elem, egui::vec2(0.0, 0.0), size);
            }
        }
        egui::Shape::Path(ref mut path_shape) => {
            for point in path_shape.points.iter_mut() {
                *point = (point.to_vec2() * size).to_pos2();
            }
        }
        _ => unimplemented!()
    }
    shape.translate(translation);
}
