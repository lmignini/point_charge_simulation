
use std::fmt;
use macroquad::color::{Color, GREEN, LIGHTGRAY, WHITE};
use macroquad::color_u8;
use macroquad::math::{polar_to_cartesian, Rect, Vec2};
use macroquad::prelude::{draw_circle, draw_line};
use macroquad::shapes::draw_triangle;
use macroquad::text::{draw_text, get_text_center};
use crate::charges::{Sign};
use crate::Drawable;


fn draw_arrow(application_point: Vec2, ending_point: Vec2, color: Color) {
    if application_point == ending_point {
        draw_circle(application_point.x , application_point.y, 3.0, WHITE);
        return
    }
    draw_line(application_point.x, application_point.y, ending_point.x, ending_point.y, 5.0, color);

    // Draw the arrowhead (tip)
    let arrowhead_size = 15.0;

    // Calculate the direction vector and normalize it
    let dir_vector = ending_point - application_point;
    let dir_length = (dir_vector.x * dir_vector.x + dir_vector.y * dir_vector.y).sqrt();

    if dir_length > 0.0 {
        let direction = Vec2::new(dir_vector.x / dir_length, dir_vector.y / dir_length);
        let tip_offset = Vec2::new(dir_vector.x / 16.0, dir_vector.y / 16.0);
        // Calculate the perpendicular vector
        let perpendicular = Vec2::new(-direction.y, direction.x);

        // Calculate the points for the arrowhead triangle
        let tip = ending_point + tip_offset;
        let left_corner = ending_point - direction * arrowhead_size + perpendicular * arrowhead_size * 0.5;
        let right_corner = ending_point - direction * arrowhead_size - perpendicular * arrowhead_size * 0.5;

        // Draw the triangle
        draw_triangle(
            tip,
            left_corner,
            right_corner,
            color
        );
    }
}

#[derive(Debug)]
pub struct ChargeCircle {
    pub(crate) center: Vec2,
    pub radius: f32,
    color: Color,
    symbol: Option<Sign>,
}
impl ChargeCircle {
    pub fn new(center: Vec2, radius: f32, color: Color, symbol: Option<Sign>) -> Self {
        ChargeCircle { center, radius, color, symbol }
    }

    pub fn enclosing_square(&self, padding: f32) -> Rect {
        Rect {
            x: self.center.x - self.radius - padding / 2.0,
            y: self.center.y - self.radius - padding / 2.0,
            w: self.radius * 2.0 + padding,
            h: self.radius * 2.0+ padding,
        }
    }
}
impl Drawable for ChargeCircle {
    fn draw(&self) {
        draw_circle(self.center.x, self.center.y, self.radius, self.color);

        let thickness: f32 = self.radius / 4.0;
        if let Some(symbol) = &self.symbol {
            match symbol {
                Sign::Positive => {
                    draw_line(self.center.x - self.radius / 2.0, self.center.y, self.center.x + self.radius  / 2.0, self.center.y, thickness, WHITE);
                    draw_line(self.center.x, self.center.y - self.radius  / 2.0, self.center.x, self.center.y + self.radius  / 2.0, thickness, WHITE);
                }
                Sign::Negative => {
                    draw_line(self.center.x - self.radius  / 2.0, self.center.y, self.center.x + self.radius  / 2.0, self.center.y, thickness, WHITE);
                }
                Sign::Neutral => ()
            }


        }
    }
}

pub struct ForceArrow {
    application_point: Vec2,
    ending_point: Vec2,
    color: Color

}

impl ForceArrow {
    const MAX_ARROW_MAGNITUDE: f32 = 90.0;
    pub fn new(application_point: Vec2, rho: f32, max_magnitude:f32, theta: f32, color: Color) -> Self {
        let raw_magnitude =  rho;
        let scaled_magnitude = 46.0 + raw_magnitude * Self::MAX_ARROW_MAGNITUDE / max_magnitude;

        //dbg!(raw_magnitude, max_magnitude, scaled_magnitude);
        // assert!(max_magnitude >= raw_magnitude, "{max_magnitude} < {raw_magnitude}");
        // assert!(scaled_magnitude <= Self::MAX_ARROW_MAGNITUDE+ 10.0, "{scaled_magnitude} > {}", Self::MAX_ARROW_MAGNITUDE + 10.0); // +10.0 for tolerance
        let ending_point = (polar_to_cartesian(scaled_magnitude, theta) + application_point);
        ForceArrow { application_point, ending_point, color }
    }


}

impl Drawable for ForceArrow {
    fn draw(&self) {
        draw_arrow(self.application_point, self.ending_point, self.color);
    }
}


pub struct FieldArrow {
    application_point: Vec2,
    ending_point: Vec2,
    color: Color,
}

impl FieldArrow {
    pub fn new(application_point: Vec2, rho: f32, max_magnitude:f32, theta: f32, potential: f32) -> Self {
        let mut ending_point= application_point;
        let color_intensity =  (30 +f32::round((rho * 255.0) / max_magnitude) as u16).min(255) as u8;

        let color = color_u8!(color_intensity, color_intensity, color_intensity, 255);

        if (rho > 0.0) {
            ending_point = polar_to_cartesian(rho.min(30.0),theta );
        }

        FieldArrow {
            application_point: application_point,
            ending_point: ending_point,
            color: color
        }
    }



    pub fn update(&mut self, rho: f32, max_magnitude: f32, theta: f32, potential: f32) {
        let mut ending_point= self.application_point;
        let color_intensity =  (30 +f32::round((rho * 255.0) / max_magnitude) as u16).min(255) as u8;
        // let color = Self::calculate_color_by_potential(potential, color_intensity);
        if (rho > 0.0) {
            ending_point = polar_to_cartesian(rho.min(50.0),theta ) + self.application_point;
        }
        self.color = color_u8!(color_intensity, color_intensity, color_intensity, 255);;
        self.ending_point = ending_point;
    }
}

impl Drawable for FieldArrow {
    fn draw(&self) {
        // dbg!(self.application_point, self.ending_point, self.application_point.distance(self.ending_point));
        draw_arrow(self.application_point, self.ending_point, self.color);
    }
}