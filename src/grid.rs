pub use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub mouse_status: MouseStatus,      // what the mouse is currently doing
    pub wall_color: Color,              // the color of the wall that is curently being drawn
    pub bottom_height: i32, // the bottom height of the sector that is currently being drawn
    pub top_height: i32,    // the top height of the sector that is currently being drawn
    pub scale: i32,         // how large the squares of the grid appear on the screen
    pub selected_sector: Option<usize>, // the sector that is currently being moved
    pub selected_wall: Option<usize>, // the wall that is currently being moved
    pub selected_point: Option<usize>, // the point that is currently being moved
    pub view_shift_x: i32,  // offset of the grid from the map in the x direction
    pub view_shift_y: i32,  // offset of the grid from the map in the y direction
    pub highlight_x: Option<i32>, // the initial  x position of the currently drawn highlight box
    pub highlight_y: Option<i32>, // the initial  y position of the currently drawn highlight box
    pub selection: Selection, // the points, walls, & vectors that are in a highlight area
    pub state: State,
    pub new_sector: Option<Vec<(i32, i32)>>,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            mouse_status: MouseStatus {
                mouse_x: 0,
                mouse_y: 0,

                click_count: 0,
                button: None,
                relative_x: None,
                relative_y: None,
            },

            wall_color: colors::GREEN,
            bottom_height: 0,
            top_height: 40,
            scale: 10,
            selected_sector: None,
            selected_wall: None,
            selected_point: None,
            view_shift_x: 0,
            view_shift_y: 0,
            highlight_x: None,
            highlight_y: None,
            selection: Selection {
                sectors: Vec::new(),
                walls: Vec::new(),
                points: Vec::new(),
            },
            state: State::Free,
            new_sector: None,
        }
    }

    pub fn get_mouse_status(&mut self, mouse_state: MouseState) {
        self.mouse_status = MouseStatus::get(mouse_state, self.mouse_status.click_count)
    } // gets the mouse.state from the SDL event pump

    pub fn deselect(&mut self) {
        //self.selected_wall = None;
        //self.selected_sector = None;
        self.selected_point = None;
    } // deselects all points, walls, & sectors; called every frame that the left mouse button is not pressed

    pub fn view_down(&mut self) {
        self.view_shift_y -= 10
    }
    pub fn view_up(&mut self) {
        self.view_shift_y += 10
    }
    pub fn view_left(&mut self) {
        self.view_shift_x += 10
    }
    pub fn view_right(&mut self) {
        self.view_shift_x -= 10
    }

    pub fn next_wall(&mut self, player: &mut PlayerInfo) {
        match self.selected_wall {
            Some(wall) => {
                if wall >= player.level.number_of_walls as usize - 1 {
                    self.selected_wall = Some(0);
                } else {
                    self.selected_wall = Some(wall + 1);
                }
            }
            _ => self.selected_wall = Some(0),
        }
    }

    pub fn new_sector(&mut self, player: &mut PlayerInfo) {
        let random_color_number = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            % 9;
        let new_color = match random_color_number {
            0 => colors::RED,
            1 => colors::GREEN,
            2 => colors::BLUE,
            3 => colors::YELLOW,
            4 => colors::CYAN,
            5 => colors::PURPLE,
            6 => colors::BROWN,
            7 => colors::PINK,
            8 => colors::ORANGE,
            _ => colors::BLACK,
        };

        let new_sector_walls: [Wall; 4] = [
            Wall {
                x1: 32.0,
                y1: 32.0,
                x2: 32.0,
                y2: 64.0,
                color: new_color,
                texture: Some(textures::BRAT_TEXTURE),
                u: 1.0,
                v: 1.0,
            },
            Wall {
                x1: 64.0,
                y1: 32.0,
                x2: 32.0,
                y2: 32.0,
                color: new_color,
                texture: Some(textures::BRAT_TEXTURE),
                u: 1.0,
                v: 1.0,
            },
            Wall {
                x1: 64.0,
                y1: 64.0,
                x2: 64.0,
                y2: 32.0,
                color: new_color,
                texture: Some(textures::BRAT_TEXTURE),
                u: 1.0,
                v: 1.0,
            },
            Wall {
                x1: 32.0,
                y1: 64.0,
                x2: 64.0,
                y2: 64.0,
                color: new_color,
                texture: Some(textures::BRAT_TEXTURE),
                u: 1.0,
                v: 1.0,
            },
        ];
        player.level.walls.append(&mut new_sector_walls.to_vec());
        player.level.number_of_walls += 4;
        player.level.sectors.push(Sector {
            wall_start: player.level.number_of_walls as i32 - 4,
            wall_end: player.level.number_of_walls as i32,
            bottom_height: 0,
            top_height: 40,
            distance: 0.0,
            top_color: colors::WHITE,
            bottom_color: colors::BLACK,
            surface_points: [0; SCREEN_WIDTH],
            surface: None,
            surface_texture: None,
        });
        player.level.number_of_sectors += 1;
    } // creates a new cyan sector in the center of the grid
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub sectors: Vec<usize>,
    pub walls: Vec<usize>,
    pub points: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Busy,
    Free,
}

impl Selection {
    pub fn from_level(level: &Level) -> Selection {
        let sectors = Vec::<usize>::with_capacity(level.number_of_sectors as usize);
        let walls = Vec::<usize>::with_capacity(level.number_of_walls as usize);
        let points = Vec::<usize>::with_capacity(2 * level.number_of_walls as usize);
        Selection {
            sectors,
            walls,
            points,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Button {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseStatus {
    pub mouse_x: i32,
    pub mouse_y: i32,

    pub click_count: usize,
    pub button: Option<Button>,
    pub relative_x: Option<i32>,
    pub relative_y: Option<i32>,
}

impl MouseStatus {
    pub fn get(mouse_state: MouseState, click_count: usize) -> MouseStatus {
        let mouse_x = mouse_state.x();
        let mouse_y = mouse_state.y();
        let button = match mouse_state.left() {
            true => Some(Button::Left),
            false => match mouse_state.right() {
                true => Some(Button::Right),
                false => None,
            },
        };

        MouseStatus {
            mouse_x,
            mouse_y,

            click_count,
            button,
            relative_x: None,
            relative_y: None,
        }
    }
}

pub fn save(player: &mut PlayerInfo) -> () {
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .truncate(true)
        .create(true)
        .open("src/level.rs")
        .expect("Failed to read level.rs loser!");

    let header = format!(
        "pub use sdl2::pixels::Color;\npub use crate::{{colors, textures, Sector, Surface, Wall}};\npub const NUM_SECTORS: usize = {:?}; \npub const NUM_WALLS: usize = {:?};\n\n//SECTORS:\npub const INIT_SECTORS: [Sector; NUM_SECTORS] = [",
        player.level.number_of_sectors, player.level.number_of_walls
    );
    file.write_all(header.as_bytes())
        .expect("Unable to write your data loser!");

    for s in 0..player.level.number_of_sectors {
        let sector = format!(
            "Sector{{\n wall_start:{:?},\n wall_end:{:?},\n bottom_height:{:?},\n top_height:{:?},\n distance:{:?},\n top_color:Color::RGBA{:?},\n bottom_color:Color::RGBA{:?},\n surface:{:?},\n surface_points:[0; crate::SCREEN_WIDTH],\n surface_texture:{}\n}},\n\n",
            player.level.sectors[s as usize].wall_start,
            player.level.sectors[s as usize].wall_end,
            player.level.sectors[s as usize].bottom_height,
            player.level.sectors[s as usize].top_height,
            player.level.sectors[s as usize].distance,
            player.level.sectors[s as usize].top_color.rgba(),
            player.level.sectors[s as usize].bottom_color.rgba(),
            player.level.sectors[s as usize].surface,
            //player.level.sectors[s as usize].surface_points,
            match player.level.sectors[s as usize].surface_texture {
                Some(texture) => format!("Some(textures::{})", texture.name),
                _ => "None".to_string(),
            },

        );
        file.write_all(sector.as_bytes())
            .expect("Unable to write your data loser!");
    }
    file.write_all(
        format!(
            "];\n\n//WALLS:\n\n pub const INIT_WALLS:[Wall; {:?}] = [",
            player.level.number_of_walls
        )
        .as_bytes(),
    )
    .expect("Unable to write your data loser!");

    for w in 0..player.level.number_of_walls {
        let x1 = player.level.walls[w as usize].x1;
        let y1 = player.level.walls[w as usize].y1;
        let x2 = player.level.walls[w as usize].x2;
        let y2 = player.level.walls[w as usize].y2;
        let wall = format!(
            "Wall{{\n x1:{:?},\n y1:{:?},\n x2:{:?},\n y2:{:?},\n color:Color::RGBA{:?},\n texture:Some(textures::{}),\n u:{:?},\n v:{:?}}},\n\n",
            x1,
            y1,
            x2,
            y2,
            player.level.walls[w as usize].color.rgba(),
            player.level.walls[w as usize].texture.unwrap().name,
            player.level.walls[w as usize].u,
            player.level.walls[w as usize].v,
        );
        file.write_all(wall.as_bytes())
            .expect("Unable to write your data loser!")
    }
    file.write_all("\n];\n".as_bytes())
        .expect("Unable to write your data loser!");

    println!("Level Saved ~<3")
} // writes the current level to level.rs, deleting whatever data is currently there
  // !!!CAUTION!!! NOT REVERSIBLE !!!

impl renderer::Renderer {
    pub fn draw_big_dot(&mut self, x: f32, y: f32, color: Color) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(Rect::new(
            (x * PIXEL_SCALE as f32) as i32,
            (y * PIXEL_SCALE as f32) as i32,
            5 * PIXEL_SCALE as u32,
            5 * PIXEL_SCALE as u32,
        ))?;
        Ok(())
    } // draws a dot that is larger than draw_dot

    pub fn draw_thick_line(
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
            self.draw_big_dot(x1 + n as f32 * dx, y1 + n as f32 * dy, color)?;
        }
        Ok(())
    } // draws a line using big_dot

    pub fn draw_circle(&mut self, x: f32, y: f32, radius: u32, color: Color) -> Result<(), String> {
        for t in 0..360 {
            let x1 = x + (radius as f32 * cosine(t));
            let y1 = y + (radius as f32 * sine(t));
            self.draw_dot(x1, y1, color)?;
        }
        Ok(())
    } // draws a circle using draw_dot

    pub fn draw_player(
        &mut self,
        x: i32,
        y: i32,
        color: Color,
        grid: &mut Grid,
        player: &mut PlayerInfo,
    ) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(Rect::new(
            x * PIXEL_SCALE as i32,
            y * PIXEL_SCALE as i32,
            grid.scale as u32 * 2 * PIXEL_SCALE as u32,
            grid.scale as u32 * 2 * PIXEL_SCALE as u32,
        ))?;

        for t in player.angle_h as i32 - 22..player.angle_h as i32 + 22 {
            let x1 = x as f32 + (70.0 * sine(t));
            let y1 = y as f32 + (70.0 * cosine(t));
            self.draw_dot(x1, y1, color)?;
        }
        Ok(())
    } // Draws the player as a square and draws an arc to indicate the direction they are facing

    pub fn text(
        render: &mut renderer::Renderer,
        texture_creator: &TextureCreator<WindowContext>,
        font: &sdl2::ttf::Font,
        string: String,
        color: Color,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let surface = font
            .render(&string)
            .blended(color)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let text_box = Rect::new(x, y, width, height);
        render.canvas.copy(&texture, None, text_box)?;
        Ok(())
    } // creates all the neccessary components to render text

    pub fn draw_new_sector(
        &mut self,
        grid: &mut Grid,
        player: &mut PlayerInfo,
    ) -> Result<(), String> {
        let random_color_number = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            % 9;
        let new_color = match random_color_number {
            0 => colors::RED,
            1 => colors::GREEN,
            2 => colors::BLUE,
            3 => colors::YELLOW,
            4 => colors::CYAN,
            5 => colors::PURPLE,
            6 => colors::BROWN,
            7 => colors::PINK,
            8 => colors::ORANGE,
            _ => colors::BLACK,
        };
        match &mut grid.new_sector {
            Some(points) => match grid.mouse_status.click_count {
                0 => {
                    self.draw_dot(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        colors::WHITE,
                    )?;
                    if grid.mouse_status.button == Some(Button::Left) {
                        points.push((
                            grid.mouse_status.mouse_x as i32,
                            grid.mouse_status.mouse_y as i32,
                        ));
                        grid.mouse_status.click_count += 1;
                    }
                }
                1 => {
                    self.draw_thick_line(
                        points[0].0 as f32,
                        points[0].1 as f32,
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        colors::WHITE,
                    )?;
                    if grid.mouse_status.button == Some(Button::Left) {
                        points.push((
                            grid.mouse_status.mouse_x as i32,
                            grid.mouse_status.mouse_y as i32,
                        ));
                        grid.mouse_status.click_count += 1;
                    }
                }
                2 => {
                    self.draw_thick_line(
                        points[0].0 as f32,
                        points[0].1 as f32,
                        points[1].0 as f32,
                        points[1].1 as f32,
                        colors::GREY3,
                    )?;
                    self.draw_thick_line(
                        points[1].0 as f32,
                        points[1].1 as f32,
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        colors::WHITE,
                    )?;
                    if grid.mouse_status.button == Some(Button::Left) {
                        points.push((
                            grid.mouse_status.mouse_x as i32,
                            grid.mouse_status.mouse_y as i32,
                        ));
                        grid.mouse_status.click_count += 1;
                    }
                }
                _ => {
                    for (i, (x, y)) in points.iter().enumerate() {
                        if i == points.len() - 1 {
                            self.draw_thick_line(
                                *x as f32,
                                *y as f32,
                                grid.mouse_status.mouse_x as f32,
                                grid.mouse_status.mouse_y as f32,
                                colors::WHITE,
                            )?;
                        } else {
                            self.draw_thick_line(
                                *x as f32,
                                *y as f32,
                                points[i + 1].0 as f32,
                                points[i + 1].1 as f32,
                                colors::GREY3,
                            )?;
                        }
                    }
                    if grid.mouse_status.button == Some(Button::Left) {
                        points.push((
                            grid.mouse_status.mouse_x as i32,
                            grid.mouse_status.mouse_y as i32,
                        ));
                        grid.mouse_status.click_count += 1;
                    }
                    let distance = distance(
                        points.first().unwrap().0 as f32,
                        points.first().unwrap().1 as f32,
                        points.last().unwrap().0 as f32,
                        points.last().unwrap().1 as f32,
                    );

                    if points.len() >= 3 && distance <= 8.0 {
                        if grid.mouse_status.button == Some(Button::Left) {
                            points.push((
                                grid.mouse_status.mouse_x as i32,
                                grid.mouse_status.mouse_y as i32,
                            ));
                            grid.mouse_status.click_count += 1;
                        }

                        player.level.sectors.push(Sector {
                            wall_start: player.level.number_of_walls as i32,
                            wall_end: (player.level.number_of_walls + points.len() as u32 - 2)
                                as i32,
                            bottom_height: 0,
                            top_height: 40,
                            top_color: colors::BLACK,
                            bottom_color: colors::WHITE,
                            distance: 0.0,
                            surface: None,
                            surface_points: [0; SCREEN_WIDTH],
                            surface_texture: Some(textures::BRAT_TEXTURE),
                        });
                        player.level.number_of_sectors += 1;

                        for (i, (x, y)) in points.iter().enumerate() {
                            if i == points.len() - 1 {
                                player.level.walls.push(Wall {
                                    x1: ((points[0].0 as f32) / grid.scale as f32)
                                        - grid.view_shift_x as f32,
                                    y1: ((points[0].1 as f32) / grid.scale as f32)
                                        - grid.view_shift_y as f32,
                                    x2: ((points[i - 1].0 as f32) / grid.scale as f32
                                        - grid.view_shift_x as f32),
                                    y2: ((points[i - 1].1 as f32) / grid.scale as f32)
                                        - grid.view_shift_y as f32,

                                    color: new_color,
                                    u: 1.0,
                                    v: 1.0,
                                    texture: Some(textures::BRAT_TEXTURE),
                                });
                                player.level.number_of_walls += 1;
                            } else {
                                player.level.walls.push(Wall {
                                    x1: (*x as f32 / grid.scale as f32) - grid.view_shift_x as f32,
                                    y1: (*y as f32 / grid.scale as f32) - grid.view_shift_y as f32,
                                    x2: (points[i + 1].0 as f32 / grid.scale as f32)
                                        - grid.view_shift_x as f32,
                                    y2: (points[i + 1].1 as f32 / grid.scale as f32)
                                        - grid.view_shift_y as f32,

                                    color: new_color,
                                    u: 1.0,
                                    v: 1.0,
                                    texture: Some(textures::BRAT_TEXTURE),
                                });
                                player.level.number_of_walls += 1;
                            }
                        }
                        grid.new_sector = None;
                        grid.mouse_status.click_count = 0;
                    }
                }
            },
            _ => {
                grid.mouse_status.click_count = 0;
                grid.new_sector = Some(vec![]);

                self.draw_dot(
                    grid.mouse_status.mouse_x as f32,
                    grid.mouse_status.mouse_y as f32,
                    colors::WHITE,
                )?;
            }
        }
        Ok(())
    }

    pub fn highlight_rectangle(
        &mut self,
        grid: &mut Grid,
        init_x: i32,
        init_y: i32,
        draw: bool,
    ) -> Result<Rect, String> {
        let selection = Rect::new(
            init_x,
            init_y,
            (grid.mouse_status.mouse_x - init_x).abs() as u32,
            (grid.mouse_status.mouse_y - init_y).abs() as u32,
        );
        if draw {
            self.canvas.set_draw_color(Color {
                r: 70,
                g: 70,
                b: 70,
                a: 129,
            });
            self.canvas.set_blend_mode(BlendMode::Blend);
            self.canvas.fill_rect(selection)?;
        }

        Ok(selection)
    } // generates the current highlight rectangle and optionally draws it

    pub fn draw2d(
        // Master function for the the level builder;
        &mut self, // Draws a 2D representation of the sectors & walls in level.rs
        player: &mut PlayerInfo,
        grid: &mut Grid,
        font: &sdl2::ttf::Font,
    ) -> Result<(), String> {
        self.draw_mode = renderer::DrawMode::Draw2D;
        grid.scale = no_less_than_one(grid.scale);
        let scale_width = SCREEN_WIDTH / 160 * grid.scale as usize;
        let scale_height = SCREEN_HEIGHT / 120 * grid.scale as usize;

        //draw grid
        for x in 0..SCREEN_WIDTH as usize {
            let grid_x = x * scale_width;
            for y in 0..1 * SCREEN_HEIGHT {
                self.draw_dot(grid_x as f32, y as f32, colors::BLACK)?;
            }
        }
        for y in 0..SCREEN_HEIGHT as usize {
            let grid_y = y * scale_height;
            for x in 0..1 * SCREEN_WIDTH {
                self.draw_dot(x as f32, grid_y as f32, colors::BLACK)?;
            }
        }

        //draw sectors
        for s in 0..player.level.number_of_sectors as usize {
            for wall in player.level.sectors[s as usize].wall_start
                ..player.level.sectors[s as usize].wall_end
            {
                self.draw_thick_line(
                    (player.level.walls[wall as usize].x1 + grid.view_shift_x as f32)
                        * grid.scale as f32,
                    (player.level.walls[wall as usize].y1 + grid.view_shift_y as f32)
                        * grid.scale as f32,
                    (player.level.walls[wall as usize].x2 + grid.view_shift_x as f32)
                        * grid.scale as f32,
                    (player.level.walls[wall as usize].y2 + grid.view_shift_y as f32)
                        * grid.scale as f32,
                    player.level.walls[wall as usize].color,
                )?; // Draw walls

                // circle selected points from the drawn highlight
                if grid.selected_point.is_none() {
                    for point in grid.selection.points.clone() {
                        if (point as f32 / 2.0).fract() == 0.5 {
                            self.draw_circle(
                                (player.level.walls[(point as usize + 1) / 2].x1
                                    + grid.view_shift_x as f32)
                                    * grid.scale as f32,
                                (player.level.walls[(point as usize + 1) / 2].y1
                                    + grid.view_shift_y as f32)
                                    * grid.scale as f32,
                                4,
                                colors::WHITE,
                            )?; // circle all points in selection and keep them circled until another click
                        }
                    }
                }
                if grid.highlight_x.is_some() && grid.selected_point.is_none() {
                    let highghlight = self.highlight_rectangle(
                        grid,
                        grid.highlight_x.unwrap(),
                        grid.highlight_y.unwrap(),
                        false,
                    )?; // get the highlighted area

                    if highghlight.contains_point((
                        ((player.level.walls[wall as usize].x1 + grid.view_shift_x as f32)
                            * grid.scale as f32) as i32,
                        ((player.level.walls[wall as usize].y1 + grid.view_shift_y as f32)
                            * grid.scale as f32) as i32,
                    )) {
                        if wall > 0 {
                            let point_number = wall as usize + wall as usize - 1;
                            grid.selection.points.push(point_number); // if point1 is highlighted add it to selection.points
                        }
                    }
                    if highghlight.contains_point((
                        ((player.level.walls[wall as usize].x2 + grid.view_shift_x as f32)
                            * grid.scale as f32) as i32,
                        ((player.level.walls[wall as usize].y2 + grid.view_shift_y as f32)
                            * grid.scale as f32) as i32,
                    )) {
                        let point_number = wall as usize + wall as usize;
                        grid.selection.points.push(point_number); // if point2 is highlighted add it to selection.points
                    }
                }

                if grid.selected_wall.is_some() {
                    if wall == grid.selected_wall.unwrap() as i32 {
                        grid.selected_sector = Some(s);
                        self.draw_thick_line(
                            (player.level.walls[wall as usize].x1 + grid.view_shift_x as f32)
                                * grid.scale as f32,
                            (player.level.walls[wall as usize].y1 + grid.view_shift_y as f32)
                                * grid.scale as f32,
                            (player.level.walls[wall as usize].x2 + grid.view_shift_x as f32)
                                * grid.scale as f32,
                            (player.level.walls[wall as usize].y2 + grid.view_shift_y as f32)
                                * grid.scale as f32,
                            colors::WHITE,
                        )?; // Draw walls
                    }
                } // sets the selected sector according to the selected wall

                if distance(
                    grid.mouse_status.mouse_x as f32,
                    grid.mouse_status.mouse_y as f32,
                    (player.level.walls[wall as usize].x2 + grid.view_shift_x as f32)
                        * grid.scale as f32,
                    (player.level.walls[wall as usize].y2 + grid.view_shift_y as f32)
                        * grid.scale as f32,
                ) <= 6.0
                    && grid.highlight_x.is_none()
                {
                    for i in 1..6 {
                        self.draw_circle(
                            (player.level.walls[wall as usize].x2 + grid.view_shift_x as f32)
                                * grid.scale as f32,
                            (player.level.walls[wall as usize].y2 + grid.view_shift_y as f32)
                                * grid.scale as f32,
                            i,
                            colors::WHITE,
                        )?; // point mouse-over  animation
                    }
                }
            }
        }

        self.draw_player(
            (player.position.x + grid.view_shift_x) * grid.scale as i32,
            (player.position.y + grid.view_shift_y) * grid.scale as i32,
            colors::GREEN,
            grid,
            player,
        )?; //draw player

        if grid.new_sector.is_some() {
            self.draw_new_sector(grid, player)?;
        }

        let bar = (6 * SCREEN_HEIGHT) / 7;
        for y in bar..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                self.draw_dot(x as f32, y as f32, colors::GREY4)?;
            }
        } //draw background for toolbar

        let texture_creator = self.canvas.texture_creator();
        //draw text and buttons:
        let screen_x =
            ((grid.mouse_status.mouse_x) as f32 / (grid.scale as f32)) - grid.view_shift_x as f32;
        let screen_y =
            ((grid.mouse_status.mouse_y) as f32 / (grid.scale as f32)) - grid.view_shift_y as f32;
        let mouse_x_text = format!("Mouse x: {:?}", screen_x).to_string();
        let mouse_y_text = format!("Mouse y: {:?}", screen_y).to_string();

        if grid.selected_sector.is_some() {
            let sector_text = format!("Sector:{:#?}", grid.selected_sector.unwrap()).to_string();
            let wall_text = format!("Wall:{:#?}", grid.selected_wall.unwrap()).to_string();
            let wall_u_text = format!(
                "texture u:{}",
                player.level.walls[grid.selected_wall.unwrap()].u
            );
            let wall_v_text = format!(
                "texture v:{}",
                player.level.walls[grid.selected_wall.unwrap()].v
            );
            let floor_height_text = format!(
                "floor_z:{:#?}",
                player.level.sectors[grid.selected_sector.unwrap() as usize].bottom_height
            )
            .to_string();
            let ceiling_height_text = format!(
                "ceiling_z:{:#?}",
                player.level.sectors[grid.selected_sector.unwrap() as usize].top_height
            )
            .to_string();

            let texture_preview_rect = Rect::new(445, 735, 90, 90);
            self.canvas.set_draw_color(colors::RED);
            self.canvas.fill_rect(texture_preview_rect)?;
            let texture = get_texture(
                &texture_creator,
                player.level.walls[grid.selected_wall.unwrap() as usize]
                    .texture
                    .unwrap()
                    .width,
                player.level.walls[grid.selected_wall.unwrap() as usize]
                    .texture
                    .unwrap()
                    .height,
                player.level.walls[grid.selected_wall.unwrap() as usize]
                    .texture
                    .unwrap()
                    .data,
            )?;
            self.canvas.copy(&texture, None, texture_preview_rect)?;
            self.canvas.set_draw_color(colors::BLACK);
            for i in 0..3 {
                self.canvas.draw_rect(Rect::new(
                    444 - i,
                    734 - i,
                    (91 + i * 2) as u32,
                    (91 + i * 2) as u32,
                ))?;
            }

            Self::text(
                self,
                &texture_creator,
                &font,
                wall_text,
                colors::BLACK,
                240,
                740,
                110,
                25,
            )?; // wall number text
            Self::text(
                self,
                &texture_creator,
                &font,
                wall_u_text,
                colors::BLACK,
                240,
                770,
                120,
                20,
            )?; // wall u text
            Self::text(
                self,
                &texture_creator,
                &font,
                "+".to_string(),
                colors::BLACK,
                370,
                770,
                25,
                25,
            )?; // wall u plus
            Self::text(
                self,
                &texture_creator,
                &font,
                "-".to_string(),
                colors::BLACK,
                400,
                760,
                25,
                40,
            )?; // wall u minus
            Self::text(
                self,
                &texture_creator,
                &font,
                wall_v_text,
                colors::BLACK,
                240,
                800,
                120,
                20,
            )?; // wall v text
            Self::text(
                self,
                &texture_creator,
                &font,
                "+".to_string(),
                colors::BLACK,
                370,
                800,
                25,
                25,
            )?; // wall v plus
            Self::text(
                self,
                &texture_creator,
                &font,
                "-".to_string(),
                colors::BLACK,
                400,
                795,
                25,
                40,
            )?; // wall u minus

            Self::text(
                self,
                &texture_creator,
                &font,
                sector_text,
                colors::BLACK,
                25,
                740,
                140,
                25,
            )?; // sector number text

            Self::text(
                self,
                &texture_creator,
                &font,
                floor_height_text,
                colors::BLACK,
                25,
                770,
                120,
                20,
            )?; // sector floor text
            Self::text(
                self,
                &texture_creator,
                &font,
                "+".to_string(),
                colors::BLACK,
                160,
                770,
                25,
                25,
            )?; // sector floor plus
            Self::text(
                self,
                &texture_creator,
                &font,
                "-".to_string(),
                colors::BLACK,
                190,
                765,
                25,
                40,
            )?; // sector floor minus
            Self::text(
                self,
                &texture_creator,
                &font,
                ceiling_height_text,
                colors::BLACK,
                25,
                790,
                120,
                20,
            )?; // sector ceiling text
            Self::text(
                self,
                &texture_creator,
                &font,
                "+".to_string(),
                colors::BLACK,
                160,
                790,
                25,
                25,
            )?; // sector cieling plus
            Self::text(
                self,
                &texture_creator,
                &font,
                "-".to_string(),
                colors::BLACK,
                190,
                785,
                25,
                40,
            )?; // sector ceiling minus
        }

        Self::text(
            self,
            &texture_creator,
            &font,
            mouse_x_text,
            colors::BLACK,
            925,
            740,
            180,
            30,
        )?; // mouse x text
        Self::text(
            self,
            &texture_creator,
            &font,
            mouse_y_text,
            colors::BLACK,
            925,
            770,
            180,
            30,
        )?; // mouse y text

        if grid.highlight_x.is_some() && grid.selected_point.is_none() {
            self.highlight_rectangle(
                grid,
                grid.highlight_x.unwrap(),
                grid.highlight_y.unwrap(),
                true,
            )?;
        }

        Ok(())
    }
}

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
