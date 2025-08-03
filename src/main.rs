use crate::SimulationState::{Paused, Running};
use is_close::{is_close, AVERAGE};
use itertools::Itertools;
use macroquad::math::f32;
use macroquad::prelude::*;
use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use ndarray::{Array, OwnedRepr};
use point_charge_simulation::charges::Sign::Neutral;
use point_charge_simulation::charges::{color_based_on_potential, PointCharge, TestCharge};
use point_charge_simulation::voltmeter::Voltmeter;
use std::default::Default;
use std::vec;
use macroquad::miniquad::CursorIcon;

const WINDOW_WIDTH: u16 = 800;
const WINDOW_HEIGHT: u16 = 500;

const ELECTRIC_FIELD_DENSITY: usize = 25;
const POTENTIAL_DENSITY: usize = 1;
const PADDING_FROM_WINDOW_BORDERS: u16 = 0;


const RUNNING_SIMULATION_TRIANGLE_VERTICES: (Vec2, Vec2, Vec2) = (
    Vec2::new(WINDOW_WIDTH as f32 - 10.0, 20.0),  // Left vertex
    Vec2::new(WINDOW_WIDTH as f32 - 30.0, 10.0),  // Top right
    Vec2::new(WINDOW_WIDTH as f32 - 30.0,  30.0),  // Bottom right
);

 const PAUSED_SIMULATION_RECTANGLES: (Rect, Rect) = (
     Rect::new(WINDOW_WIDTH as f32 - 30.0, 10.0, 7.0, 20.0),  // Left rectangle
     Rect::new(WINDOW_WIDTH as f32 - 17.0, 10.0, 7.0, 20.0)   // Right rectangle
     );

fn window_conf() -> Conf {
    Conf {
        window_title: "Point charge simulation".to_owned(),
        window_width: i32::from(WINDOW_WIDTH),
        window_height: i32::from(WINDOW_HEIGHT),
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(PartialEq, Eq)]
enum SimulationState {
    Running,
    Paused
}


#[allow(clippy::similar_names)]
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
    let mut max_potential: f32 = 0.0;
    let mut potential_image = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, BLACK);

    let transparent_equipotential_lines: Image = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, color_u8!(255,255,255, 0));
    let mut equipotential_lines_image = Image::gen_image_color(WINDOW_WIDTH , WINDOW_HEIGHT , color_u8!(255,255,255, 0));
    let mut potentials_array = Array::<(Vec2, f32), Ix2>::from_elem((WINDOW_WIDTH as usize,WINDOW_HEIGHT as usize), (Vec2::ZERO, 0.0f32));
    for (x, y) in potential_xy_meshgrid {
        potentials_array[[x as usize,y as usize]] = (Vec2::new(f32::from(x), f32::from(y)), 0.0);
    }
    let mut voltmeter: Voltmeter = Voltmeter::new();


    let mut cursor_is_over_a_charge: bool = false;
    let mut dragging_charge: Option<usize> = None;


    loop {
        clear_background(BLACK);
        let delta_time = get_frame_time();

        if is_key_pressed(KeyCode::C) {
            voltmeter.clear_equipotentials();
            equipotential_lines_image = transparent_equipotential_lines.clone();
        }
        if is_key_pressed(KeyCode::V) {
            voltmeter.is_active = !voltmeter.is_active;
        }
        if is_key_pressed(KeyCode::Escape) {
            toggle_simulation_state(&mut simulation_state);

        }

        let mouse_position = Vec2 { x: mouse_position().0, y: mouse_position().1 };

        if is_mouse_button_pressed(MouseButton::Left) {
            for (i, charge) in charges.iter().enumerate() {
                if charge.drawing_circle.contains(mouse_position){
                    cursor_is_over_a_charge = true;
                    dragging_charge = Some(i);
                    break;
                }
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            dragging_charge = None;
        }

        cursor_is_over_a_charge = false;
        for (i, charge) in charges.iter_mut().enumerate() {
            // Handle visual hover state
            if charge.drawing_circle.contains(mouse_position) {
                cursor_is_over_a_charge = true;
                charge.is_selected = true;
            } else {
                // Keep selection only if this is the charge being dragged
                charge.is_selected = dragging_charge == Some(i);
            }

            // Update position for dragged charge
            if is_mouse_button_down(MouseButton::Left) && dragging_charge == Some(i) {
                charge.center = mouse_position;
                charge.drawing_circle.center = mouse_position;
            }
        }
        if (cursor_is_over_a_charge) {
            miniquad::window::set_mouse_cursor(CursorIcon::Pointer);
        } else {
            miniquad::window::set_mouse_cursor(CursorIcon::Default);
        }


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
        if simulation_state == Running {

            update_charges(&mut charges, delta_time);

        }

        clear_potential(&mut potentials_array);
        update_field(&mut test_charges, &charges);
        max_potential = update_potential_and_return_max(&mut potentials_array, &charges);
        voltmeter.update(mouse_position, &charges);
        equipotential_lines_image = transparent_equipotential_lines.clone();
        update_potential_images(&potentials_array, max_potential, &voltmeter.equipotentials, &mut potential_image, &mut equipotential_lines_image);
        draw_potential(&potential_image);
        draw_field(&test_charges);
        draw_equipotential_lines(&equipotential_lines_image);
        draw_charges(&charges);

        voltmeter.draw();
        draw_fps();
        draw_simulation_state(&simulation_state);
        next_frame().await;
    }


}

fn toggle_simulation_state(simulation_state: &mut SimulationState) {
    if *simulation_state == Running {
        *simulation_state = Paused;
    } else {
        *simulation_state = Running;
    }
}

fn update_field(test_charges: &mut Vec<TestCharge>, charges: &Vec<PointCharge>) {
    for test_charge in &mut *test_charges {
        test_charge.clear_forces();
    }
    for test_charge in &mut *test_charges {
        for charge in charges {
            test_charge.force_with(charge);
        }
    }

    for test_charge in &mut *test_charges {

        for charge in charges {
           if test_charge.center.distance_squared(charge.center) < (1.5*(PointCharge::DEFAULT_RADIUS)).powi(2) {
               test_charge.is_hidden = true;
               break;
           }
            test_charge.is_hidden = false;
        }
        if (!test_charge.is_hidden) {
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
            charge1.check_collision_with(charge2);

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
        if charge.sign != Neutral {
        charge.draw_net_force();
            }
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




fn clear_potential(potentials_array: &mut ArrayBase<OwnedRepr<(Vec2, f32)>, Ix2>) {
    potentials_array.par_map_inplace(
        |(_point, potential)| *potential = 0.0f32
    );
}



fn update_potential_and_return_max(potentials_array: &mut ArrayBase<OwnedRepr<(Vec2, f32)>, Ix2>, charges: &Vec<PointCharge>) -> f32 {
    // Process calculations in parallel and modify values in place
    for charge in charges {
        if charge.sign == Neutral { continue}
        potentials_array.par_map_inplace(
            |(point, potential)| *potential += charge.potential_contribution_at(point)
        );

    }
    // Return fixed max value as you're doing
    100.0
}

fn update_potential_images(potentials_array: &ArrayBase<OwnedRepr<(Vec2, f32)>, Ix2>, max_potential: f32, equipotentials: &[f32], potential_image: &mut Image, equipotential_lines_image: &mut Image) {
    // Process all points in parallel and collect updates
    let updates: Vec<(Vec2, Color, bool)> = potentials_array.par_iter()
        .map(|(point, potential)| {
            // Check for equipotential lines first
            let is_equipotential = if (potential.abs() < 10.0) { equipotentials.iter().any(|equip| is_close!(*potential+10.0, *equip+10.0, abs_tol=1e-1)) } else { equipotentials.iter().any(|equip| is_close!(*potential, *equip, rel_tol=1e-2, method=AVERAGE)) };

            // Determine color based on potential or equipotential status
            let color = if is_equipotential {
                GREEN
            } else {
                color_based_on_potential(*potential, max_potential)
            };

            (*point, color, is_equipotential)
        }).collect();
    // equipotential_lines_image.update(TRANSPARENT_COLOR_SLICE);
    // Apply updates to both images sequentially
    for (point, color, is_equipotential) in updates {
        #[allow(clippy::cast_possible_truncation)]
        potential_image.set_pixel(point.x as u32, point.y as u32, color);

        if is_equipotential {
            equipotential_lines_image.set_pixel(point.x as u32, point.y as u32, GREEN);
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

fn draw_simulation_state(simulation_state: &SimulationState) {
    if simulation_state == &Running {
        draw_triangle(RUNNING_SIMULATION_TRIANGLE_VERTICES.0 ,RUNNING_SIMULATION_TRIANGLE_VERTICES.1, RUNNING_SIMULATION_TRIANGLE_VERTICES.2, WHITE);

    } else {
        draw_rectangle(PAUSED_SIMULATION_RECTANGLES.0.x, PAUSED_SIMULATION_RECTANGLES.0.y, PAUSED_SIMULATION_RECTANGLES.0.w, PAUSED_SIMULATION_RECTANGLES.0.h, WHITE);
        draw_rectangle(PAUSED_SIMULATION_RECTANGLES.1.x, PAUSED_SIMULATION_RECTANGLES.1.y, PAUSED_SIMULATION_RECTANGLES.1.w, PAUSED_SIMULATION_RECTANGLES.1.h, WHITE);
    }
}
