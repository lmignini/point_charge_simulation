
use std::fmt;
use macroquad::color::{Color, GREEN, LIGHTGRAY, PURPLE, WHITE};
use macroquad::color_u8;
use macroquad::math::{polar_to_cartesian, Rect, Vec2};
use macroquad::prelude::{draw_circle, draw_line};
use macroquad::shapes::draw_triangle;
use macroquad::text::{draw_text, get_text_center};
use macroquad::window::set_fullscreen;
use crate::charges::{PointCharge, Sign};
use crate::Drawable;


pub fn draw_arrow(application_point: Vec2, ending_point: Vec2,  body_size: f32, arrowhead_size: f32,  color: Color) {
    if application_point == ending_point {
        draw_circle(application_point.x, application_point.y, 3.0, color);
        return
    }

    // Calculate the direction vector and normalize it
    let dir_vector = ending_point - application_point;

    let direction = dir_vector.normalize_or_zero();

    if direction == Vec2::ZERO {
        // eprintln!("Error calculating normalized vector when drawing arrow! Fallback to drawing a single line!");
        draw_line(application_point.x, application_point.y, ending_point.x, ending_point.y, 5.0, color);
    } else {
        // Calculate the perpendicular vector

        let perpendicular = direction.perp();

        // Calculate arrowhead size

        // Calculate where the line should end (slightly before the ending_point)
        let line_end = ending_point - direction * arrowhead_size * 0.5;

        // Draw the line from start to the shortened end point
        draw_line(application_point.x, application_point.y, line_end.x, line_end.y, body_size, color);

        // Calculate the points for the arrowhead triangle
        let left_corner = ending_point - direction * arrowhead_size + perpendicular * arrowhead_size * 0.5;
        let right_corner = ending_point - direction * arrowhead_size - perpendicular * arrowhead_size * 0.5;

        // Draw the triangle with tip exactly at ending_point
        draw_triangle(
            ending_point,
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
    is_fixed: bool,
}
impl ChargeCircle {
    pub fn new(center: Vec2, radius: f32, color: Color, symbol: Option<Sign>, is_fixed: bool) -> Self {
        ChargeCircle { center, radius, color, symbol, is_fixed}
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

            if self.is_fixed {
                draw_text("f", self.center.x +self.radius /6.0, self.center.y - self.radius /4.0, 16.0, WHITE);
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
    const MAX_ARROW_MAGNITUDE: f32 = PointCharge::DEFAULT_RADIUS * 2.0;
    pub fn new(application_point: Vec2, rho: f32, max_magnitude:f32, theta: f32, color: Color) -> Self {
        let raw_magnitude =  rho;
        let scaled_magnitude = 46.0 + raw_magnitude * Self::MAX_ARROW_MAGNITUDE / max_magnitude;

        //dbg!(raw_magnitude, max_magnitude, scaled_magnitude);
        // assert!(max_magnitude >= raw_magnitude, "{max_magnitude} < {raw_magnitude}");
        // assert!(scaled_magnitude <= Self::MAX_ARROW_MAGNITUDE+ 10.0, "{scaled_magnitude} > {}", Self::MAX_ARROW_MAGNITUDE + 10.0); // +10.0 for tolerance
        let ending_point = (polar_to_cartesian(scaled_magnitude.min(Self::MAX_ARROW_MAGNITUDE), theta) + application_point);
        ForceArrow { application_point, ending_point, color }
    }


}

impl Drawable for ForceArrow {
    fn draw(&self) {
        draw_arrow(self.application_point, self.ending_point, 2.5, 7.5, self.color);
    }
}


pub struct FieldArrow {
    application_point: Vec2,
    ending_point: Vec2,
    color: Color,
}

impl FieldArrow {
    const MAX_RHO: f32= 20.0;
    pub fn new(application_point: Vec2, rho: f32, max_magnitude:f32, theta: f32, potential: f32) -> Self {
        let mut ending_point= application_point;
        let color_intensity =  (30 +f32::round((rho * 255.0) / max_magnitude) as u16).min(255) as u8;

        let color = color_u8!(color_intensity, color_intensity, color_intensity, 255);

        if (rho > 0.0) {
            ending_point = polar_to_cartesian(rho.min(Self::MAX_RHO),theta ) + application_point;
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
            ending_point = polar_to_cartesian(rho.min(Self::MAX_RHO),theta ) + self.application_point;
        }
        self.color = color_u8!(color_intensity, color_intensity, color_intensity, 255);;
        self.ending_point = ending_point;
    }
}

impl Drawable for FieldArrow {
    fn draw(&self) {
        // dbg!(self.application_point, self.ending_point, self.application_point.distance(self.ending_point));
        draw_arrow(self.application_point, self.ending_point, 2.5, 7.5, self.color);
    }
}