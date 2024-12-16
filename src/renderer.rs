use std::cmp;
use std::f32::EPSILON;

use textures::WATER_GUN_TEXTURE;

use crate::grid::Grid;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
    Draw2D,
    Draw3D,
}

pub struct Renderer {
    pub canvas: Canvas<Window>,
    pub draw_mode: DrawMode,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer {
            canvas,
            draw_mode: DrawMode::Draw3D,
        })
    } // Create a new renderer from nuthin!

    fn draw_background(&mut self) {
        self.canvas.set_draw_color(Color::GRAY);
        self.canvas.clear()
    } // Fills the entire screen with a solid color

    pub fn draw(
        &mut self,
        player: &mut PlayerInfo,
        grid: &mut Grid,
        font: &sdl2::ttf::Font,
    ) -> Result<(), String> {
        self.draw_background();
        match self.draw_mode {
            DrawMode::Draw2D => self.draw2d(player, grid, font)?,
            DrawMode::Draw3D => self.draw3d(player)?,
        };

        self.canvas.present();
        Ok(())
    } // Top level draw function that runs every tick

    pub fn draw_dot(&mut self, x: f32, y: f32, color: Color) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(Rect::new(
            (x * PIXEL_SCALE as f32) as i32,
            (y * PIXEL_SCALE as f32) as i32,
            PIXEL_SCALE as u32,
            PIXEL_SCALE as u32,
        ))?;
        Ok(())
    } // atomic draw function, draws a single pixel (accounting for PIXEL_SCALE(BROKEN DO NOT CHANGE!!!))

    pub fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) -> Result<(), String> {
        let x_float = x2 - x1;
        let y_float = y2 - y1;
        let x = x_float as i32;
        let y = y_float as i32;
        let max = std::cmp::max(x.abs(), y.abs());
        let dx = x_float / one_if_none(max as f32);
        let dy = y_float / one_if_none(max as f32);
        for n in 0..max {
            self.draw_dot(x1 + n as f32 * dx, y1 + n as f32 * dy, color)?;
        }
        Ok(())
    }

    pub fn draw_wall(
        &mut self,
        player: &mut PlayerInfo,
        x1: f32,
        x2: f32,
        b1: f32,
        b2: f32,
        t1: f32,
        t2: f32,
        cycle: u32,
        _color: Color,
        sector: &mut Sector,
        wall: &mut Wall,
    ) -> Result<(), String> {
        // Debug logging
        println!(
            "Drawing wall: x1={}, x2={}, b1={}, b2={}, t1={}, t2={}",
            x1, x2, b1, b2, t1, t2
        );

        // Clipping and texture calculation
        let difference_bottom_y = b2 - b1;
        let difference_top_y = t2 - t1;
        let xs = x1;
        let difference_x = one_if_none(x2 - x1);
        let mut x1_clipped = x1;
        let mut x2_clipped = x2;

        // Horizontal texture
        let mut horizontal_texture = 0.0;
        let h_step = (wall.texture.unwrap().width as f32) * wall.u / (x2 - x1);
        if x1_clipped < 0.0 {
            horizontal_texture -= h_step as f32 * x1;
            x1_clipped = 0.0
        }
        if x2 < 0.0 {
            x2_clipped = 0.0
        }
        if x1 > SCREEN_WIDTH as f32 {
            x1_clipped = SCREEN_WIDTH as f32
        }
        if x2 > SCREEN_WIDTH as f32 {
            x2_clipped = SCREEN_WIDTH as f32
        }

        // Near-plane clipping
        let near_plane = 0.1; // Adjust this value as needed
        let clipped_points = clip_near_plane(x1_clipped, 1.0, b1, x2_clipped, 1.0, b2, near_plane);

        if let Some(((x1_clipped, _, b1), (x2_clipped, _, b2))) = clipped_points {
            // Draw x vertical lines
            for x in (x1_clipped as i32)..(x2_clipped as i32) {
                let y1 = difference_bottom_y * (x as f32 + 0.5 - xs) / difference_x as f32 + b1;
                let y2 = difference_top_y * (x as f32 + 0.5 - xs) / difference_x as f32 + t1;

                // Clip y
                let mut y1_clipped = y1;
                let mut y2_clipped = y2;

                // Vertical texture
                let mut vertical_texture = 0.0;
                let v_step = (wall.texture.unwrap().height as f32) * wall.v / (y2 - y1);
                if y1 < 0.0 {
                    vertical_texture -= v_step as f32 * y1;
                    y1_clipped = 0.0;
                }
                if y2 < 0.0 {
                    y2_clipped = 0.0;
                }
                if y1 > SCREEN_HEIGHT as f32 {
                    y1_clipped = SCREEN_HEIGHT as f32;
                }
                if y2 > SCREEN_HEIGHT as f32 {
                    y2_clipped = SCREEN_HEIGHT as f32;
                }

                // Drawing logic
                for y in y1_clipped as i32..y2_clipped as i32 {
                    if let Some(texture) = wall.texture {
                        let height = texture.height as f32;
                        let width = texture.width as f32;
                        let pixel = (vertical_texture.trunc() % height) * width
                            + (horizontal_texture.trunc() % width);
                        let pixel_bytes = texture.data[pixel as usize].to_le_bytes();
                        let pixel_color = Color {
                            r: pixel_bytes[0],
                            g: pixel_bytes[1],
                            b: pixel_bytes[2],
                            a: pixel_bytes[3],
                        };

                        self.draw_dot(x as f32, y as f32, pixel_color)?;
                        vertical_texture += v_step as f32;
                    }
                }
                horizontal_texture += h_step as f32;
            }
        }
        Ok(())
    } // Draws a given wall in 3D perspective accounting for player position

    pub fn draw_first_person(&mut self, gun_texture: Texture) -> Result<(), String> {
        let mut horizontal_texture: f32 = 0.0;
        let h_step = gun_texture.width as f32 / (0.3 * SCREEN_WIDTH as f32);
        let mut vertical_texture: f32 = 0.0;
        let v_step = (gun_texture.height as f32) / (HALF_HEIGHT) as f32;

        for x in (0.6 * SCREEN_WIDTH as f32) as usize..(0.9 * SCREEN_WIDTH as f32) as usize {
            for y in (HALF_HEIGHT)..SCREEN_HEIGHT {
                let height = gun_texture.height as f32;
                let width = gun_texture.width as f32;
                let pixel = (vertical_texture.trunc() % height) * width
                    + (horizontal_texture.trunc() % width);
                let pixel_bytes = gun_texture.data[pixel as usize].to_le_bytes();
                let pixel_color = Color {
                    r: pixel_bytes[0],
                    g: pixel_bytes[1],
                    b: pixel_bytes[2],
                    a: pixel_bytes[3],
                };
                if pixel_color == Color::RGBA(0, 0, 0, 0) {
                } else if x > (0.7 * SCREEN_WIDTH as f32) as usize
                    && y < (0.6 * SCREEN_HEIGHT as f32) as usize
                {
                } else {
                    self.draw_dot(x as f32, y as f32, pixel_color)?;
                }
                vertical_texture += v_step as f32;
            }
            horizontal_texture += h_step as f32;
        }
        Ok(())
    }

    pub fn draw3d(&mut self, player_raw: &mut PlayerInfo) -> Result<(), String> {
        // Master function for the player perspective;
        self.draw_mode = DrawMode::Draw3D;
        let mut player = PlayerInfo::distances(player_raw);

        for s in 0..player.level.number_of_sectors {
            // draws sectors/walls from level.rs in 3D as the player sees it
            let mut sector = player.level.sectors[s as usize];
            sector.distance = 0.0;
            let mut number_of_cycles = 1;
            if player.position.z < sector.bottom_height {
                sector.surface = Some(Surface::BottomScan); // if the player is below the bottom of the sector we collect the floor points
                number_of_cycles += 1;
                for x in 0..SCREEN_WIDTH {
                    sector.surface_points[x] = SCREEN_HEIGHT as u32;
                } // in the event that one of the walls isnt drawn we fill the missing surface with the bottom color
            } else if player.position.z > sector.top_height {
                sector.surface = Some(Surface::TopScan); // if the player is above the top of the sector we collect the ceiling points
                number_of_cycles += 1;
                for x in 0..SCREEN_WIDTH {
                    sector.surface_points[x] = 0 as u32;
                } // in the event that one of the walls isnt drawn we fill the missing surface with the top color
            } else {
                sector.surface = None;
            } // if the player can't see either surface we don't need to collect any points

            for cycle in 0..number_of_cycles {
                for w in sector.wall_start..sector.wall_end {
                    let mut wall = player.level.walls[w as usize];
                    let color = wall.color;
                    //oftset bottom 2 points by player:
                    let mut x1 = wall.x1 as i32 - player.position.x;
                    let mut y1 = wall.y1 as i32 - player.position.y;
                    let mut x2 = wall.x2 as i32 - player.position.x;
                    let mut y2 = wall.y2 as i32 - player.position.y;

                    if cycle == 1 {
                        let swapx = x1;
                        x1 = x2;
                        x2 = swapx;
                        let swapy = y1;
                        y1 = y2;
                        y2 = swapy;
                    } // on the second pass draw the back sides of the walls and the surfaces we collected points for

                    //world x position:
                    let mut world_x1 =
                        x1 as f32 * cosine(player.angle_h) - y1 as f32 * sine(player.angle_h);
                    let mut world_x2 =
                        x2 as f32 * cosine(player.angle_h) - y2 as f32 * sine(player.angle_h);
                    let mut world_x3 = world_x1;
                    let mut world_x4 = world_x2;

                    //world y position:
                    let mut world_y1 =
                        y1 as f32 * cosine(player.angle_h) + x1 as f32 * sine(player.angle_h);
                    let mut world_y2 =
                        y2 as f32 * cosine(player.angle_h) + x2 as f32 * sine(player.angle_h);
                    let mut world_y3 = world_y1;
                    let mut world_y4 = world_y2;
                    sector.distance += distance(
                        0.0,
                        0.0,
                        (world_x1 + world_x2) / 2.0,
                        (world_y1 + world_y2) / 2.0,
                    );

                    //world z height:
                    let mut world_z1 = sector.bottom_height as f32 - player.position.z as f32;
                    let mut world_z2 = sector.bottom_height as f32 - player.position.z as f32;
                    let mut world_z3 = sector.top_height as f32 - player.position.z as f32;
                    let mut world_z4 = sector.top_height as f32 - player.position.z as f32;

                    if world_y1.trunc() < 0.0 && world_y2.trunc() < 0.0 {
                        continue;
                    } else if world_y1 < 0.0 {
                        Self::clip_behind(
                            &mut world_x1,
                            &mut world_y1,
                            &mut world_z1,
                            world_x2,
                            world_y2,
                            world_z2,
                        );
                        Self::clip_behind(
                            &mut world_x3,
                            &mut world_y3,
                            &mut world_z3,
                            world_x4,
                            world_y4,
                            world_z4,
                        );
                    } else if world_y2.trunc() < 0.0 {
                        Self::clip_behind(
                            &mut world_x2,
                            &mut world_y2,
                            &mut world_z2,
                            world_x1,
                            world_y1,
                            world_z1,
                        );
                        Self::clip_behind(
                            &mut world_x4,
                            &mut world_y4,
                            &mut world_z4,
                            world_x3,
                            world_y3,
                            world_z3,
                        );
                    }
                    //screen x:
                    let fov = 600.0;
                    let epsilon = 0.1; // Small value to prevent division by zero and large terms

                    let screen_x1 = world_x1
                        * (fov / (world_y1.abs().max(epsilon) * world_y1.signum()))
                        + HALF_WIDTH as f32;
                    let screen_x2 = world_x2
                        * (fov / (world_y2.abs().max(epsilon) * world_y2.signum()))
                        + HALF_WIDTH as f32;

                    // screen y:
                    let screen_y1 = world_z1
                        * (fov / (world_y1.abs().max(epsilon) * world_y1.signum()))
                        + HALF_HEIGHT as f32;
                    let screen_y2 = world_z2
                        * (fov / (world_y2.abs().max(epsilon) * world_y2.signum()))
                        + HALF_HEIGHT as f32;
                    let screen_y3 = world_z3
                        * (fov / (world_y3.abs().max(epsilon) * world_y3.signum()))
                        + HALF_HEIGHT as f32;
                    let screen_y4 = world_z4
                        * (fov / (world_y4.abs().max(epsilon) * world_y4.signum()))
                        + HALF_HEIGHT as f32;
                    self.draw_wall(
                        &mut player,
                        screen_x1,
                        screen_x2,
                        screen_y1,
                        screen_y2,
                        screen_y3,
                        screen_y4,
                        cycle,
                        color,
                        &mut sector,
                        &mut wall,
                    )?;
                }
                sector.distance /= (sector.wall_end - sector.wall_start).max(1) as f32;
            }
        }
        self.draw_first_person(WATER_GUN_TEXTURE)?;
        Ok(())
    }
    // draw3d functions:
    //world -> screen functions:

    //Clipping Functions:
    pub fn clip_width(n: f32) -> f32 {
        if n < f32::EPSILON {
            return f32::EPSILON;
        }
        if n > SCREEN_WIDTH as f32 {
            return SCREEN_WIDTH as f32;
        } else {
            return n;
        }
    } // prevents over drawing horizontally based on screen width

    pub fn clip_height(n: f32) -> f32 {
        if n < f32::EPSILON {
            return f32::EPSILON;
        }
        if n > SCREEN_HEIGHT as f32 {
            return SCREEN_HEIGHT as f32;
        } else {
            return n;
        }
    } // prevents over drawing vertically based on screen height

    pub fn clip_behind(x1: &mut f32, y1: &mut f32, z1: &mut f32, x2: f32, y2: f32, z2: f32) {
        let da = *y1;
        let db = y2;
        let d = (da - db).max(f32::EPSILON);
        let s = (da / d).max(f32::EPSILON);
        *x1 = *x1 + s * (x2 - (*x1));
        *y1 = (*y1 + s * (y2 - (*y1))).max(f32::EPSILON);
        *z1 = *z1 + s * (z2 - (*z1));
    } //prevents overdrawing behind the player
}
