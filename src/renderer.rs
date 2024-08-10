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
        //hold difference in distance
        let difference_bottom_y = b2 - b1;
        let difference_top_y = t2 - t1;
        let xs = x1;
        let difference_x = one_if_none(x2 - x1);

        let mut horizontal_texture = 0.0;
        let h_step = (1.0 / (x1 - x2)).div_euclid(wall.texture.unwrap().width as f32 * 4.0);

        //clip x
        let mut x1_clipped = Self::clip_width(x1);
        let x2_clipped = Self::clip_width(x2);
        if x1 < 0.0 {
            horizontal_texture -= h_step as f32 * x1_clipped;
            x1_clipped = 0.0;
        }
        //draw x vertical lines
        for x in (x1_clipped as i32)..(x2_clipped as i32) {
            // the y start and end points
            let y1 = difference_bottom_y * (x as f32 + 0.5 - xs) / difference_x as f32 + b1;
            let y2 = difference_top_y * (x as f32 + 0.5 - xs) / difference_x as f32 + t1;

            let mut vertical_texture = 0.0;
            let v_step = (1.0 / (y1 - y2)).div_euclid(wall.texture.unwrap().height as f32 * 4.0);

            //clip y3

            let mut y1_clipped = Self::clip_height(y1);
            let mut y2_clipped = Self::clip_height(y2);
            if y1 < 0.0 {
                vertical_texture -= v_step as f32 * y1_clipped;
                y1_clipped = 0.0;
            }
            //disable draw color to suppress warning since texture is being drawn instead
            //let mut draw_color = color;
            //surface
            if cycle == 0 {
                // on the first pass we collect the points for the surface we want to draw
                if sector.surface == Some(Surface::BottomScan) {
                    sector.surface_points[x as usize] = y1_clipped as u32;
                } // floor points
                if sector.surface == Some(Surface::TopScan) {
                    sector.surface_points[x as usize] = y2_clipped as u32;
                } // ceiling points
                for y in y1_clipped as i32..y2_clipped as i32 {
                    if wall.texture.is_some() {
                        let pixel = (wall.texture.unwrap().height as f32
                            - vertical_texture.rem_euclid(wall.texture.unwrap().height as f32)
                            - 1.0)
                            * wall.texture.unwrap().width as f32
                            - horizontal_texture.rem_euclid(wall.texture.unwrap().width as f32);
                        let pixel_bytes = wall.texture.unwrap().data[pixel as usize].to_le_bytes();
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
            } else if cycle == 1 {
                let x_offset = SCREEN_WIDTH as f32 / 2.0;
                let y_offset = SCREEN_HEIGHT as f32 / 2.0;
                let fov = 700.0;
                let x2 = x - x_offset as i32;
                let wall_offset = 0.0;

                if sector.surface == Some(Surface::BottomScan) {
                    y2_clipped = sector.surface_points[x as usize] as f32;
                    //draw_color = sector.bottom_color;
                }
                if sector.surface == Some(Surface::TopScan) {
                    y1_clipped = sector.surface_points[x as usize] as f32;
                    //draw_color = sector.top_color;
                }

                let move_z = (player.position.z - wall_offset) / y_offset;
                let y_start = y1_clipped - y_offset;
                let y_end = y2_clipped - y_offset;
                for y in y_start as u32..y_end as u32 {
                    let mut z = y as f32;
                    if z as f32 == 0.0 {
                        z = 0.0001;
                    }
                    let fx = x2_clipped / z * move_z;
                    let fy = fov / z * move_z;
                    let rx = fx * sine(player.angle_h) - fy * cosine(player.angle_h)
                        + (player.position.y / 60.0 * 3.0);
                    let ry = fx * cosine(player.angle_h)
                        + fy * sine(player.angle_h)
                        + (player.position.x / 60.0 * 3.0);
                    let pixel = (wall.texture.unwrap().height as f32
                        - (ry % wall.texture.unwrap().height as f32))
                        - 1.0
                            * (wall.texture.unwrap().width as f32
                                - (rx % wall.texture.unwrap().width as f32)
                                - 1.0);
                    let pixel_bytes = wall.texture.unwrap().data[pixel as usize].to_be_bytes();
                    let pixel_color = Color {
                        r: pixel_bytes[3],
                        g: pixel_bytes[2],
                        b: pixel_bytes[1],
                        a: pixel_bytes[0],
                    };
                    self.draw_dot(x2 as f32 + x_offset, y as f32 + y_offset, pixel_color)?;
                }
            }
            // for y in (y1_clipped as i32)..(y2_clipped as i32) {
            //     self.draw_dot(x as f32, y as f32, draw_color)?;
            // }
        }
        Ok(())
    } // Draws a given wall in 3D perspective accounting for player position

    pub fn draw3d(&mut self, player_raw: &mut PlayerInfo) -> Result<(), String> {
        // Master function for the player perspective;
        self.draw_mode = DrawMode::Draw3D;
        let mut player = PlayerInfo::distances(player_raw);

        for s in 0..player.level.number_of_sectors {
            // draws sectors/walls from level.rs in 3D as the player sees it
            let mut sector = player.level.sectors[s as usize];
            sector.distance = 0.0;
            let mut number_of_cycles = 1;
            if player.position.z < sector.bottom_height as f32 {
                sector.surface = Some(Surface::BottomScan); // if the player is below the bottom of the sector we collect the floor points
                number_of_cycles += 1;
                for x in 0..SCREEN_WIDTH {
                    sector.surface_points[x] = SCREEN_HEIGHT as u32;
                } // in the event that one of the walls isnt drawn we fill the missing surface with the bottom color
            } else if player.position.z > sector.top_height as f32 {
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
                    let mut x1 = wall.x1 - player.position.x;
                    let mut y1 = wall.y1 - player.position.y;
                    let mut x2 = wall.x2 - player.position.x;
                    let mut y2 = wall.y2 - player.position.y;

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
                        y1 as f32 * cosine(player.angle_h) + x1 as f32 * sine(player.angle_h) + 0.1;
                    let mut world_y2 =
                        y2 as f32 * cosine(player.angle_h) + x2 as f32 * sine(player.angle_h) + 0.1;
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

                    if world_y1 < 0.0 && world_y2 < 0.0 {
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
                    } else if world_y2 < 0.0 {
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
                    let screen_x1 = world_x1 * 700.0 / world_y1 + HALF_WIDTH as f32;
                    let screen_x2 = world_x2 * 700.0 / world_y2 + HALF_WIDTH as f32;

                    //screen y:
                    let screen_y1 = world_z1 * 700.0 / world_y1 + HALF_HEIGHT as f32;
                    let screen_y2 = world_z2 * 700.0 / world_y2 + HALF_HEIGHT as f32;
                    let screen_y3 = world_z3 * 700.0 / world_y3 + HALF_HEIGHT as f32;
                    let screen_y4 = world_z4 * 700.0 / world_y4 + HALF_HEIGHT as f32;
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
                sector.distance /= (sector.wall_end - sector.wall_start) as f32;
            }
        }
        Ok(())
    }
    // draw3d functions:
    //world -> screen functions:

    //Clipping Functions:
    pub fn clip_width(n: f32) -> f32 {
        if n < 0.0 {
            return 1.0;
        }
        if n > SCREEN_WIDTH as f32 {
            return SCREEN_WIDTH as f32 - 1.0;
        } else {
            return n;
        }
    } // prevents over drawing horizontally based on screen width

    pub fn clip_height(n: f32) -> f32 {
        if n < 0.0 {
            return 1.0;
        }
        if n > SCREEN_HEIGHT as f32 {
            return SCREEN_HEIGHT as f32 - 1.0;
        } else {
            return n;
        }
    } // prevents over drawing vertically based on screen height

    pub fn clip_behind(x1: &mut f32, y1: &mut f32, z1: &mut f32, x2: f32, y2: f32, z2: f32) {
        let da = one_if_none(*y1);
        let db = y2;
        let d = one_if_none(da - db);
        let s = da / d;
        *x1 = *x1 + s * (x2 - (*x1));
        *y1 = one_if_none(*y1 + s * (y2 - (*y1)));
        *z1 = *z1 + s * (z2 - (*z1));
    } //prevents overdrawing behind the player
}

// texture formatter:
pub fn get_texture<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    texture_width: u32,
    texture_height: u32,
    texture_data: &[u32],
) -> Result<sdl2::render::Texture<'a>, String> {
    let mut texture = texture_creator
        .create_texture_streaming(
            Some(PixelFormatEnum::RGBA8888),
            texture_width,
            texture_height,
        )
        .map_err(|e| e.to_string())?;

    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        for index in 0..4096 {
            let bytes = texture_data[index].to_be_bytes();
            buffer[index * 4] = bytes[0];
            buffer[index * 4 + 1] = bytes[1];
            buffer[index * 4 + 2] = bytes[2];
            buffer[index * 4 + 3] = bytes[3];
        }
    })?;

    Ok(texture)
} // Creates a texture from a given array of u32s

pub fn format_texture(path: String, name: String) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| e.to_string())?;

    let mut generated_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("src/generated_textures/{}.rs", name))
        .map_err(|e| e.to_string())?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    for (i, big_chunk) in contents.split("#define").enumerate() {
        if i == 2 {
            let width = big_chunk
                .split(" ")
                .last()
                .expect("Improper format near width!")
                .trim()
                .parse::<u32>()
                .expect("parse error! (width)");
            println!("Width: {:?}", width);

            let width_rs = format!("\npub const {}_WIDTH: u32 = {};\n\n", name, width);
            generated_file
                .write_all(width_rs.as_bytes())
                .expect("write error @ width!");
        }
        if i == 3 {
            let chunk = big_chunk.rsplit("{").last().expect("Improper format!");
            let height = chunk
                .rsplit("*")
                .last()
                .expect("Improper format near height!")
                .matches(char::is_numeric)
                .collect::<String>()
                .parse::<u32>()
                .expect("parse error! (height)");
            println!("Height: {:?}", height);

            let height_rs = format!("pub const {}_HEIGHT: u32 = {:?};\n\n", name, height);
            generated_file
                .write_all(height_rs.as_bytes())
                .expect("write error @ height!");

            let name_and_size = chunk
                .split("*")
                .last()
                .expect("Improper format near texture name!")
                .trim()
                .split("uint32_t")
                .last()
                .expect("Improper format near texture name!");

            let size = name_and_size
                .split_once("]")
                .expect("Improper format near size !")
                .1
                .matches(char::is_numeric)
                .collect::<String>()
                .parse::<u32>()
                .expect("parse error! (height)");
            println!("Size: {:?}", size);

            let data = big_chunk
                .split("{")
                .last()
                .expect("Improper {format near data start!")
                .split(",")
                .collect::<Vec<&str>>();
            println!("data: {:?}", data);

            let data_header_rs = format!("pub const {}_ARRAY:[u32; {:?}] = [\n", name, size);
            generated_file
                .write_all(data_header_rs.as_bytes())
                .expect("write error @ array definiton!");

            for pixel in data {
                let pixel_clean =
                    pixel.trim_matches(|c| c == ' ' || c == '\n' || c == '}' || c == ';');

                let pixel_clean_rs = format!("{},", pixel_clean);
                generated_file
                    .write_all(pixel_clean_rs.as_bytes())
                    .expect("write error in data!");

                println!("pixel: {:?}", pixel_clean);
            }
            generated_file
                .write_all("\n];".as_bytes())
                .expect("write error at last line!");

            let mut mod_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .truncate(false)
                .open(format!("src/generated_textures/mod.rs"))
                .map_err(|e| e.to_string())?;
            mod_file
                .write_all(format!("pub mod {};\n", name).as_bytes())
                .expect("write error in mod file!");
        }
    }
    Ok(())
}
