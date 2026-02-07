use std::io::Write;
use crossterm::{cursor, style::{self, Stylize}, terminal, QueueableCommand};

pub struct Renderer {
    width: usize,
    height: usize,
    camera_x: f64,
    camera_y: f64,
    dir_x: f64,
    dir_y: f64,
    plane_x: f64,
    plane_y: f64,
}

#[derive(Clone)]
pub struct RenderBuffer {
    pub content: Vec<char>,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            camera_x: 5.0,
            camera_y: 5.0,
            dir_x: -1.0,
            dir_y: 0.0,
            plane_x: 0.0,
            plane_y: 0.66,
        }
    }

    pub fn move_camera(&mut self, speed: f64) {
        self.camera_x += self.dir_x * speed;
        self.camera_y += self.dir_y * speed;
    }

    pub fn rotate_camera(&mut self, speed: f64) {
        let old_dir_x = self.dir_x;
        self.dir_x = self.dir_x * f64::cos(speed) - self.dir_y * f64::sin(speed);
        self.dir_y = old_dir_x * f64::sin(speed) + self.dir_y * f64::cos(speed);
        let old_plane_x = self.plane_x;
        self.plane_x = self.plane_x * f64::cos(speed) - self.plane_y * f64::sin(speed);
        self.plane_y = old_plane_x * f64::sin(speed) + self.plane_y * f64::cos(speed);
    }

    pub fn render<W: Write>(&mut self, map: &Vec<Vec<u8>>, writer: &mut W) -> std::io::Result<()> {
        let mut buffer = vec![' '; self.width * self.height];
        
        // Raycasting Loop
        for x in 0..self.width {
            let camera_x = 2.0 * x as f64 / self.width as f64 - 1.0;
            let ray_dir_x = self.dir_x + self.plane_x * camera_x;
            let ray_dir_y = self.dir_y + self.plane_y * camera_x;

            let mut map_x = self.camera_x as i32;
            let mut map_y = self.camera_y as i32;

            let mut side_dist_x;
            let mut side_dist_y;

            let delta_dist_x = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };

            let mut perp_wall_dist;

            let mut step_x;
            let mut step_y;

            let mut hit = 0;
            let mut side = 0;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (self.camera_x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - self.camera_x) * delta_dist_x;
            }

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (self.camera_y - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - self.camera_y) * delta_dist_y;
            }

            // DDA
            while hit == 0 {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                // Boundary check
                if map_x < 0 || map_x >= map.len() as i32 || map_y < 0 || map_y >= map[0].len() as i32 {
                     hit = 1; // Treat out of bounds as wall
                } else {
                     if map[map_x as usize][map_y as usize] > 0 {
                         hit = 1;
                     }
                }
            }

            if side == 0 {
                perp_wall_dist = (map_x as f64 - self.camera_x + (1.0 - step_x as f64) / 2.0) / ray_dir_x;
            } else {
                perp_wall_dist = (map_y as f64 - self.camera_y + (1.0 - step_y as f64) / 2.0) / ray_dir_y;
            }

            let line_height = (self.height as f64 / perp_wall_dist) as i32;
            let mut draw_start = -line_height / 2 + self.height as i32 / 2;
            if draw_start < 0 { draw_start = 0; }
            let mut draw_end = line_height / 2 + self.height as i32 / 2;
            if draw_end >= self.height as i32 { draw_end = self.height as i32 - 1; }

            // Choose wall character based on distance/side
            let char_type = if side == 1 { '#' } else { '%' };
            let wall_char = if perp_wall_dist < 2.0 { '█' }
                           else if perp_wall_dist < 4.0 { '▓' }
                           else if perp_wall_dist < 6.0 { '▒' }
                           else { '░' };

            for y in draw_start..draw_end {
                buffer[y as usize * self.width + x] = wall_char;
            }
        }

        // Draw HUD overlay
        // (Simple text overlay)
        
        // Output to screen
        writer.queue(cursor::MoveTo(0, 0))?;
        for row in buffer.chunks(self.width) {
            let s: String = row.iter().collect();
            writer.queue(style::Print(s))?;
            writer.queue(cursor::MoveToNextLine(1))?;
        }
        writer.flush()?;

        Ok(())
    }
}
