use std::fmt;
use crate::geometry::{ForceArrow, ChargeCircle, FieldArrow};
use macroquad::color::{Color, BLUE, GREEN, LIGHTGRAY, RED, WHITE};
use macroquad::color_u8;
use macroquad::math::{cartesian_to_polar, polar_to_cartesian, Rect, UVec2, Vec2};
use macroquad::prelude::{draw_circle, draw_line, draw_rectangle_lines};
use crate::charges::Sign::Neutral;
use crate::Drawable;
use self::Sign::{Negative, Positive};


#[derive(PartialEq, Debug)]
pub enum Sign {
    Neutral = 0,
    Positive = 1,
    Negative = -1,

}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Positive {
            write!(f, "+")
        } else if *self == Negative {
            write!(f, "-")
        } else {
            write!(f, "/")
        }

    }
}

const K: f32 = 8.99 * 10e9;
const FORCE_SCALING_FACTOR: f32 = 50e5;



pub fn calculate_potential(point: &Vec2, charges: &Vec<PointCharge>) -> f32 {
    let mut potential: f32 = 0.0;
    for charge in charges {
        let distance = point.distance(charge.center);
        potential += K * charge.q / distance;

    }

    return potential;

}
pub fn color_based_on_potential(potential: f32, max_potential: f32) -> Color {
    let mut color_intensity: u8 = 0;
    if potential.abs() > max_potential {
        color_intensity = 255;
    } else if max_potential > 0.1 {
        color_intensity = (f32::round(2.0* ((potential.abs() * 255.0) / max_potential)) as u16).min(255) as u8;;
    }
    // dbg!(color_intensity);
    let color: Color;
    if potential > 1.0 {
        color = color_u8!(color_intensity, 0, 0, 255);
    } else if potential < -1.0 {
        color = color_u8!(0, 0, color_intensity, 255);
    } else {
        color = color_u8!(color_intensity, color_intensity, color_intensity, 255);
    }
    return color;
}


#[derive(Debug)]
pub struct PointCharge {
    pub id: usize,
    pub center: Vec2,
    pub drawing_circle: ChargeCircle,
    pub sign: Sign,
    pub is_fixed: bool,
    is_selected: bool,
    pub is_colliding: bool,

    pub m: f32,
    q: f32,
    forces: Vec<Vec2>,
    net_force: Vec2,
    max_force_magnitude: f32,
    acceleration: Vec2,
    pub velocity: Vec2,

}



impl PointCharge {
    pub const DEFAULT_RADIUS: f32 = 16.0;
    const DEFAULT_CHARGE: f32 = 2e-8;
    const DEFAULT_MASS: f32 = 1.67 * 10e-3;
    const NULL_VECTOR: Vec2 = Vec2::ZERO;
    const ENCLOSING_SQUARE_PADDING: f32 = Self::DEFAULT_RADIUS * 2.5;
    const FRICTION: f32 = 0.95;

    pub fn new_positive_charge(id: usize, center: Vec2, is_fixed: bool) -> Self {
        let drawing_circle = ChargeCircle::new(
            center,
            PointCharge::DEFAULT_RADIUS,
            RED,
            Some(Positive),
            is_fixed);

        PointCharge {
            id: id,
            center: center,
            drawing_circle: drawing_circle,
            sign: Positive,
            is_fixed: is_fixed,
            is_selected: false,
            is_colliding: false,

            m: Self::DEFAULT_MASS,
            q: Self::DEFAULT_CHARGE,
            forces: vec![],
            net_force: Self::NULL_VECTOR,
            max_force_magnitude: 0.0,
            acceleration: Self::NULL_VECTOR,
            velocity: Self::NULL_VECTOR

        }


    }

    pub fn new_negative_charge(id: usize, center: Vec2, is_fixed: bool) -> Self {
        let drawing_circle = ChargeCircle::new(
            center,
            PointCharge::DEFAULT_RADIUS,
            BLUE,
        Some(Negative),
            is_fixed);

        PointCharge {
            id: id,
            center: center,
            drawing_circle: drawing_circle,
            sign: Negative,
            is_fixed: is_fixed,
            is_selected: false,
            is_colliding: false,

            m: Self::DEFAULT_MASS,
            q: -Self::DEFAULT_CHARGE,
            forces: vec![],
            net_force: Self::NULL_VECTOR,
            max_force_magnitude: 0.0,
            acceleration: Self::NULL_VECTOR,
            velocity: Self::NULL_VECTOR

        }



    }


    // Add this constructor to PointCharge impl
    pub fn new_neutral_charge_from_merge(id: usize, center: Vec2, is_fixed: bool) -> Self {
        let drawing_circle = ChargeCircle::new(
            center,
            PointCharge::DEFAULT_RADIUS,
            LIGHTGRAY, // Gray for neutral
            Some(Neutral),
        is_fixed); // No sign for neutral charge

        PointCharge {
            id,
            center,
            drawing_circle,
            sign: Sign::Neutral,
            is_fixed,
            is_selected: false,
            is_colliding: false,

            m: 2.0*Self::DEFAULT_MASS,
            q: 0.0, // Neutral has zero charge
            forces: vec![],
            net_force: Self::NULL_VECTOR,
            max_force_magnitude: 0.0,
            acceleration: Self::NULL_VECTOR,
            velocity: Self::NULL_VECTOR
        }
    }

    pub fn force_with(&mut self, point_charge: &PointCharge) -> Vec2 {
        let distance_squared = self.center.distance_squared(point_charge.center);
        let magnitude =  FORCE_SCALING_FACTOR * K * self.q * point_charge.q / distance_squared;
        let delta = Vec2::new(self.center.x - point_charge.center.x, self.center.y - point_charge.center.y);
        let direction = delta.y.atan2(delta.x);

        let force = Vec2::new(magnitude, direction);
        self.forces.push(force);
        force
    }

    pub fn calculate_net_force(&mut self) {
        self.net_force = cartesian_to_polar(self.forces.iter()
            .map(|force| polar_to_cartesian(force.x, force.y))
            .sum());
    }

    pub fn calculate_max_force(&mut self) {

       self.max_force_magnitude = self.forces.iter().max_by(|force1, force2| force1.x.partial_cmp(&force2.x).expect("Failed to compare!") ).unwrap_or(&Vec2::INFINITY).x;
        self.max_force_magnitude = self.max_force_magnitude.max((self.net_force).x);


        // dbg!(&self.forces, self.net_force, self.max_force_magnitude);

    }

    pub fn clear_forces(&mut self) {
        self.forces.clear();
        self.net_force = Self::NULL_VECTOR;
    }

    pub fn calculate_acceleration(&mut self) {

        if self.is_fixed {
            //dbg!(&self.forces);
            // dbg!(self.net_force);
            self.acceleration = Self::NULL_VECTOR;
        } else {
            self.acceleration = Vec2::new(self.net_force.x / self.m, self.net_force.y);
        }
    }

    pub fn calculate_velocity(&mut self) {
        if self.is_fixed {
            self.velocity = Self::NULL_VECTOR;
        } else {
            let mut new_speed = self.acceleration.x + self.velocity.x * Self::FRICTION;
            if new_speed <= 0.005 || self.is_colliding {
                new_speed = 0.0;
            }
            self.velocity = Vec2::new(new_speed, self.net_force.y);
        }
    }


    pub fn movement(&mut self, delta: f32) {
        if self.is_colliding {
            return;
        }
        let cartesian_velocity = polar_to_cartesian(self.velocity.x, self.velocity.y);
        self.center += cartesian_velocity * delta;
        self.drawing_circle.center += cartesian_velocity * delta;
        // dbg!(self.center, cartesian_velocity);
    }



    pub fn check_collision_with(&mut self, point_charge: &mut PointCharge, delta_time: f32) {
        // Reset collision state
        self.is_colliding = false;

        // Get distance between charges
        let distance_squared = self.center.distance_squared(point_charge.center);
        let min_distance = (self.drawing_circle.radius + point_charge.drawing_circle.radius);

        // Check if colliding
        if distance_squared < min_distance.powi(2) {
            self.is_colliding = true;
            point_charge.is_colliding = true;

            // Only apply collision response if at least one charge is not fixed
            if !self.is_fixed || !point_charge.is_fixed {
                let distance = distance_squared.sqrt();
                if distance > 0.0 {
                    let overlap = min_distance - distance;
                    let direction = (self.center - point_charge.center).normalize();

                    // Calculate how much each charge should move (based on fixed status)
                    let self_factor = if self.is_fixed { 0.0 } else { 1.0 };
                    let other_factor = if point_charge.is_fixed { 0.0 } else { 1.0 };

                    // Normalize movement factors
                    let total_factor = self_factor + other_factor;
                    let (self_factor, other_factor) = if total_factor > 0.0 {
                        (self_factor / total_factor, other_factor / total_factor)
                    } else {
                        (0.0, 0.0)
                    };

                    // Apply position correction with strength proportional to overlap
                    let correction_factor = 1.5; // Stronger correction for low FPS
                    self.center += direction * overlap * self_factor * correction_factor;
                    point_charge.center -= direction * overlap * other_factor * correction_factor;

                    // Update drawing positions
                    self.drawing_circle.center = self.center;
                    point_charge.drawing_circle.center = point_charge.center;

                    // Apply velocity correction (bounce effect)
                    // Apply velocity correction (bounce effect)
                    if !self.is_fixed && !point_charge.is_fixed {
                        // Calculate relative velocity
                        let rel_velocity = self.velocity - point_charge.velocity;
                        let vel_along_normal = rel_velocity.dot(direction);

                        // Only apply bounce if objects are moving toward each other
                        if vel_along_normal < 0.0 {
                            let restitution = 0.5; // Bounciness factor

                            // Calculate impulse with mass consideration
                            let impulse_scalar = -(1.0 + restitution) * vel_along_normal /
                                (1.0/self.m + 1.0/point_charge.m);

                            // Apply velocity changes proportional to inverse mass
                            self.velocity += direction * impulse_scalar / self.m;
                            point_charge.velocity -= direction * impulse_scalar / point_charge.m;
                        }
                    }
                }
            }
        }
    }


    pub fn enclosing_square(&self) -> Rect {
        self.drawing_circle.enclosing_square(Self::ENCLOSING_SQUARE_PADDING)
    }


    pub fn potential_contribution_at(&self, point: &Vec2) -> f32 {
        let distance = self.center.distance(*point);
        return K * self.q / distance;
    }

    // Add this method to check for opposite charges
    pub fn should_merge_with(&self, other: &PointCharge) -> bool {
        (self.sign == Positive && other.sign == Negative) ||
            (self.sign == Negative && other.sign == Positive)
    }

    pub fn draw_forces(&self ) {
        for force in &self.forces {
            ForceArrow::new(self.center, force.x, self.max_force_magnitude, force.y, LIGHTGRAY ).draw();
        }

    }

    pub fn draw_net_force(&self) {
        ForceArrow::new(self.center, self.net_force.x, self.max_force_magnitude, self.net_force.y, GREEN).draw();
    }

    pub fn draw(&self) {


        self.drawing_circle.draw();
        // let tmp = self.drawing_circle.enclosing_square(Self::ENCLOSING_SQUARE_PADDING);
        // draw_rectangle_lines(tmp.x, tmp.y, tmp.w, tmp.h, 3.0, GREEN);
    }

}


impl fmt::Display for PointCharge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID: {}, Charge center: ({}, {}), Sign: {}",self.id,  self.center.x, self.center.y, self.sign)
    }
}
pub struct TestCharge {
    pub center: Vec2,
    pub drawing_arrow: FieldArrow,
    pub is_hidden: bool,
    q: f32,
    forces: Vec<Vec2>,
    pub net_force: Vec2,
    max_force_magnitude: f32,
    potential: f32
}

impl TestCharge {

    const NULL_VECTOR: Vec2 = Vec2::ZERO;
    pub fn new(center: Vec2) -> Self {
        TestCharge {
            center,
            drawing_arrow: FieldArrow::new(center, 0.0, 0.0, 0.0, 0.0),
            is_hidden: false,
            q: 1.0,
            forces: vec![],
            net_force: Self::NULL_VECTOR,
            max_force_magnitude: 0.0,
            potential: 0.0,
        }

    }

    pub fn force_with(&mut self, point_charge: &PointCharge) -> Vec2 {
        let distance_squared = self.center.distance_squared(point_charge.center);
        let magnitude =  FORCE_SCALING_FACTOR * K * point_charge.q / distance_squared;
        let delta = Vec2::new(self.center.x - point_charge.center.x, self.center.y - point_charge.center.y);
        let direction = delta.y.atan2(delta.x);

        let force = Vec2::new(magnitude, direction);
        self.forces.push(force);
        force
    }

    pub fn calculate_net_force(&mut self) {
        self.net_force = cartesian_to_polar(self.forces.iter()
            .map(|force| polar_to_cartesian(force.x, force.y))
            .sum());
    }

    pub fn set_max_force(&mut self, max_magnitude: f32) {

        // self.max_force_magnitude = self.forces.iter().max_by(|force1, force2| force1.x.partial_cmp(&force2.x).expect("Failed to compare!") ).unwrap_or(&Vec2::INFINITY).x;
        self.max_force_magnitude = max_magnitude;


        // dbg!(&self.forces, self.net_force, self.max_force_magnitude);

    }

    pub fn clear_forces(&mut self) {
        self.forces.clear();
        self.net_force = Self::NULL_VECTOR;
        self.max_force_magnitude = 0.0;
        self.potential = 0.0;
    }


    pub fn update_arrow(&mut self) {
        self.drawing_arrow.update((self.net_force.x), self.max_force_magnitude, self.net_force.y, self.potential);
    }
    pub fn draw(&self) {
        if (!self.is_hidden) {
            self.drawing_arrow.draw();
        }
    }
}
