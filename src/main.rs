use std::cmp::max;
use std::collections::HashMap;
use std::default::Default;
use itertools::Itertools;
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::iter::Map;
use std::path::Iter;
use std::vec;
use is_close::{is_close, AVERAGE};
use macroquad::math::f32;
use macroquad::miniquad::gl::glPolygonOffset;
use macroquad::prelude::*;
use macroquad::prelude::scene::clear;
use point_charge_simulation::geometry::{ForceArrow, ChargeCircle};
use point_charge_simulation::{charges, Drawable, SplitOneMut};
use point_charge_simulation::charges::{calculate_potential, color_based_on_potential, PointCharge, TestCharge};
use point_charge_simulation::voltmeter::Voltmeter;
use crate::SimulationState::Running;

const MOVEMENT_SPEED: f32 = 200.0;
const WINDOW_WIDTH: u16 = 1600;
const WINDOW_HEIGHT: u16 = 1000;

const ELECTRIC_FIELD_DENSITY: usize = 50;
const POTENTIAL_DENSITY: usize = 10;
const PADDING_FROM_WINDOW_BORDERS: u16 = 50;


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
    let mut charges: Vec<PointCharge> = vec![];
    let delta_time = get_frame_time();
    let mut simulation_state: SimulationState = Running;

    let field_x_points = (PADDING_FROM_WINDOW_BORDERS..=WINDOW_WIDTH - PADDING_FROM_WINDOW_BORDERS).step_by(ELECTRIC_FIELD_DENSITY);
    let field_y_points = (PADDING_FROM_WINDOW_BORDERS..=WINDOW_HEIGHT - PADDING_FROM_WINDOW_BORDERS).step_by(ELECTRIC_FIELD_DENSITY);
    let field_xy_meshgrid = field_x_points.cartesian_product(field_y_points);

    let mut test_charges: Vec<TestCharge> = vec![];
    for (x,y) in field_xy_meshgrid {
      test_charges.push(TestCharge::new(Vec2::new(f32::from(x), f32::from(y))));
    }


    let potential_x_points = (0..=WINDOW_WIDTH).step_by(POTENTIAL_DENSITY);
    let potential_y_points = (0..=WINDOW_HEIGHT).step_by(POTENTIAL_DENSITY);
    let potential_xy_meshgrid = potential_x_points.cartesian_product(potential_y_points);
    let mut potential_map: HashMap<UVec2, f32> = HashMap::new();
    let mut max_potential: f32 = 0.0;

    for (x, y) in potential_xy_meshgrid {
        potential_map.insert(UVec2::new(u32::from(x), u32::from(y)), 0.0);
        // potential_vec2.push(Vec2::new(x as f32, y as f32));
    }

    let mut voltmeter: Voltmeter = Voltmeter::new();




    loop {

        clear_background(BLACK);
        let mouse_position = Vec2 { x: mouse_position().0, y: mouse_position().1 };
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
            update_charges(&mut charges);
            max_potential = update_potential_and_return_max(&mut potential_map, &charges);
        }

        voltmeter.update(mouse_position, &charges);

        draw_potential(&potential_map, max_potential, &voltmeter.equipotentials);
        draw_field(&test_charges);
        draw_charges(&charges);

        voltmeter.draw();

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


fn update_charges(charges: &mut Vec<PointCharge>) {
    for charge in &mut *charges {
        charge.clear_forces();
    }
    for i in 0..charges.len() {
        let (charge1, other_charges) = charges.split_one_mut(i);
        // dbg!(&charge1);
        for charge2 in other_charges {
            charge1.force_with(charge2);
            charge1.check_collision_with(charge2);
        }
    }

    for charge in charges {
        charge.calculate_net_force();
        charge.calculate_max_force();
        charge.calculate_acceleration();
        charge.calculate_velocity();
        charge.movement();
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
    let mut max_potential: f32 = 0.0;
    for (key, value) in potential_map {
        let temp_vec2 = Vec2::new(key.x as f32, key.y as f32);
        let current_potential = calculate_potential(&temp_vec2, charges);
        *value = current_potential;
        max_potential = 200.0;

    }
    max_potential

}
fn draw_potential(potential_map: &HashMap<UVec2, f32>, max_potential:f32, equipotentials: &Vec<f32>) {
   for (point, potential) in potential_map {
       let color = color_based_on_potential(*potential, max_potential);

       draw_circle(point.x as f32, point.y as f32, 6.0, color);
       for equipotential in equipotentials {
           let mut equipotential_points: Vec<UVec2> = vec![];
               if is_close!(*potential, *equipotential, rel_tol=1e-2, method=AVERAGE) {
               draw_circle(point.x as f32, point.y as f32, 6.0, GREEN);
               break;
           }
       }
   }
}


