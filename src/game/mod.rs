use crate::TShape;

use raylib::prelude::*;

use std::time::{Instant, Duration};
use std::ffi::CStr;

const HEIGHT: i32 = 1000;
const WIDTH: i32 = 800;

pub enum Direction {
    Right,
    Left,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Score {
    pub lines: u32,
}

pub struct Game {
    lines: Vec<Vec<bool>>,

    delta: Instant,
    shape: Vec<Position>,
    shapes: TShape,

    pub score: Score,
    debug: bool,
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub smooth: bool,
    pub mode3d: bool,
}

pub struct Assets {
    theme: Sound,
    thump: Sound,
    metal_crate: Model,
    table: Model,
    shader: Shader,
    tbox: Texture2D,
}

pub struct Renderer<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    audio: &'a mut RaylibAudio,
    framebuffer: RenderTexture2D,
    camera: Camera3D,
    pub game: Game,
    assets: Assets,
    settings: Settings,
}

impl<'a> Renderer<'a> {
    pub fn new(rl: &'a mut RaylibHandle, thread: &'a RaylibThread, audio: &'a mut RaylibAudio, settings: Settings) -> Result<Renderer<'a>, Box<dyn std::error::Error>> {
        rl.set_window_title(thread, "Playing Tetris");

        let mut tbox = Image::load_image("assets/textures/tbox.png")?;
        tbox.resize(90, 90);

        let mut assets = Assets {
            theme: Sound::load_sound("assets/sounds/theme.mp3")?,
            thump: Sound::load_sound("assets/sounds/thump.mp3")?,
            metal_crate: rl.load_model(&thread, "assets/box.obj")?,
            table: rl.load_model(&thread, "assets/table.obj")?,
            shader: rl.load_shader(&thread, None, Some("assets/shaders/shader.fs"))?,
            tbox: rl.load_texture_from_image(&thread, &tbox)?,
        };

        Self::apply_texture(
            rl,
            thread,
            &mut assets.metal_crate,
            "assets/textures/box_Albedo.png",
            "assets/textures/box_Metallic.png",
            "assets/textures/box_Normal.png",
            "assets/textures/box_Roughness.png",
        )?;

        Self::apply_texture(
            rl,
            thread,
            &mut assets.table,
            "assets/textures/table/Table_Base_Color.png",
            "assets/textures/table/Table_Metallic.png",
            "assets/textures/table/Table_Normal_OpenGL.png",
            "assets/textures/table/Table_Roughness.png",
        )?;

        let framebuffer = rl.load_render_texture(&thread, WIDTH as u32, HEIGHT as u32)?;
        let shapes = TShape::load("assets/shapes.tshape")?;

        Ok(Renderer {
            rl,
            thread,
            audio,
            framebuffer,
            camera: Camera3D::perspective(
                Vector3::new(70.0, 35.0, 0.0),
                Vector3::new(0.0, 30.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                60.0,
            ),
            game: Game {
                lines: vec![vec![false; 5]; 10],

                delta: Instant::now(),
                shape: shapes.rand_shape(),
                shapes,

                score: Score {
                    lines: 0,
                },
                debug: false,
            },
            assets,
            settings,
        })
    }

    fn load_texture(rl: &mut RaylibHandle, thread: &RaylibThread, texture: &str) -> Result<raylib::ffi::Texture, Box<dyn std::error::Error>> {
        let texture = unsafe {
            let mut t = rl.load_texture(&thread, texture)?;
            t.gen_texture_mipmaps();
            t.unwrap()
        };

        Ok(texture)
    }

    fn apply_texture(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        model: &mut Model,
        albedo: &str,
        metallic: &str,
        normal: &str,
        rough: &str
    ) -> Result<(), Box<dyn std::error::Error>> {

        let albedo = Self::load_texture(rl, thread, albedo)?;
        let metallic = Self::load_texture(rl, thread, metallic)?;
        let normal = Self::load_texture(rl, thread, normal)?;
        let rough = Self::load_texture(rl, thread, rough)?;

        let material = &mut model.materials_mut()[0];
        let maps = material.maps_mut();

        maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = albedo;
        maps[MaterialMapIndex::MATERIAL_MAP_METALNESS as usize].texture = metallic;
        maps[MaterialMapIndex::MATERIAL_MAP_NORMAL as usize].texture = normal;
        maps[MaterialMapIndex::MATERIAL_MAP_ROUGHNESS as usize].texture = rough;

        Ok(())
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let fps = self.rl.get_fps();
        let mut drawer = self.rl.begin_drawing(&self.thread);

        drawer.clear_background(Color::BLACK);

        if self.settings.mode3d {
            let mut texture_drawer = drawer.begin_texture_mode(&self.thread, &mut self.framebuffer);
            texture_drawer.clear_background(Color::from_hex("0B0D13")?);

            // render 3d
            {
                let mut render3d = texture_drawer.begin_mode3D(&self.camera);

                // table
                render3d.draw_model_ex(
                    &self.assets.table,
                    Vector3::new(
                        -10.0,
                        0.0,
                        0.0,
                    ),
                   Vector3::new(
                       90.0,
                       0.0,
                       0.0,
                    ),
                    -90.0,
                    Vector3::new(
                        0.15,
                        0.15,
                        0.15,
                    ),
                    Color::WHITE,
                );

                for (y, line) in self.game.lines.iter().enumerate() {
                    for (x, block) in line.iter().enumerate() {
                        let mut position = Vector3::new(
                            0.0,
                            (y as f32 * 5.5) + 5.0,
                            (x as f32 * 5.5) - (22.0 / 2.0)
                        );

                        if *block {
                            render3d.draw_model(&self.assets.metal_crate, position, 16.0, Color::WHITE);
                        } else if self.game.shape.contains(&Position { x, y }) {
                            if self.settings.smooth {
                                position.y -= 11.0 * self.game.delta.elapsed().as_secs_f32();
                            }

                            render3d.draw_model(&self.assets.metal_crate, position, 16.0, Color::WHITE);
                        }

                        if self.game.debug {
                            render3d.draw_cube_wires(
                                position,
                                5.0,
                                5.0,
                                5.0,
                                Color::RED,
                            );
                        }
                    }
                }
            }
        } else {
            // render 2d
            for (y, line) in self.game.lines.iter().enumerate() {
                for (x, block) in line.iter().enumerate() {
                    let mut position = Vector2::new(
                        (x as f32 * -90.0) + 530.0,
                        (y as f32 * -90.0) + 700.0,
                    );

                    if *block {
                        drawer.draw_texture(&self.assets.tbox, position.x as i32, position.y as i32, Color::WHITE);
                    } else if self.game.shape.contains(&Position { x, y }) {
                        if self.settings.smooth {
                            position.y += 180.0 * self.game.delta.elapsed().as_secs_f32();
                        }

                        drawer.draw_texture(&self.assets.tbox, position.x as i32, position.y as i32, Color::WHITE);
                    }
                }
            }
        }

        if self.settings.mode3d {
            // shaders
            {
                let mut shader = drawer.begin_shader_mode(&self.assets.shader);

                shader.draw_texture_rec(
                    self.framebuffer.texture(),
                    Rectangle::new(
                        0.0,
                        0.0,
                        self.framebuffer.texture.width as f32,
                        -self.framebuffer.texture.height as f32,
                    ),
                    Vector2::new(0.0, 0.0),
                    Color::WHITE
                );
            }
        }

        // Score
        {
            let score = format!("{}", self.game.score.lines);

            drawer.draw_text(
                &score,
                (WIDTH / 2) - (text::measure_text(&score, 30) / 2),
                20,
                30,
                Color::WHITE,
            );
        }

        // Debug menu
        if self.game.debug {
            let labels: Vec<String> = vec![
                format!("FPS: {}\0", fps),
            ];

            drawer.gui_set_style(GuiControl::DEFAULT, 16, 16);

            drawer.gui_group_box(
                Rectangle::new(10.0, 10.0, 150.0, (labels.len() as f32 * 15.0) + 5.0),
                Some(CStr::from_bytes_with_nul(b"Debug Menu\0")?)
            );

            for (index, label) in labels.iter().enumerate() {
                drawer.gui_set_style(GuiControl::DEFAULT, 16, 15);

                drawer.gui_label(
                    Rectangle::new(10.0, (index as f32 * 15.0) + 15.0, 100.0, 15.0),
                    Some(CStr::from_bytes_with_nul(label.as_bytes())?)
                );
            }
        }

        Ok(())
    }

    fn get_corner_position(&mut self, direction: Direction) -> Position {
        let mut corner = self.game.shape[0];

        for position in &self.game.shape {
            match direction {
                Direction::Right => {
                    if position.x < corner.x {
                        corner = *position;
                    }
                },
                Direction::Left => {
                    if position.x > corner.x {
                        corner = *position;
                    }
                },
            }
        }

        corner
    }

    fn shape_is_next_to_block(&mut self, direction: Direction) -> bool {
        for position in &self.game.shape {
            match direction {
                Direction::Right => {
                    if position.x > 0 {
                        if self.game.lines[position.y][position.x - 1] {
                            return true;
                        }
                    }
                },
                Direction::Left => {
                    if position.x < 4 {
                        if self.game.lines[position.y][position.x + 1] {
                            return true;
                        }
                    }
                },
            }
        }

        false
    }

    fn get_top_position(&mut self) -> Position {
        let mut top = self.game.shape[0];

        self.game.shape
            .iter()
            .for_each(|pos| if top.y > pos.y { top = *pos });

        top
    }

    fn rotate_shape(&mut self) {
        let top = self.get_top_position();
        let left_corner = self.get_corner_position(Direction::Right);

        let mut translated: [[bool; 2]; 2] = [[false; 2]; 2];
        {
            for position in &self.game.shape {
                translated[position.y - top.y][position.x - left_corner.x] = true;
            }
        }

        translated = [
            [translated[1][0], translated[0][0]],
            [translated[1][1], translated[0][1]]
        ];

        self.game.shape = Vec::new();
        {
            for (y, row) in translated.iter().enumerate() {
                for (x, position) in row.iter().enumerate() {
                    if *position {
                        self.game.shape.push(Position { x: x + left_corner.x, y: y + top.y});
                    }
                }
            }
        }
    }

    fn handle_input(&mut self) {
        if let Some(key) = self.rl.get_key_pressed() {
            match key {
                KeyboardKey::KEY_D => {
                    self.game.debug = !self.game.debug;
                },
                KeyboardKey::KEY_RIGHT => {
                    if self.get_corner_position(Direction::Right).x != 0 && !self.shape_is_next_to_block(Direction::Right) {
                        for position in &mut self.game.shape {
                            position.x -= 1;
                        }
                    }
                },
                KeyboardKey::KEY_LEFT => {
                    if self.get_corner_position(Direction::Left).x != 4 && !self.shape_is_next_to_block(Direction::Left) {
                        for position in &mut self.game.shape {
                            position.x += 1;
                        }
                    }
                },
                KeyboardKey::KEY_UP => {
                    self.rotate_shape();
                },
                KeyboardKey::KEY_DOWN => {
                    if !self.is_collision() {
                        self.move_down();
                    }
                },
                _ => {},
            }
        }
    }

    fn lock_size(&mut self) {
        if self.rl.is_window_resized() {
            self.rl.set_window_size(WIDTH, HEIGHT);
        }
    }

    fn is_collision(&mut self) -> bool {
        for position in &self.game.shape {
            if position.y == 0 {
                return true;
            }

            if self.game.lines[position.y - 1][position.x] || self.game.lines[position.y][position.x] {
                return true;
            }
        }

        false
    }

    fn update_position(&mut self) {
        if self.game.delta.elapsed() >= Duration::from_secs_f64(0.5) {
            self.move_down();

            if self.is_collision() {
                self.audio.play_sound(&self.assets.thump);

                for position in &self.game.shape {
                    self.game.lines[position.y][position.x] = true;
                }

                self.game.shape = self.game.shapes.rand_shape();
            }

            self.game.delta = Instant::now();
        }
    }

    fn move_down(&mut self) {
        for index in 0..self.game.shape.len() {
            if self.game.shape[index].y > 0 {
                self.game.shape[index].y -= 1;
            }
        }
    }

    fn is_valid_point(&self, line: &Vec<bool>) -> bool {
        for block in line {
            if !block {
                return false;
            }
        }

        true
    }

    fn update_lines(&mut self) {
        let mut index = 0;

        for line in self.game.lines.clone() {
            if self.is_valid_point(&line) {
                self.game.lines.remove(index);
                self.game.lines.push(vec![false; 5]);

                self.game.score.lines += 1;
            }

            index += 1;
        }
    }

    fn play_theme(&mut self) {
        if !self.audio.is_sound_playing(&self.assets.theme) {
            self.audio.play_sound(&self.assets.theme);
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.game.delta = Instant::now();

        while !self.rl.window_should_close() {
            self.draw()?;
            self.lock_size();
            self.play_theme();
            self.handle_input();
            self.update_position();
            self.update_lines();
        }

        Ok(())
    }
}


