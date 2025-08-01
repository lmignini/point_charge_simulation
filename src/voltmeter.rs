use macroquad::color::{GREEN, RED, WHITE};
use macroquad::math::{Rect, Vec2};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle_lines};
use macroquad::text::{draw_text, draw_text_ex, get_text_center, TextParams};
use crate::charges::{calculate_potential, PointCharge};

pub struct Voltmeter {
    reticle_center: Vec2,
    rectangle: Rect,
    measured_potential: f32,
    pub is_active: bool,
    pub equipotentials: Vec<f32>,

}

impl Default for Voltmeter {
    fn default() -> Self {
        Self::new()
    }
}



impl Voltmeter {
    const RETICLE_RADIUS: f32 = 24.0;
    const RECTANGLE_VERTICAL_OFFSET: f32 = Self::RETICLE_RADIUS + 10.0;
    const RECTANGLE_HORIZONTAL_OFFSET: f32 = 2.0*Self::RETICLE_RADIUS;
    const HORIZONTAL_TEXT_OFFSET: f32 = Self::RETICLE_RADIUS / 4.0;
    const VERTICAL_TEXT_OFFSET: f32 = 25.0;
    const MEASURE_FONT_SIZE: u16 = 24;
    #[must_use]
    pub fn new() -> Self {
        let reticle_center = Vec2::ZERO;
        let rectangle: Rect = Rect {
            x: reticle_center.x - Self::RECTANGLE_HORIZONTAL_OFFSET,
            y: reticle_center.y + Self::RECTANGLE_VERTICAL_OFFSET,
            w: Self::RETICLE_RADIUS * 6.0,
            h: Self::RETICLE_RADIUS * 2.0,
        };
        Voltmeter {
            reticle_center: reticle_center,
            rectangle: rectangle,
            measured_potential: 0.0,
            is_active: false,
            equipotentials: vec![]
        }
    }
    fn movement(&mut self, new_position: Vec2) {
        self.reticle_center = new_position;
        self.rectangle = Rect {
            x: self.reticle_center.x - Self::RECTANGLE_HORIZONTAL_OFFSET,
            y: self.reticle_center.y + Self::RECTANGLE_VERTICAL_OFFSET,
            w: Self::RETICLE_RADIUS * 4.0,
            h: Self::RETICLE_RADIUS * 2.0,
        };
    }
    pub fn update(&mut self, new_position: Vec2, charges: &Vec<PointCharge>) {
        self.movement(new_position);
        self.measured_potential = calculate_potential(&self.reticle_center, charges);
    }

    pub fn add_equipotential(&mut self) {
        self.equipotentials.push(self.measured_potential);
    }

    pub fn clear_equipotentials(&mut self) {
        self.equipotentials.clear();
    }

    pub fn draw(&self) {
        if self.is_active {
            draw_line(self.reticle_center.x, self.reticle_center.y + Self::RETICLE_RADIUS, self.reticle_center.x, self.reticle_center.y + Self::RECTANGLE_VERTICAL_OFFSET, 5.0, WHITE);
            draw_circle_lines(self.reticle_center.x, self.reticle_center.y, Self::RETICLE_RADIUS, 3.0, WHITE);
            draw_line(self.reticle_center.x - Self::RETICLE_RADIUS, self.reticle_center.y, self.reticle_center.x + Self::RETICLE_RADIUS, self.reticle_center.y, 1.0, WHITE);
            draw_line(self.reticle_center.x, self.reticle_center.y - Self::RETICLE_RADIUS, self.reticle_center.x, self.reticle_center.y + Self::RETICLE_RADIUS, 1.0, WHITE);
            draw_rectangle_lines(self.rectangle.x, self.rectangle.y, self.rectangle.w, self.rectangle.h, 4.0, WHITE);
            let measured_string = format!("{:.1} V", self.measured_potential);
            let text_dimensions = get_text_center(&measured_string, None, Self::MEASURE_FONT_SIZE, 1.0, 0.0);

            // Calculate centered position
            let text_x = self.rectangle.x + (self.rectangle.w /2.0) -  text_dimensions.x;
            let text_y = self.rectangle.y + (self.rectangle.h /2.0) - text_dimensions.y;

            draw_text_ex(&measured_string, text_x, text_y, TextParams {
                 font_size: Self::MEASURE_FONT_SIZE,
                color: GREEN,
                ..Default::default()
                          },);
        }
    }
}