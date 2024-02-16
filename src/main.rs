mod tshape;
mod game;
mod menu;

use tshape::TShape;
use game::Renderer;
use menu::Menu;

use std::process;

fn main() {
    let mut menu = match Menu::new() {
        Ok(menu) => menu,
        Err(err) => {
            println!("[ERROR] failed to initialize menu: {}", err.to_string());
            process::exit(1);
        },
    };

    if let Err(err) = menu.run() {
        println!("[ERROR] failed to run menu: {}", err.to_string());
        process::exit(1);
    }
}

