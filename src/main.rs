use macroquad::prelude::*;

const GRID_SIZE: usize = 4;

#[derive(Clone, Copy, Debug)]
enum Cube {
    Green,
    Red,
    Blue
}

type CubeColumn = [Option<Cube>; GRID_SIZE];
type CubeGrid = [[[Option<Cube>; GRID_SIZE]; GRID_SIZE]; GRID_SIZE];
#[macroquad::main("BasicShapes")]
async fn main() {
    let mut x = 0.0;

    let mut dragged_cube: Option<Cube> = None;
    let mut switch = false;
    let bounds = 8.0;

    let mut cubes : CubeGrid = CubeGrid::default();
    let world_up = vec3(0.0, 0.0, 1.0);

    let mut camera_target = vec3(0.0, 0.0, 1.0);
    let mut camera_position = camera_target + vec3(5.0, 5.0, 2.0);
    let mut idx = IVec3::ZERO;
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = false;
    set_cursor_grab(grabbed);
    show_mouse(false);

    loop {
        let delta = get_frame_time();

        clear_background(LIGHTGRAY);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;
        last_mouse_position = mouse_position;

        x += if switch { 0.04 } else { -0.04 };
        if x >= bounds || x <= -bounds {
            switch = !switch;
        }

        // Going 3d!
        egui_macroquad::ui(|egui_ctx| {

            egui::SidePanel::right("my_right_panel")
                .resizable(false)
                .show(egui_ctx, |ui| {
                ui.label("Trascina i colori su questa griglia!");
                // show the widget that is used to place the cubes
                let colored_square_size = ui.spacing().interact_size.y * egui::vec2(2.5, 2.5);
                let colored_square_spacing = ui.spacing().interact_size.y * egui::vec2(0.2, 0.2);
                let shrink_amount = ui.spacing().interact_size.y * egui::vec2(0.25, 0.25);
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
                                draw_column(&cubes[x][y], ui, rect, shrink_amount);
                            }
                            ui.end_row();
                        }
                    });

                if ui.input().pointer.any_released() {
                    dragged_cube = None;
                }
                ui.label("Colori disponibili");
                let square_size = ui.spacing().interact_size.y * egui::vec2(2.0, 2.0);
                ui.horizontal(|ui| {
                    if draw_drag_cube(ui, egui::Color32::RED, square_size) {
                        dragged_cube = Some(Cube::Red);
                    }
                    if draw_drag_cube(ui, egui::Color32::GREEN, square_size) {
                        dragged_cube = Some(Cube::Green);
                    }
                    if draw_drag_cube(ui, egui::Color32::BLUE, square_size) {
                        dragged_cube = Some(Cube::Blue);
                    }
                });
            });
            // BOT BAR: camera
            egui::TopBottomPanel::bottom("camera_panel").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let camera_button_size = ui.spacing().interact_size.y * egui::vec2(3.0, 3.0);
                    if draw_x_icon(ui, camera_button_size).clicked() {
                        dbg!("change camera to front!");
                    };
                    if draw_y_icon(ui, camera_button_size).clicked() {
                        println!("camera to side");
                    }
                    if draw_z_icon(ui, camera_button_size).clicked() {
                        println!("camera to top");
                    }
                    if draw_xyz_icon(ui, camera_button_size).clicked() {
                        println!("camera to isometric");
                    }
                });
            });
        });

        set_camera(&Camera3D {
            position: camera_position,
            up: world_up,
            projection: Projection::Orthographics,
            target: camera_target,
            fovy: 5.0,
            viewport: Some((0, 0, 600, 480)), // viewport is (x, y, w, h)
            aspect: Some(600_f32/480_f32),
            ..Default::default()
        });

        let selector_position = idx.as_f32();
        if is_key_pressed(KeyCode::G) {
            cubes[idx.x as usize][idx.y as usize][idx.z as usize] = Some(Cube::Green);
        }

        if is_key_pressed(KeyCode::R) {
            cubes[idx.x as usize][idx.y as usize][idx.z as usize] = Some(Cube::Red);
        }

        if is_key_pressed(KeyCode::B) {
            cubes[idx.x as usize][idx.y as usize][idx.z as usize] = Some(Cube::Blue);
        }

        if is_key_pressed(KeyCode::X) {
            cubes[idx.x as usize][idx.y as usize][idx.z as usize] = None;
        }

        if is_key_pressed(KeyCode::PageUp) {
            idx.z = clamp(idx.z + 1, 0, GRID_SIZE as i32 - 1);
        }
        if is_key_pressed(KeyCode::PageDown) {
            idx.z = clamp(idx.z - 1, 0, GRID_SIZE as i32 - 1);
        }
        if is_key_pressed(KeyCode::Up) {
            idx.x = clamp(idx.x - 1, 0, GRID_SIZE as i32 - 1);
        }
        if is_key_pressed(KeyCode::Down) {
            idx.x = clamp(idx.x + 1, 0, GRID_SIZE as i32 - 1);
        }
        if is_key_pressed(KeyCode::Left) {
            idx.y = clamp(idx.y - 1, 0, GRID_SIZE as i32 - 1);
        }
        if is_key_pressed(KeyCode::Right) {
            idx.y = clamp(idx.y + 1, 0, GRID_SIZE as i32 - 1);
        }
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                for z in 0..GRID_SIZE {
                    let edge_color = if idx.x == x as i32 && idx.y == y as i32 && idx.z == z as i32 {
                        GOLD
                    } else {
                        BLACK
                    };
                    if let Some(cube) = cubes[x][y][z] {
                        match cube {
                        Cube::Red => {
                            draw_cube(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), None, RED);
                            draw_cube_wires(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), edge_color);
                        }
                        Cube::Green => {
                            draw_cube(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), None, GREEN);
                            draw_cube_wires(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), edge_color);
                        }
                        Cube::Blue => {
                            draw_cube(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), None, BLUE);
                            draw_cube_wires(vec3(x as f32, y as f32, z as f32), vec3(1.0, 1.0, 1.0), edge_color);
                        }
                        }
                    }
                }
            }
        }
        draw_cube_wires(idx.as_f32(), vec3(1.0, 1.0, 1.0), DARKPURPLE);

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

fn draw_column(col: &CubeColumn, ui: &mut egui::Ui, rect: egui::Rect, shrink: egui::Vec2) {
    if ui.is_rect_visible(rect) {
        // first, paint "the background"
        ui.painter()
            .rect(rect, 0.0, egui::Color32::DARK_GRAY, egui::Stroke::default());
        for z in 0..GRID_SIZE {
            if let Some(cube) = col[z] {
                let cube_color = match cube {
                    Cube::Green => egui::Color32::GREEN,
                    Cube::Red => egui::Color32::RED,
                    Cube::Blue => egui::Color32::BLUE,
                };
                    ui.painter()
                        .rect(rect.shrink2(shrink * z as f32), 0.0, cube_color, egui::Stroke::default());
                }
        }
    }
}

fn draw_drag_cube(ui: &mut egui::Ui, color: egui::Color32, size: egui::Vec2) -> bool {
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::drag());
    ui.painter().rect(rect, 2.0, color, egui::Stroke::default());
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
