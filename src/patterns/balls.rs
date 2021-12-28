use std::collections::HashMap;
use std::time::Instant;

use super::Pattern;
use crate::tree::Pixel;

use rand::random;
use macroquad::color::*;
use macroquad::prelude::{Vec3, vec3, Camera3D, set_camera, draw_sphere, next_frame, clear_background};

use rand_distr::{Distribution, Normal};

/// Container for information pertaining to an individual ball.
#[derive(Copy, Clone, Debug)]
pub struct Ball {
    pos: Vec3,
    vel: Vec3,
    color: Color,
    radius: f32,
    // Not strictly necessary
    mass: f32,
}

impl Ball {
    fn new(pos: Vec3, vel: Vec3, color: Color, radius: f32) -> Self {
        Ball {
            pos,
            vel,
            color,
            radius,
            mass: (4. / 3.) * std::f32::consts::PI * radius.powf(3.),
        }
    }
    /// Determines if two balls collide.
    pub fn collides(&self, other: &Self) -> bool {
        self.pos.distance(other.pos) <= (self.radius + other.radius)
    }

    /// Applies perfectly elastic collision physics to both balls
    ///
    /// Math stolen from: https://atmos.illinois.edu/courses/atmos100/userdocs/3Dcollisions.html
    /// Which is actually really bad.
    pub fn collision(&mut self, other: &mut Self) {
        // Step 1: calculate the 3d angle between colliders
        // This uses dot product
        let delta_1 = (other.pos - self.pos).normalize();
        let angle_1 = self.vel.angle_between(delta_1);
        let delta_2 = (self.pos - other.pos).normalize();
        let angle_2 = other.vel.angle_between(delta_2);

        // Step 2: calculate colliders force vectors towards each other
        // (There is an optimization step here, previous angle could skip
        // an acos, and this could skip the cos)
        let v_center_1_i = self.vel.length() * f32::cos(angle_1);
        let v_center_2_i = other.vel.length() * f32::cos(angle_2);

        // Check if they are moving towards or away from each other.
        if v_center_1_i + v_center_2_i <= 0. {
            // They are currently intersecting, but moving away from each other,
            // don't apply collision.
            return;
        }

        // Step 4: determine the force vector normal to the center-line
        // to  recompose new collision
        let normal_1 = self.vel - v_center_1_i * delta_1;
        let normal_2 = other.vel - v_center_2_i * delta_2;

        // Step 5: Switch the collider's force vectors (modified for 1-D differing masses)
        //let v_center_1_f = v_center_2_i;
        //let v_center_2_f = v_center_1_i;
        let mass_sum = self.mass + other.mass;
        let v_center_1_f = (self.mass - other.mass) / mass_sum * v_center_1_i
            + 2. * other.mass / mass_sum * v_center_2_i;
        let v_center_2_f = 2. * self.mass / mass_sum * v_center_1_i
            + (other.mass - self.mass) / mass_sum * v_center_2_i;

        // Step 6: Compose vectors into new velocity
        // THIS IS WRONG
        self.vel = v_center_1_f * delta_2 + normal_1;
        other.vel = v_center_2_f * delta_1 + normal_2;
    }

    /// Update position based on current velocity.
    pub fn update(&mut self) {
        // TODO programmatically set this via frame rate
        self.pos += self.vel / 30.;
    }
}

/// Creates a bunch of balls that float around the bounding box of the tree
/// and collide elastically with each other. (Potentially transferring colour
/// as well.)
pub struct BallPattern {
    /// Clone of tree structure. This never changes so a clone is fine.
    tree: Vec<Pixel>,

    /// List of balls
    balls: Vec<Ball>,

    // Maximum bound of Z height (ceiling)
    zlim_max: f32,
    // Minimum bound of Z height (floor)
    zlim_min: f32,
}

pub fn update_ball_collisions(mut ball: usize, balls: &mut Vec<Ball>) {
    if ball >= balls.len() {
        return;
    }

    // Check for inter-ball collisions
    // TODO surely there must be a nice iterator way to do this
    for i in ball..balls.len() {
        for j in (i + 1)..balls.len() {
            // SAFETY: The correct bounds are being used and I am only
            // accessing them in each iter loop. Also they aren't ever
            // accessing the same element.
            unsafe {
                // TODO actually understand this syntax
                let a = &mut *(balls.get_unchecked_mut(i) as *mut _);
                let b = &mut *(balls.get_unchecked_mut(j) as *mut _);
                if Ball::collides(a, b) {
                    Ball::collision(a, b);
                }
            }
        }
    }

    ball += 1;
    update_ball_collisions(ball, balls);
}

impl Pattern for BallPattern {
    fn from_tree(tree: &Vec<Pixel>, args: &HashMap<String, String>) -> Self {
        // Parameters that will be turned into arguments eventully.
        let num_balls: u8 = match args.get("num") {
            Some(num) => num.as_str().parse().unwrap(),
            None => 6,
        };
        let avg_start_vel: f32 = match args.get("avg_vel") {
            Some(vel) => vel.as_str().parse().unwrap(),
            None => 0.5,
        };
        let radius_mean: f32 = match args.get("rmean") {
            Some(mean) => mean.as_str().parse().unwrap(),
            None => 0.2,
        };
        let radius_deviation: f32 = match args.get("rdev") {
            Some(dev) => dev.as_str().parse().unwrap(),
            None => 0.05,
        };
        // Not sure if I want maximum velocity here
        // This will need to be taken into account in the collision code.
        // Maybe needs to be maximum momentum?
        //let max_vel: f32 = 1.; // (units/s)

        // Radius distribution
        let radius_distr = Normal::new(radius_mean, radius_deviation).unwrap();

        // Find ceiling of tree (idk if I care about the floor, I can add it
        // later if I want)
        let mut zlim_max: f32 = 0.;
        for pixel in tree {
            // NOTE: Assumes no inf/-inf or NaN.
            if pixel.z > zlim_max {
                zlim_max = pixel.z;
            }
        }

        let mut balls = Vec::new();
        for _ in 0..num_balls {
            let x = random::<f32>() * 2.0 - 1.0;
            let y = random::<f32>() * 2.0 - 1.0;
            let z = random::<f32>() * zlim_max;
            let vel_x = (random::<f32>() - 0.5) * avg_start_vel * 2.0;
            let vel_y = (random::<f32>() - 0.5) * avg_start_vel * 2.0;
            let vel_z = (random::<f32>() - 0.5) * avg_start_vel * 2.0;
            let pos = vec3(x, y, z);
            let vel = vec3(vel_x, vel_y, vel_z);

            // Generate random colour
            let r = random::<f32>();
            let g = random::<f32>();
            let b = random::<f32>();
            // Normalize to 50% intensity
            // TODO parameterize this intensity
            let intensity = (r + g + b) / 3.;
            let intensity_ratio = intensity / 0.5;
            let r = (r * intensity_ratio * 255.) as u8;
            let g = (g * intensity_ratio * 255.) as u8;
            let b = (b * intensity_ratio * 255.) as u8;
            let color = Color::from_rgba(r, g, b, 255);

            let radius = radius_distr.sample(&mut rand::thread_rng());
            balls.push(Ball::new(pos, vel, color, radius));
        }

        BallPattern {
            tree: (*tree).clone(),
            balls,
            zlim_max,
            zlim_min: 0.,
        }
    }

    fn next_frame(&mut self) -> Option<Vec<Color>> {
        // "Render" based on initial positions
        let mut frame = Vec::new();
        for pixel in &self.tree {
            let mut color = Color::from_rgba(0, 0, 0, 255);
            for ball in &self.balls {
                if (ball.pos - *pixel).length() <= ball.radius {
                    color = ball.color;
                }
            }

            frame.push(color);
        }

        update_ball_collisions(0, &mut self.balls);

        // Check for wall collisions
        for ball in &mut self.balls {
            if ball.pos.x < -1.0 && ball.vel.x < 0. {
                ball.vel.x = -ball.vel.x;
            } else if ball.pos.x > 1.0 && ball.vel.x > 0. {
                ball.vel.x = -ball.vel.x;
            }

            if ball.pos.y < -1.0 && ball.vel.y < 0. {
                ball.vel.y = -ball.vel.y;
            } else if ball.pos.y > 1.0 && ball.vel.y > 0. {
                ball.vel.y = -ball.vel.y;
            }

            if ball.pos.z < self.zlim_min && ball.vel.z < 0. {
                ball.vel.z = -ball.vel.z;
            } else if ball.pos.z > self.zlim_max && ball.vel.z > 0. {
                ball.vel.z = -ball.vel.z;
            }

            // Update positions while we're here
            ball.update();
        }

        Some(frame)
    }
}

pub async fn run_ball_loop(mut pattern: BallPattern, rpm: u32, fps: u32) {
    // Pre-calculate rotational velocity of scene
    let rot_vel: f32 = std::f32::consts::PI * 2. * (rpm as f32 / 60.);
    // Too lazy to do fixed-point math

    // Prep rotation
    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    loop {
        // Set up basic scene
        clear_background(DARKGRAY);

        let frame_time = Instant::now();
        let delta = frame_time - prev_frame_time;
        prev_frame_time = frame_time;

        // Set up camera
        theta += (delta.as_millis() as f32) / 1000. * rot_vel; // Update camera angle
        set_camera(&Camera3D {
            position: vec3(theta.sin() * 4., theta.cos() * 4., 3.),
            target: vec3(0., 0., 1.5),
            up: vec3(0., 0., 1.),
            ..Default::default()
        });
        // Check for inter-ball collisions
        // TODO surely there must be a nice iterator way to do this
        update_ball_collisions(0, &mut pattern.balls);
        // Check for wall collisions
        for ball in &mut pattern.balls {
            if ball.pos.x < -1.0 && ball.vel.x < 0. {
                ball.vel.x = -ball.vel.x;
            } else if ball.pos.x > 1.0 && ball.vel.x > 0. {
                ball.vel.x = -ball.vel.x;
            }

            if ball.pos.y < -1.0 && ball.vel.y < 0. {
                ball.vel.y = -ball.vel.y;
            } else if ball.pos.y > 1.0 && ball.vel.y > 0. {
                ball.vel.y = -ball.vel.y;
            }

            if ball.pos.z < pattern.zlim_min && ball.vel.z < 0. {
                ball.vel.z = -ball.vel.z;
            } else if ball.pos.z > pattern.zlim_max && ball.vel.z > 0. {
                ball.vel.z = -ball.vel.z;
            }

            // Update positions while we're here
            ball.update();
        }

        for ball in &pattern.balls {
            draw_sphere(ball.pos, ball.radius, None, ball.color);
        }
        next_frame().await;
    }
}
