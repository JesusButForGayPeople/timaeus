use crate::grid::Grid;
use crate::*;

pub struct Renderer {
    pub canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
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
        //self.draw2d(player, grid, font)?;
        self.draw3d(player)?;
        self.canvas.present();
        Ok(())
    } // Top level draw function that runs every tick

    pub fn draw_dot(&mut self, x: f32, y: f32, color: Color) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(Rect::new(
            (x * PIXEL_SCALE as f32) as i32,
            (y * PIXEL_SCALE as f32) as i32,
            2 * PIXEL_SCALE as u32,
            2 * PIXEL_SCALE as u32,
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
        x1: f32,
        x2: f32,
        b1: f32,
        b2: f32,
        t1: f32,
        t2: f32,
        color: Color,
        sector: &mut Sector,
    ) -> Result<(), String> {
        //hold difference in distance
        let difference_bottom_y = b2 - b1;
        let difference_top_y = t2 - t1;
        let difference_x = one_if_none(x2 - x1);
        //clip x
        let x1_clipped = Self::clip_width(x1);
        let x2_clipped = Self::clip_width(x2);
        //draw x vertical lines
        for x in (x1_clipped as i32)..(x2_clipped as i32) {
            // the y start and end points
            let y1 = difference_bottom_y * (x as f32 - x1 + 0.5) / difference_x as f32 + b1;
            let y2 = difference_top_y * (x as f32 - x1 + 0.5) / difference_x as f32 + t1;
            //clip y
            let y1_clipped = Self::clip_height(y1);
            let y2_clipped = Self::clip_height(y2);

            //surface

            if sector.surface == Surface::TopScan {
                sector.surface_points[x as usize] = y2_clipped as u32;
            }
            if sector.surface == Surface::TopDraw {
                for y in y2_clipped as u32..sector.surface_points[x as usize] {
                    self.draw_dot(x as f32, y as f32, sector.top_color)?;
                }
            }
            if sector.surface == Surface::BottomScan {
                sector.surface_points[x as usize] = y1_clipped as u32;
            }
            if sector.surface == Surface::BottomDraw {
                for y in sector.surface_points[x as usize]..y1_clipped as u32 {
                    self.draw_dot(x as f32, y as f32, sector.bottom_color)?;
                }
            }
            if sector.surface == Surface::None {}
            for y in (y1_clipped as i32)..(y2_clipped as i32) {
                self.draw_dot(x as f32, y as f32, color)?;
            }
        }
        Ok(())
    } // Draws a given wall in 3D perspective accounting for player position

    pub fn draw3d(&mut self, player_raw: &mut PlayerInfo) -> Result<(), String> {
        // Master function for the player perspective;
        let player = PlayerInfo::distances(player_raw);
        for sector in &mut player.level.sectors {
            // draws sectors/walls from level.rs in 3D as the player sees it

            if player.position.z < sector.bottom_height as f32 {
                sector.surface = Surface::BottomScan;
                for x in 0..SCREEN_WIDTH {
                    sector.surface_points[x] = SCREEN_HEIGHT as u32;
                }
            } else if player.position.z > sector.top_height as f32 {
                sector.surface = Surface::TopScan;
                for x in 0..SCREEN_WIDTH {
                    sector.surface_points[x] = 0 as u32;
                }
            } else {
                sector.surface = Surface::None
            } // Determine if the player can see the top/bottom of the sector

            for mut j in 0..2 {
                if j == 0 {
                    for (i, wall) in player.level.walls.iter().enumerate() {
                        if sector.surface == Surface::TopScan {
                            sector.surface = Surface::TopDraw
                        }
                        if sector.surface == Surface::BottomScan {
                            sector.surface = Surface::BottomDraw
                        }
                        if sector.wall_start as usize <= i && i < sector.wall_end as usize {
                            let color = wall.color;
                            //oftset bottom 2 points by player:
                            let x1 = wall.x1 - player.position.x;
                            let y1 = wall.y1 - player.position.y;
                            let x2 = wall.x2 - player.position.x;
                            let y2 = wall.y2 - player.position.y;

                            //world x position:
                            let world_x1 = x1 as f32 * cosine(player.angle_h)
                                - y1 as f32 * sine(player.angle_h);
                            let world_x2 = x2 as f32 * cosine(player.angle_h)
                                - y2 as f32 * sine(player.angle_h);
                            let world_x3 = world_x1;
                            let world_x4 = world_x2;

                            //world y position:
                            let world_y1 = y1 as f32 * cosine(player.angle_h)
                                + x1 as f32 * sine(player.angle_h);
                            let world_y2 = y2 as f32 * cosine(player.angle_h)
                                + x2 as f32 * sine(player.angle_h);
                            let world_y3 = world_y1;
                            let world_y4 = world_y2;

                            //world z height:
                            let world_z1 = sector.bottom_height as f32 - player.position.z as f32;
                            let world_z2 = sector.bottom_height as f32 - player.position.z as f32;
                            let world_z3 = sector.top_height as f32 - player.position.z as f32;
                            let world_z4 = sector.top_height as f32 - player.position.z as f32;

                            if world_y1 < 1.0 && world_y2 < 1.0 {
                                continue;
                            }

                            if world_y1 < 1.0 {
                                let XYZ {
                                    x: x1,
                                    y: y1,
                                    z: z1,
                                } = Self::clip_behind(
                                    world_x1, world_y1, world_z1, world_x2, world_y2, world_z2,
                                );
                                let XYZ {
                                    x: x2,
                                    y: y2,
                                    z: z2,
                                } = Self::clip_behind(
                                    world_x3,
                                    world_y3,
                                    world_z3 as f32,
                                    world_x4,
                                    world_y4,
                                    world_z4 as f32,
                                );
                                //screen x:
                                let screen_x1 = Self::screen_x(x1 as f32, y1 as f32);
                                let screen_x2 = Self::screen_x(x2 as f32, y2 as f32);

                                //screen y:
                                let screen_y1 = Self::screen_y(z1 as f32, y1 as f32);
                                let screen_y2 = Self::screen_y(z2 as f32, y1 as f32);
                                let screen_y3 = Self::screen_y(world_z3 as f32, world_y3 as f32);
                                let screen_y4 = Self::screen_y(world_z4 as f32, world_y4 as f32);

                                self.draw_wall(
                                    screen_x1, screen_x2, screen_y1, screen_y2, screen_y3,
                                    screen_y4, color, sector,
                                )?;
                            } else if world_y2 < 1.0 {
                                let XYZ {
                                    x: x1,
                                    y: y1,
                                    z: z1,
                                } = Self::clip_behind(
                                    world_x2, world_y2, world_z2, world_x1, world_y1, world_z1,
                                );
                                let XYZ {
                                    x: x2,
                                    y: y2,
                                    z: z2,
                                } = Self::clip_behind(
                                    world_x4,
                                    world_y4,
                                    world_z4 as f32,
                                    world_x3,
                                    world_y3,
                                    world_z3 as f32,
                                );
                                //screen x:
                                let screen_x1 = Self::screen_x(x1 as f32, y1 as f32);
                                let screen_x2 = Self::screen_x(x2 as f32, y2 as f32);

                                //screen y:
                                let screen_y1 = Self::screen_y(z1 as f32, y1 as f32);
                                let screen_y2 = Self::screen_y(z2 as f32, y1 as f32);
                                let screen_y3 = Self::screen_y(world_z3 as f32, world_y3 as f32);
                                let screen_y4 = Self::screen_y(world_z4 as f32, world_y4 as f32);

                                self.draw_wall(
                                    screen_x1 as f32,
                                    screen_x2 as f32,
                                    screen_y1 as f32,
                                    screen_y2 as f32,
                                    screen_y3 as f32,
                                    screen_y4 as f32,
                                    color,
                                    sector,
                                )?;
                            } else {
                                //screen x:
                                let screen_x1 = world_x1 * 700.0 / world_y1 + HALF_WIDTH as f32;
                                let screen_x2 = world_x2 * 700.0 / world_y2 + HALF_WIDTH as f32;

                                //screen y:
                                let screen_y1 = world_z1 * 700.0 / world_y1 + HALF_HEIGHT as f32;
                                let screen_y2 = world_z2 * 700.0 / world_y2 + HALF_HEIGHT as f32;
                                let screen_y3 =
                                    world_z3 as f32 * 700.0 / world_y3 + HALF_HEIGHT as f32;
                                let screen_y4 =
                                    world_z4 as f32 * 700.0 / world_y4 + HALF_HEIGHT as f32;

                                self.draw_wall(
                                    screen_x1, screen_x2, screen_y1, screen_y2, screen_y3,
                                    screen_y4, color, sector,
                                )?;
                            }
                        }
                    }
                }
                //the j ==  0 arm draws the backwalls by switching the x1 and x2 coordinates and assigns the points for the surfaces.
                //the j ==  1 arm assigns x1 and x2 properly to draw the outer face of the wall and draws the surfaces.
                //other than switching the initial x coordinates and handling the surfaces differently the two arms are identical
                if j == 1 {
                    let start = sector.wall_start as usize;
                    let end = sector.wall_end as usize;
                    for (i, wall) in player.level.walls.iter().enumerate() {
                        if sector.surface != Surface::None {
                            if start <= i && i < end {
                                let color = wall.color;
                                //oftset bottom 2 points by player:
                                let x1 = wall.x2 - player.position.x;
                                let y1 = wall.y2 - player.position.y;
                                let x2 = wall.x1 - player.position.x;
                                let y2 = wall.y1 - player.position.y;

                                let world_x1 = x1 as f32 * cosine(player.angle_h)
                                    - y1 as f32 * sine(player.angle_h);
                                let world_x2 = x2 as f32 * cosine(player.angle_h)
                                    - y2 as f32 * sine(player.angle_h);
                                let world_x3 = world_x1;
                                let world_x4 = world_x2;

                                //world y position:
                                let world_y1 = y1 as f32 * cosine(player.angle_h)
                                    + x1 as f32 * sine(player.angle_h);
                                let world_y2 = y2 as f32 * cosine(player.angle_h)
                                    + x2 as f32 * sine(player.angle_h);
                                let world_y3 = world_y1;
                                let world_y4 = world_y2;

                                //world z height:
                                let world_z1 =
                                    sector.bottom_height as f32 - player.position.z as f32;
                                let world_z2 =
                                    sector.bottom_height as f32 - player.position.z as f32;
                                let world_z3 = world_z1 + sector.top_height as f32;
                                let world_z4 = world_z2 + sector.top_height as f32;

                                if world_y1 < 1.0 && world_y2 < 1.0 {
                                    continue;
                                }

                                if world_y1 < 1.0 {
                                    let XYZ {
                                        x: x1,
                                        y: y1,
                                        z: z1,
                                    } = Self::clip_behind(
                                        world_x1, world_y1, world_z1, world_x2, world_y2, world_z2,
                                    );
                                    let XYZ {
                                        x: x2,
                                        y: y2,
                                        z: z2,
                                    } = Self::clip_behind(
                                        world_x3,
                                        world_y3,
                                        world_z3 as f32,
                                        world_x4,
                                        world_y4,
                                        world_z4 as f32,
                                    );
                                    //screen x:
                                    let screen_x1 = Self::screen_x(x1 as f32, y1 as f32);
                                    let screen_x2 = Self::screen_x(x2 as f32, y2 as f32);

                                    //screen y:
                                    let screen_y1 = Self::screen_y(z1 as f32, y1 as f32);
                                    let screen_y2 = Self::screen_y(z2 as f32, y1 as f32);
                                    let screen_y3 =
                                        Self::screen_y(world_z3 as f32, world_y3 as f32);
                                    let screen_y4 =
                                        Self::screen_y(world_z4 as f32, world_y4 as f32);

                                    self.draw_wall(
                                        screen_x1, screen_x2, screen_y1, screen_y2, screen_y3,
                                        screen_y4, color, sector,
                                    )?;
                                } else if world_y2 < 1.0 {
                                    let XYZ {
                                        x: x1,
                                        y: y1,
                                        z: z1,
                                    } = Self::clip_behind(
                                        world_x2, world_y2, world_z2, world_x1, world_y1, world_z1,
                                    );
                                    let XYZ {
                                        x: x2,
                                        y: y2,
                                        z: z2,
                                    } = Self::clip_behind(
                                        world_x4,
                                        world_y4,
                                        world_z4 as f32,
                                        world_x3,
                                        world_y3,
                                        world_z3 as f32,
                                    );
                                    //screen x:
                                    let screen_x1 = Self::screen_x(x1 as f32, y1 as f32);
                                    let screen_x2 = Self::screen_x(x2 as f32, y2 as f32);

                                    //screen y:
                                    let screen_y1 = Self::screen_y(z1 as f32, y1 as f32);
                                    let screen_y2 = Self::screen_y(z2 as f32, y1 as f32);
                                    let screen_y3 =
                                        Self::screen_y(world_z3 as f32, world_y3 as f32);
                                    let screen_y4 =
                                        Self::screen_y(world_z4 as f32, world_y4 as f32);

                                    self.draw_wall(
                                        screen_x1 as f32,
                                        screen_x2 as f32,
                                        screen_y1 as f32,
                                        screen_y2 as f32,
                                        screen_y3 as f32,
                                        screen_y4 as f32,
                                        color,
                                        sector,
                                    )?;
                                } else {
                                    //screen x:
                                    let screen_x1 = world_x1 * 700.0 / world_y1 + HALF_WIDTH as f32;
                                    let screen_x2 = world_x2 * 700.0 / world_y2 + HALF_WIDTH as f32;

                                    //screen y:
                                    let screen_y1 =
                                        world_z1 * 700.0 / world_y1 + HALF_HEIGHT as f32;
                                    let screen_y2 =
                                        world_z2 * 700.0 / world_y2 + HALF_HEIGHT as f32;
                                    let screen_y3 =
                                        world_z3 as f32 * 700.0 / world_y3 + HALF_HEIGHT as f32;
                                    let screen_y4 =
                                        world_z4 as f32 * 700.0 / world_y4 + HALF_HEIGHT as f32;

                                    self.draw_wall(
                                        screen_x1, screen_x2, screen_y1, screen_y2, screen_y3,
                                        screen_y4, color, sector,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    // draw3d functions:
    //world -> screen functions:
    pub fn screen_x(x: f32, y: f32) -> f32 {
        x * 600.0 / one_if_none(y + HALF_WIDTH as f32)
    } // calculates the screen x position of a pixel given the world x and y

    pub fn screen_y(z: f32, y: f32) -> f32 {
        z * 600.0 / one_if_none(y + HALF_HEIGHT as f32)
    } // calculates the screen y position of a pixel given the world x and y

    //Clipping Functions:
    pub fn clip_width(n: f32) -> f32 {
        if n < 1.0 {
            return 1.0;
        }
        if n > SCREEN_WIDTH as f32 {
            return SCREEN_WIDTH as f32;
        } else {
            return n;
        }
    } // prevents over drawing horizontally based on screen width

    pub fn clip_height(n: f32) -> f32 {
        if n < 1.0 {
            return 1.0;
        }
        if n > SCREEN_HEIGHT as f32 {
            return SCREEN_HEIGHT as f32;
        } else {
            return n;
        }
    } // prevents over drawing vertically based on screen height

    pub fn clip_behind(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> XYZ {
        let dy = y1 - y2;
        let d = one_if_none(dy);
        let y = one_if_none(y1);
        let s = y / one_if_none(d);
        let x_clipped = x1 + s * (x2 - (x1));
        let y_clipped = y1 + s * (y2 - (y1));
        let z_clipped = z1 + s * (z2 - (z1));
        XYZ {
            x: x_clipped,
            y: y_clipped,
            z: z_clipped,
        }
    } //prevents overdrawing behind the player
}