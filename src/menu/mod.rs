use raylib::prelude::*;

use crate::{Renderer, game::Settings};

const HEIGHT: i32 = 1000;
const WIDTH: i32 = 800;

pub struct Assets {
    font: Font,
}

#[derive(PartialEq)]
pub enum Label {
    Button(&'static str),
    Toggle {
        label: &'static str,
        state: bool,
    },
}

impl Label {
    pub fn get_label(&self) -> &'static str {
        match self {
            Label::Button(label) => &label,
            Label::Toggle { label, .. } => &label,
        }
    }
}

pub struct Menu {
    rl: RaylibHandle,
    thread: RaylibThread,
    audio: RaylibAudio,
    assets: Assets,
    settings: Settings,
    selected: usize,
    labels: Vec<Label>,
    title: &'static str,
    should_close: bool,
}

impl Menu {
    pub fn new() -> Result<Menu, Box<dyn std::error::Error>> {
        let (mut rl, thread) = raylib::init()
            .title("Tetris")
            .size(WIDTH, HEIGHT)
            .build();

        let audio = RaylibAudio::init_audio_device();

        let assets = Assets {
            font: rl.load_font_ex(&thread, "assets/ui/InriaSerif-Regular.ttf", 60, FontLoadEx::Default(256))?,
        };

        Ok(Menu {
            rl,
            thread,
            audio,
            assets,
            settings: Settings {
                smooth: true,
            },
            selected: 0,
            labels: vec![Label::Button("Play"), Label::Button("Settings"), Label::Button("Exit")],
            title: "Tetris",
            should_close: false,
        })
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut drawer = self.rl.begin_drawing(&self.thread);

        drawer.clear_background(Color::from_hex("0B0D13")?);

        let fg = Color::from_hex("FFFFFF")?;
        let bg = Color::from_hex("0F1923")?;

        let size = text::measure_text_ex(&self.assets.font, self.title, 40.0, 42.0);

        drawer.draw_text_ex(
            &self.assets.font,
            self.title,
            Vector2::new(
                ((WIDTH / 2) - 150) as f32,
                ((HEIGHT / 2) as f32 - size.y - 100.0) as f32,
            ),
            40.0,
            42.0 - ((size.x - 300.0) / (self.title.len() - 1) as f32),
            fg
        );

        for (index, label) in self.labels.iter().enumerate() {
            drawer.draw_rectangle_rounded(
                Rectangle::new(
                    ((WIDTH / 2) - 150) as f32,
                    ((HEIGHT / 2) + (90 * index as i32)) as f32,
                    300.0,
                    60.0,
                ),
                0.3,
                200,
                bg
            );

            if &self.labels[self.selected] == label {
                drawer.draw_rectangle_rounded_lines(
                    Rectangle::new(
                        ((WIDTH / 2) - 150) as f32,
                        ((HEIGHT / 2) + (90 * index as i32)) as f32,
                        300.0,
                        60.0,
                    ),
                    0.3,
                    200,
                    1,
                    fg,
                );
            }

            let text = match label {
                Label::Button(label) => {
                    label
                },
                Label::Toggle { label, state } => {
                    if *state {
                        drawer.draw_circle(
                            (WIDTH / 2) + 130,
                            (HEIGHT / 2) + (90 * index as i32) + 30,
                            10.0,
                            fg,
                        );
                    } else {
                        drawer.draw_circle_lines(
                            (WIDTH / 2) + 130,
                            (HEIGHT / 2) + (90 * index as i32) + 30,
                            10.0,
                            fg,
                        );
                    }

                    label
                },
            };

            drawer.draw_text_ex(
                &self.assets.font,
                &text,
                Vector2::new(
                    ((WIDTH / 2) as f32 - (text::measure_text_ex(&self.assets.font, &text, 40.0, 2.0).x / 2.0)) as f32,
                    ((HEIGHT / 2) + (90 * index as i32) + 10) as f32,
                ),
                40.0,
                2.0,
                fg
            );
        }

        Ok(())
    }

    fn draw_loading(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut drawer = self.rl.begin_drawing(&self.thread);

        drawer.clear_background(Color::from_hex("0B0D13")?);

        let text = "Loading Assets";
        let size = text::measure_text_ex(&self.assets.font, text, 40.0, 2.0);

        drawer.draw_text_ex(
            &self.assets.font,
            &text,
            Vector2::new(
                (WIDTH / 2) as f32 - (size.x / 2.0),
                (HEIGHT / 2) as f32 - (size.y / 2.0),
            ),
            40.0,
            2.0,
            Color::from_hex("FFFFFF")?
        );

        Ok(())
    }

    fn play_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_loading()?;

        let mut renderer = Renderer::new(&mut self.rl, &self.thread, &mut self.audio, self.settings)?;

        renderer.run()?;

        Ok(())
    }

    fn enter_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let label = self.labels[self.selected].get_label();

        if label == "Play" {
            self.play_game()?;
        } else if label == "Smooth" {
            self.settings.smooth = !self.settings.smooth;
        } else if label == "Settings" {
            self.selected = 0;
            self.title = "Settings";
        } else if label == "Back" {
            self.selected = 0;
            self.title = "Tetris";
        } else if label == "Exit" {
            self.should_close = true;
        }

        Ok(())
    }

    fn update_menu(&mut self) {
        if self.title == "Settings" {
            self.labels = vec![Label::Toggle { label: "Smooth", state: self.settings.smooth }, Label::Button("Back")];
        } else if self.title == "Tetris" {
            self.labels = vec![Label::Button("Play"), Label::Button("Settings"), Label::Button("Exit")];
        }
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(key) = self.rl.get_key_pressed() {
            match key {
                KeyboardKey::KEY_ENTER => {
                    self.enter_selected()?;
                },
                KeyboardKey::KEY_UP => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                },
                KeyboardKey::KEY_DOWN => {
                    if self.selected < self.labels.len() - 1 {
                        self.selected += 1;
                    }
                },
                _ => {},
            }
        }

        if self.rl.window_should_close() {
            self.should_close = true;
        }

        Ok(())
    }

    fn handle_mouse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mouse = self.rl.get_mouse_position();

        for index in 0..self.labels.len() {
            let rec = Rectangle::new(
                ((WIDTH / 2) - 150) as f32,
                ((HEIGHT / 2) + (90 * index as i32)) as f32,
                300.0,
                60.0,
            );

            if rec.check_collision_point_rec(mouse) {
                if self.rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                    self.enter_selected()?;
                } else {
                    self.selected = index;
                }
            }
        }

        Ok(())
    }

    fn lock_size(&mut self) {
        if self.rl.is_window_resized() {
            self.rl.set_window_size(WIDTH, HEIGHT);
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.should_close {
            self.draw()?;
            self.lock_size();
            self.handle_input()?;
            self.handle_mouse()?;
            self.update_menu();
        }

        Ok(())
    }
}


