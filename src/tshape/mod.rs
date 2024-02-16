use crate::game::Position;

use rand::Rng;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

pub struct TShape {
    shapes: Vec<Vec<Position>>,
}

impl TShape {
    pub fn load(file: &str) -> Result<TShape, Box<dyn std::error::Error>> {
        let fd = File::open(file)?;
        let reader = BufReader::new(fd);

        let mut shapes = Vec::new();

        for result in reader.lines() {
            let line = result?;

            let indices = line.split(' ')
                .filter(|x| x.len() == 3)
                .collect::<Vec<&str>>();

            let mut shape: Vec<Position> = Vec::new();
            for indice in &indices {
                let position = indice.split('-').collect::<Vec<&str>>();

                shape.push(Position {
                    x: position[0].parse()?,
                    y: position[1].parse()?,
                });
            }

            shapes.push(shape);
        }

        Ok(TShape {
            shapes,
        })
    }

    pub fn rand_shape(&self) -> Vec<Position> {
        let mut rng = rand::thread_rng();

        self.shapes[rng.gen_range(0..self.shapes.len())].clone()
    }
}

