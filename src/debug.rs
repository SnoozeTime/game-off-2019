use crate::components;
use quicksilver::{
    geom::{self, Line, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image, PixelFormat},
    lifecycle::{Asset, Window},
    Result,
};
use specs::prelude::*;

pub struct DebugText {
    debug_text: Image,
}

impl DebugText {
    pub fn new() -> Self {
        Self {
            debug_text: Image::from_raw(&[], 0, 0, PixelFormat::RGB).unwrap(),
        }
    }

    pub fn update_text(&mut self, font: &mut Asset<Font>, t: &str) -> Result<()> {
        let mut img = Image::from_raw(&[], 0, 0, PixelFormat::RGB).unwrap();
        font.execute(|font| {
            let style = FontStyle::new(32.0, Color::GREEN);
            img = font.render(t, &style).unwrap();
            Ok(())
        })?;
        self.debug_text = img;

        Ok(())
    }

    pub fn draw(&self, window: &mut Window) -> Result<()> {
        let translation = geom::Transform::translate((0, 500));
        window.draw_ex(
            &self.debug_text.area(),
            Img(&self.debug_text),
            translation,
            0,
        );

        Ok(())
    }
}

// ------------------------------------------------------
/// This will display the line of sights for each enemy.
pub fn display_enemy_line_of_sights(world: &World, window: &mut Window) {
    let enemy_storage = world.read_storage::<components::Enemy>();
    let transform_storage = world.read_storage::<components::Transform>();
    for (t, enemy) in (&transform_storage, &enemy_storage).join() {
        let d = t.direction();
        let origin_ray = t.position;
        window.draw_ex(
            &Line::new(origin_ray, origin_ray + d * enemy.detection_distance),
            Col(Color::BLUE),
            geom::Transform::IDENTITY,
            99,
        );
        let left_dir: Vector =
            (geom::Transform::rotate(t.rotation - enemy.detection_angle) * Vector::X).normalize();

        window.draw_ex(
            &Line::new(origin_ray, origin_ray + left_dir * enemy.detection_distance),
            Col(Color::BLUE),
            geom::Transform::IDENTITY,
            99,
        );

        let right_dir: Vector =
            (geom::Transform::rotate(t.rotation + enemy.detection_angle) * Vector::X).normalize();

        window.draw_ex(
            &Line::new(
                origin_ray,
                origin_ray + right_dir * enemy.detection_distance,
            ),
            Col(Color::BLUE),
            geom::Transform::IDENTITY,
            99,
        );
    }
}
