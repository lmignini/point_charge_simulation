use std::cmp::max;
use rayon::prelude::*;
use std::collections::HashMap;
use std::default::Default;
use itertools::Itertools;
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::iter::Map;
use std::path::Iter;
use std::{slice, vec};
use std::alloc::System;
use is_close::{is_close, AVERAGE};
use macroquad::math::f32;
use macroquad::miniquad::gl::glPolygonOffset;
use macroquad::prelude::*;
use macroquad::prelude::scene::clear;
use point_charge_simulation::geometry::{ForceArrow, ChargeCircle};
use point_charge_simulation::{charges, Drawable, SplitOneMut};
use point_charge_simulation::charges::{ color_based_on_potential, PointCharge, TestCharge};
use point_charge_simulation::charges::Sign::Neutral;
use point_charge_simulation::voltmeter::Voltmeter;
use crate::SimulationState::Running;

const WINDOW_WIDTH: u16 = 1600;
const WINDOW_HEIGHT: u16 = 1000;

const ELECTRIC_FIELD_DENSITY: usize = 50;
const POTENTIAL_DENSITY: usize = 1;
const PADDING_FROM_WINDOW_BORDERS: u16 = 0;

const TRANSPARENT_COLOR: Color = color_u8!(0, 0, 0, 0);
const TRANSPARENT_COLOR_SLICE: &[Color; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize] = &[TRANSPARENT_COLOR; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize];


fn window_conf() -> Conf {
    Conf {
        window_title: "Point charge simulation".to_owned(),
        window_width: i32::from(WINDOW_WIDTH),
        window_height: i32::from(WINDOW_HEIGHT),
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(PartialEq)]
enum SimulationState {
    Running,
    Paused
}

#[macroquad::main(window_conf)]
async fn main() {

    // dbg!(TRANSPARENT_COLOR_SLICE.len());
    let mut charges: Vec<PointCharge> = vec![];
    let mut simulation_state: SimulationState = Running;

    let field_x_points = (PADDING_FROM_WINDOW_BORDERS..=WINDOW_WIDTH - PADDING_FROM_WINDOW_BORDERS).step_by(ELECTRIC_FIELD_DENSITY);
    let field_y_points = (PADDING_FROM_WINDOW_BORDERS..=WINDOW_HEIGHT - PADDING_FROM_WINDOW_BORDERS).step_by(ELECTRIC_FIELD_DENSITY);
    let field_xy_meshgrid = field_x_points.cartesian_product(field_y_points);

    let mut test_charges: Vec<TestCharge> = vec![];
    for (x,y) in field_xy_meshgrid {
      test_charges.push(TestCharge::new(Vec2::new(f32::from(x), f32::from(y))));
    }


    let potential_x_points = (0..WINDOW_WIDTH).step_by(POTENTIAL_DENSITY);
    let potential_y_points = (0..WINDOW_HEIGHT).step_by(POTENTIAL_DENSITY);
    let potential_xy_meshgrid = potential_x_points.cartesian_product(potential_y_points);
    let mut potential_map: HashMap<UVec2, f32> = HashMap::with_capacity((WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize));
    let mut max_potential: f32 = 0.0;
    let mut potential_image = Image::gen_image_color(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, BLACK);

    let transparent_equipotential_lines: Image = Image::gen_image_color(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, color_u8!(255,255,255, 0));
    let mut equipotential_lines_image = Image::gen_image_color(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, color_u8!(255,255,255, 0));

    for (x, y) in potential_xy_meshgrid {
        potential_map.insert(UVec2::new(u32::from(x), u32::from(y)), 0.0);
        // potential_vec2.push(Vec2::new(x as f32, y as f32));
    }

    let mut voltmeter: Voltmeter = Voltmeter::new();




    loop {

        clear_background(BLACK);

        let delta_time = get_frame_time();
        let mouse_position = Vec2 { x: mouse_position().0, y: mouse_position().1 };
        if is_key_pressed(KeyCode::C) {
            voltmeter.clear_equipotentials();
            equipotential_lines_image = transparent_equipotential_lines.clone();
        }
        if is_key_pressed(KeyCode::V) {
            voltmeter.is_active = !voltmeter.is_active;
        }
        if simulation_state == Running {
            if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right) {
                if voltmeter.is_active && is_mouse_button_pressed(MouseButton::Left) {
                    voltmeter.add_equipotential();
                }
                let mut mouse_pointer_is_over_charge = false;

                for charge in &charges {
                    if charge.enclosing_square().contains(mouse_position) {
                        mouse_pointer_is_over_charge = true;
                    }
                }
                if !mouse_pointer_is_over_charge && !voltmeter.is_active {

                    spawn_charge(&mut charges, mouse_position);
                }
            }
            clear_potential(&mut potential_map);

            update_field(&mut test_charges, &charges);
            update_charges(&mut charges, delta_time);
            max_potential = update_potential_and_return_max(&mut potential_map, &charges);

        }

        voltmeter.update(mouse_position, &charges);
        equipotential_lines_image = transparent_equipotential_lines.clone();
        update_potential_images(&potential_map, max_potential, &voltmeter.equipotentials, &mut potential_image, &mut equipotential_lines_image);
        draw_potential(&potential_image);
        draw_field(&test_charges);
        draw_equipotential_lines(&equipotential_lines_image);
        draw_charges(&charges);

        voltmeter.draw();

        draw_fps();
        next_frame().await;
    }


    let print_charge_vector = |charges: &Vec<PointCharge>| {
        println!("\nCharges list: ");
        for charge in charges {
            println!("{charge}");
        }
    };
}

fn update_field(test_charges: &mut Vec<TestCharge>, charges: &Vec<PointCharge>) {
    for test_charge in &mut *test_charges {
        test_charge.clear_forces();
    }
    for test_charge in &mut *test_charges {
        for charge in charges {
            test_charge.force_with(charge);
            // test_charge.potential_with(charge);
        }
    }

    for test_charge in &mut *test_charges {

        for charge in charges {
           if test_charge.center.distance_squared(charge.center) < (2.0*(24.0f32)).powi(2) {
               test_charge.is_hidden = true;
               break;
           }
            test_charge.is_hidden = false;
        }
        if (test_charge.is_hidden == false) {
            test_charge.calculate_net_force();
        }
    }


    let max_magnitude: f32 =  test_charges.iter()
        .map(|charge| charge.net_force.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(f32::INFINITY);
    for test_charge in test_charges {
        test_charge.set_max_force(max_magnitude);
        test_charge.update_arrow();
    }


    }


fn update_charges(charges: &mut Vec<PointCharge>, delta: f32) {
    // Clear forces
    for charge in charges.iter_mut() {
        charge.clear_forces();
        charge.is_colliding = false;
    }

    // Track merges
    let mut to_remove = vec![false; charges.len()];
    let mut new_charges = Vec::new();

    // Handle collisions and forces
    for i in 0..charges.len() {
        if to_remove[i] { continue; }

        for j in i+1..charges.len() {
            if to_remove[j] { continue; }

            let (first, second) = charges.split_at_mut(j);
            let charge1 = &mut first[i];
            let charge2 = &mut second[0];

            charge1.force_with(charge2);
            charge1.check_collision_with(charge2, delta);

            // Check for merge condition
            if charge1.is_colliding && charge1.should_merge_with(charge2) {
                to_remove[i] = true;
                to_remove[j] = true;

                // Create new neutral charge at midpoint
                let new_center = (charge1.center + charge2.center) * 0.5;

                // Conserve momentum in velocity
                let total_mass = charge1.m + charge2.m;
                let new_velocity = if total_mass > 0.0 {
                    (charge1.velocity * charge1.m + charge2.velocity * charge2.m) / total_mass
                } else {
                    Vec2::ZERO
                };

                let mut neutral = PointCharge::new_neutral_charge_from_merge(
                    charge1.id, // Reuse an ID
                    new_center,
                    charge1.is_fixed && charge2.is_fixed
                );

                neutral.velocity = new_velocity;
                new_charges.push(neutral);
                break;
            }
        }
        // Reverse interactions (i with j<i) - this ensures all charges get updated
        for j in 0..i {
            if to_remove[j] { continue; }

            let (first, second) = charges.split_at_mut(i);
            let charge2 = &mut first[j];
            let charge1 = &mut second[0];

            charge1.force_with(charge2);
            // No collision check needed here as it's already done in the forward pass
        }
    }

    // Remove merged charges (in reverse order)
    for i in (0..charges.len()).rev() {
        if to_remove[i] {
            charges.swap_remove(i);
        }
    }

    // Add new neutral charges
    charges.extend(new_charges);

    // Update physics
    for charge in charges.iter_mut() {
        charge.calculate_net_force();
        charge.calculate_max_force();
        charge.calculate_acceleration();
        charge.calculate_velocity();
        charge.movement(delta);
    }
}
fn draw_field(test_charges: &Vec<TestCharge>) {
    for test_charge in test_charges {
        test_charge.draw();
    }
}
fn draw_charges(charges: &Vec<PointCharge>) {
    /*for charge in charges {
        charge.draw_forces();
    }*/
    for charge in charges {
        charge.draw_net_force();
    }
    for charge in charges {
        charge.draw();
    }
}

fn spawn_charge(charges: &mut Vec<PointCharge>, mouse_position: Vec2) {

    let id = charges.len() + 1;
    if is_mouse_button_pressed(MouseButton::Left) {
        if is_key_down(KeyCode::LeftShift) {
            charges.push(PointCharge::new_positive_charge(id, mouse_position, true));
        } else {
            charges.push(PointCharge::new_positive_charge(id, mouse_position, false));
        }
    } else if is_mouse_button_pressed(MouseButton::Right) {
        if is_key_down(KeyCode::LeftShift) {
            charges.push(PointCharge::new_negative_charge(id, mouse_position, true));
        } else {
            charges.push(PointCharge::new_negative_charge(id, mouse_position, false));
        }
    }
}




fn clear_potential(potential_map: &mut HashMap<UVec2, f32>) {
    for value in potential_map.values_mut() {
        *value = 0.0;
    }
}



fn update_potential_and_return_max(potential_map: &mut HashMap<UVec2, f32>, charges: &Vec<PointCharge>) -> f32 {
    // Process calculations in parallel and modify values in place
    for charge in charges {
        if charge.sign == Neutral { continue}
        potential_map.par_iter_mut().for_each(|(key, value)| {
            let point_pos = Vec2::new(key.x as f32, key.y as f32);
            *value += charge.potential_contribution_at(&point_pos);
        });
    }
    // Return fixed max value as you're doing
    200.0
}

fn update_potential_images(potential_map: &HashMap<UVec2, f32>, max_potential: f32, equipotentials: &Vec<f32>, potential_image: &mut Image, equipotential_lines_image: &mut Image) {
    // Process all points in parallel and collect updates
    let updates: Vec<(UVec2, Color, bool)> = potential_map.par_iter()
        .map(|(point, potential)| {
            // Check for equipotential lines first
            let is_equipotential:bool;
            if (potential.abs() < 10.0) {
                is_equipotential = equipotentials.iter()
                    .any(|equip| is_close!(*potential+10.0, *equip+10.0, abs_tol=1e-1));
            } else {
            is_equipotential = equipotentials.iter()
                .any(|equip| is_close!(*potential, *equip, rel_tol=1e-2, method=AVERAGE));
                };

            // Determine color based on potential or equipotential status
            let color = if is_equipotential {
                GREEN
            } else {
                color_based_on_potential(*potential, max_potential)
            };

            (*point, color, is_equipotential)
        })
        .collect();
    // equipotential_lines_image.update(TRANSPARENT_COLOR_SLICE);
    // Apply updates to both images sequentially
    for (point, color, is_equipotential) in updates {
        potential_image.set_pixel(point.x, point.y, color);

        if is_equipotential {
            equipotential_lines_image.set_pixel(point.x, point.y, GREEN);
        }
    }
}
fn draw_potential(potential_image: &Image) {



     // Create a texture from the image and draw it
     let texture = Texture2D::from_image(potential_image);
     draw_texture(&texture, 0.0, 0.0, WHITE);
}

fn draw_equipotential_lines(equipotential_lines_image: &Image) {
    // Create a texture from the image and draw it
    let texture = Texture2D::from_image(equipotential_lines_image);
    draw_texture(&texture, 0.0, 0.0, WHITE);
}
