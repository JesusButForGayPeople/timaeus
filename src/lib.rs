use crate::level::{INIT_SECTORS, INIT_WALLS, NUM_SECTORS, NUM_WALLS};
pub use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::{MouseButton, MouseState, MouseWheelDirection},
    pixels::{Color, PixelFormat, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Canvas, TextureCreator},
    ttf::Sdl2TtfContext,
    video::{Window, WindowContext},
    EventPump,
};
pub use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
    vec::Vec,
};
pub mod colors;
pub mod grid;
pub mod level;
pub mod renderer;
#[allow(non_snake_case)]
pub mod textures;

//Constants:
pub const RESOLUTION: usize = 7;
pub const SCREEN_WIDTH: usize = RESOLUTION * 160;
pub const HALF_WIDTH: usize = SCREEN_WIDTH / 2;
pub const SCREEN_HEIGHT: usize = RESOLUTION * 120;
pub const HALF_HEIGHT: usize = SCREEN_HEIGHT / 2;
pub const PIXEL_SCALE: usize = 1;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct XYZ {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Default, Debug)]
pub struct PlayerInfo {
    pub position: XYZ, // the players position in space
    pub angle_h: i32,  // the horizontal angle of the players field of view
    pub level: Level,  // the map that the player is currently within; made up of sectors
}

impl PlayerInfo {
    pub fn new() -> PlayerInfo {
        let init_sectors: Vec<Sector> = Vec::from(INIT_SECTORS); // sectors & their walls are stored in level.rs to allow for editing by draw2d
        let init_walls: Vec<Wall> = Vec::from(INIT_WALLS);
        let init_level = Level {
            number_of_sectors: NUM_SECTORS as u32,
            sectors: init_sectors,
            number_of_walls: NUM_WALLS as u32,
            walls: init_walls,
        };
        PlayerInfo {
            position: XYZ {
                x: 32,
                y: 32,
                z: 10,
            },
            angle_h: 0,
            level: init_level,
        }
    }

    pub fn distances(player: &mut PlayerInfo) -> &mut PlayerInfo {
        for sector in player.level.sectors.iter_mut() {
            for (i, wall) in player.level.walls.iter().enumerate() {
                if sector.wall_start as usize <= i && i < sector.wall_end as usize {
                    //oftset bottom 2 points by player:
                    let x1 = wall.x1 as i32 - player.position.x;
                    let y1 = wall.y1 as i32 - player.position.y;
                    let x2 = wall.x2 as i32 - player.position.x;
                    let y2 = wall.y2 as i32 - player.position.y;

                    let world_x1 =
                        x1 as f32 * cosine(player.angle_h) - y1 as f32 * sine(player.angle_h);
                    let world_x2 =
                        x2 as f32 * cosine(player.angle_h) - y2 as f32 * sine(player.angle_h);

                    //world y position:
                    let world_y1 =
                        y1 as f32 * cosine(player.angle_h) + x1 as f32 * sine(player.angle_h);
                    let world_y2 =
                        y2 as f32 * cosine(player.angle_h) + x2 as f32 * sine(player.angle_h);

                    sector.distance = distance(
                        0.0,
                        0.0,
                        (world_x1 + world_x2) / 2.0,
                        (world_y1 + world_y2) / 2.0,
                    );
                }
            }
            sector.distance /= sector.wall_end as f32 - sector.wall_start as f32;
        }
        player.level.sectors = sort(player.level.sectors.clone());
        player
    } // calculates the distance from the player to a sector and sorts the sectors by distance to the player

    // player movement funtcions:
    pub fn move_up(player: &mut PlayerInfo) {
        player.position.z -= PIXEL_SCALE as i32;
    }
    pub fn move_down(player: &mut PlayerInfo) {
        player.position.z += PIXEL_SCALE as i32;
    }
    pub fn look_left(player: &mut PlayerInfo) {
        player.angle_h -= 10;
    }
    pub fn look_right(player: &mut PlayerInfo) {
        player.angle_h += 10;
    }
    pub fn move_fowward(player: &mut PlayerInfo) {
        let dx = (sine(player.angle_h) * 10.0) as i32;
        let dy = (cosine(player.angle_h) * 10.0) as i32;
        player.position.x += dx;
        player.position.y += dy;
    }
    pub fn move_right(player: &mut PlayerInfo) {
        let dx = (sine(player.angle_h) * 10.0) as i32;
        let dy = (cosine(player.angle_h) * 10.0) as i32;
        player.position.x += dy;
        player.position.y -= dx;
    }
    pub fn move_left(player: &mut PlayerInfo) {
        let dx = (sine(player.angle_h) * 10.0) as i32;
        let dy = (cosine(player.angle_h) * 10.0) as i32;
        player.position.x -= dy;
        player.position.y += dx;
    }
    pub fn move_backward(player: &mut PlayerInfo) {
        let dx = (sine(player.angle_h) * 10.0) as i32;
        let dy = (cosine(player.angle_h) * 10.0) as i32;
        player.position.x -= dx;
        player.position.y -= dy;
    }
}

#[derive(Clone, Default, Debug)]
pub struct Level {
    pub number_of_sectors: u32,
    pub sectors: Vec<Sector>, // 3d space enclosed by walls on all sides and optionally surfaces on the top and bottom
    pub number_of_walls: u32,
    pub walls: Vec<Wall>, // horizontal pane used to build sectors
}

#[derive(Clone, Copy, Debug)]
pub struct Wall {
    pub x1: f32, // first x
    pub y1: f32, // first y
    pub x2: f32, // last x
    pub y2: f32, // last y
    pub color: Color,
    pub texture: Option<Texture>,
    pub u: f32,
    pub v: f32,
}

impl Wall {
    pub fn get_points(self) -> Vec<(f32, f32)> {
        let mut points = Vec::new();
        let x_float = self.x2 - self.x1;
        let y_float = self.y2 - self.y1;
        let x = x_float as i32;
        let y = y_float as i32;
        let max = std::cmp::max(x.abs(), y.abs());
        let dx = x_float / one_if_none(max as f32);
        let dy = y_float / one_if_none(max as f32);
        for n in 0..max {
            let point = (self.x1 + n as f32 * dx, self.y1 + n as f32 * dy);
            points.push(point);
        }
        points
    } // returns either the first or second point of a given wall
    pub fn next_texture(&mut self) {
        for (i, texture) in textures::TEXTURES.iter().enumerate() {
            match self.texture {
                Some(self_texture) => {
                    if texture == &self_texture {
                        if i == textures::TEXTURES.len() - 1 {
                            self.texture = Some(textures::TEXTURES[0]);
                            return;
                        } else {
                            self.texture = Some(textures::TEXTURES[i + 1]);
                            return;
                        }
                    }
                }
                _ => (),
            }
        }
    }
    pub fn prev_texture(&mut self) {
        for (i, texture) in textures::TEXTURES.iter().enumerate() {
            match self.texture {
                Some(self_texture) => {
                    if texture == &self_texture {
                        if i == 0 {
                            self.texture = Some(textures::TEXTURES[textures::TEXTURES.len() - 1]);
                            return;
                        } else {
                            self.texture = Some(textures::TEXTURES[i - 1]);
                            return;
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sector {
    pub wall_start: i32, // walls are assigned to sectors ordinally so each sector says which wall indicates its start
    pub wall_end: i32,   // ...  and which wall indicateds its end
    pub bottom_height: i32, // the height of the floor of the sector
    pub top_height: i32, // the height of the cieling of the sector
    pub distance: f32,   // distance from the player; calculated from the center of the sector
    pub top_color: Color, // ceiling color
    pub bottom_color: Color, // floor color
    pub surface_points: [u32; SCREEN_WIDTH], // used to store the value of the points in the visible surface of a sector which are then used to draw the surface on the next loop
    pub surface: Option<Surface>, // indicates which surface (if any) is currently being drawn
    pub surface_texture: Option<Texture>, // texture of the surface
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Surface {
    TopScan,    // indicates that ceiling poi.nts should be saved
    BottomScan, // indicates that floor points should be saved
} // ...  and as such sould not be saved nor drawn

//math functions:
pub fn sine(num: i32) -> f32 {
    ((num as f32 - 0.001) / 180.0 * std::f32::consts::PI).sin()
} // gives the sine of a float as a percentage of 360 degrees

pub fn cosine(num: i32) -> f32 {
    ((num as f32 + 0.001) / 180.0 * std::f32::consts::PI).cos()
} // gives the cosine of a floatas a percentage of 360 degrees

pub fn one_if_none(n: f32) -> f32 {
    if n == 0.0 {
        return 1.0;
    } else {
        return n;
    }
} // returns one if the given value is zero

pub fn no_less_than_one(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    } else {
        return n;
    }
} // returns four if the given value is less than four (used to cap grid scale)

pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt()
} // calculates simple 2D cartesean distance

pub fn sort(mut sec_vec: Vec<Sector>) -> Vec<Sector> {
    let mut swapped = true;
    while swapped {
        swapped = false;
        for i in 0..sec_vec.len() - 1 {
            if sec_vec[i].distance <= sec_vec[i + 1].distance {
                sec_vec.swap(i, i + 1);
                swapped = true;
            }
        }
    }
    return sec_vec;
} // simple bubble sort for sectors based on distance

pub fn mouse_point(mouse_x: f32, mouse_y: f32) -> (f32, f32) {
    (mouse_x, mouse_y)
}

pub fn wall_point(
    player: &mut PlayerInfo,
    grid: &mut grid::Grid,
    wall_number: usize,
    point: usize,
) -> Result<(f32, f32), String> {
    if point == 1 {
        Ok((
            (player.level.walls[wall_number as usize].x1 + grid.view_shift_x as f32)
                * grid.scale as f32,
            (player.level.walls[wall_number as usize].y1 + grid.view_shift_y as f32)
                * grid.scale as f32,
        ))
    } else if point == 2 {
        Ok((
            (player.level.walls[wall_number as usize].x2 + grid.view_shift_x as f32)
                * grid.scale as f32,
            (player.level.walls[wall_number as usize].y2 + grid.view_shift_y as f32)
                * grid.scale as f32,
        ))
    } else {
        Err("Error!".to_string())
    }
} // returns the first or second point of a given wall

pub fn is_even(x: i32) -> bool {
    if (x as f32 / 2.0).fract() == 0.0 {
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Texture {
    name: &'static str,
    width: u32,
    height: u32,
    data: &'static [u32],
}
