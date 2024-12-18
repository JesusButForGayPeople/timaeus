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
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
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

use std::sync::Once;

const ANGLE_STEP: usize = 10; // Angle step in degrees
const NUM_ANGLES: usize = 360 / ANGLE_STEP; // Number of discrete angles

static INIT: Once = Once::new();
static mut SINE_LOOKUP: Option<[f32; NUM_ANGLES]> = None;
static mut COSINE_LOOKUP: Option<[f32; NUM_ANGLES]> = None;

pub fn initialize_lookup_tables() {
    INIT.call_once(|| {
        let mut sine_arr = [0.0; NUM_ANGLES];
        let mut cosine_arr = [0.0; NUM_ANGLES];
        for i in 0..NUM_ANGLES {
            sine_arr[i] = (i as f32 * ANGLE_STEP as f32).to_radians().sin();
            cosine_arr[i] = (i as f32 * ANGLE_STEP as f32).to_radians().cos();
        }
        unsafe {
            SINE_LOOKUP = Some(sine_arr);
            COSINE_LOOKUP = Some(cosine_arr);
        }
    });
}

#[allow(static_mut_refs)]
pub fn get_sine_lookup() -> &'static [f32; NUM_ANGLES] {
    initialize_lookup_tables();
    unsafe { SINE_LOOKUP.as_ref().unwrap() }
}
#[allow(static_mut_refs)]
pub fn get_cosine_lookup() -> &'static [f32; NUM_ANGLES] {
    initialize_lookup_tables();
    unsafe { COSINE_LOOKUP.as_ref().unwrap() }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct XYZ {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Default, Debug)]
pub struct PlayerInfo {
    pub position: XYZ,        // the players position in space
    pub angle_h_index: usize, // the horizontal angle of the players field of view
    pub level: Level,         // the map that the player is currently within; made up of sectors
    pub mouse_state: Option<MouseState>,
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
            angle_h_index: 0,
            level: init_level,
            mouse_state: None,
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

                    let world_x1 = x1 as f32 * get_cosine_lookup()[player.angle_h_index]
                        - y1 as f32 * get_sine_lookup()[player.angle_h_index];
                    let world_x2 = x2 as f32 * get_cosine_lookup()[player.angle_h_index]
                        - y2 as f32 * get_sine_lookup()[player.angle_h_index];

                    //world y position:
                    let world_y1 = y1 as f32 * get_cosine_lookup()[player.angle_h_index]
                        + x1 as f32 * get_sine_lookup()[player.angle_h_index];
                    let world_y2 = y2 as f32 * get_cosine_lookup()[player.angle_h_index]
                        + x2 as f32 * get_sine_lookup()[player.angle_h_index];

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
        player.angle_h_index = (player.angle_h_index + NUM_ANGLES - 1) % NUM_ANGLES;
    }

    pub fn look_right(player: &mut PlayerInfo) {
        player.angle_h_index = (player.angle_h_index + 1) % NUM_ANGLES;
    }

    pub fn move_forward(player: &mut PlayerInfo) {
        let dx = (get_sine_lookup()[player.angle_h_index] * 10.0) as i32;
        let dy = (get_cosine_lookup()[player.angle_h_index] * 10.0) as i32;
        player.position.x += dx;
        player.position.y += dy;
    }

    pub fn move_right(player: &mut PlayerInfo) {
        let dx = (get_sine_lookup()[player.angle_h_index] * 10.0) as i32;
        let dy = (get_cosine_lookup()[player.angle_h_index] * 10.0) as i32;
        player.position.x += dy;
        player.position.y -= dx;
    }

    pub fn move_left(player: &mut PlayerInfo) {
        let dx = (get_sine_lookup()[player.angle_h_index] * 10.0) as i32;
        let dy = (get_cosine_lookup()[player.angle_h_index] * 10.0) as i32;
        player.position.x -= dy;
        player.position.y += dx;
    }

    pub fn move_backward(player: &mut PlayerInfo) {
        let dx = (get_sine_lookup()[player.angle_h_index] * 10.0) as i32;
        let dy = (get_cosine_lookup()[player.angle_h_index] * 10.0) as i32;
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
    ((num as f32) / 180.0 * std::f32::consts::PI).sin()
} // gives the sine of a float as a percentage of 360 degrees

pub fn cosine(num: i32) -> f32 {
    ((num as f32) / 180.0 * std::f32::consts::PI).cos()
} // gives the cosine of a floatas a percentage of 360 degrees

pub fn one_if_none(n: f32) -> f32 {
    if n.abs() <= f32::EPSILON {
        f32::EPSILON * n.signum()
    } else {
        n
    }
}

pub fn no_less_than_one(n: i32) -> i32 {
    let k = std::cmp::max(n, 1);
    assert!(k >= 1);
    k
} // returns four if the given value is less than four (used to cap grid scale)

pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let d = ((x2 - x1).hypot(y2 - y1)).abs();
    assert!(d >= 0.0);
    d
} // calculates simple 2D cartesean distance

pub fn sort(mut sec_vec: Vec<Sector>) -> Vec<Sector> {
    sec_vec.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    assert!(sec_vec[0].distance >= sec_vec[1].distance);
    sec_vec
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Texture {
    name: &'static str,
    width: u32,
    height: u32,
    data: &'static [u32],
}

//Logs:

const SECONDS_IN_A_DAY: i64 = 86400;
const SECONDS_IN_AN_HOUR: i64 = 3600;

pub fn initialize_log_file() -> std::io::Result<std::fs::File> {
    // Ensure the "logs" directory exists
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir(log_dir)?;
    }

    // Get the current time and format it
    let start_time = SystemTime::now();
    let datetime = start_time.duration_since(UNIX_EPOCH).unwrap();
    let timestamp = datetime.as_secs() as i64;

    // Convert timestamp to EST (UTC-5)
    let est_offset = -5 * SECONDS_IN_AN_HOUR;
    let est_timestamp = timestamp + est_offset;

    // Calculate date and time components
    let days_since_epoch = est_timestamp / SECONDS_IN_A_DAY;
    let seconds_in_day = est_timestamp % SECONDS_IN_A_DAY;

    let mut year = 1970;
    let mut days_remaining = days_since_epoch;

    while days_remaining >= 365 {
        if is_leap_year(year) {
            if days_remaining >= 366 {
                days_remaining -= 366;
                year += 1;
            }
        } else {
            days_remaining -= 365;
            year += 1;
        }
    }

    let (month, day) = calculate_month_and_day(days_remaining as u32, year);

    let hour = (seconds_in_day / SECONDS_IN_AN_HOUR) % 24;
    let minute = (seconds_in_day % SECONDS_IN_AN_HOUR) / 60;
    let second = seconds_in_day % 60;

    // Format the hour and period (am/pm)
    let period = if hour < 12 { "am" } else { "pm" };
    let formatted_hour = if hour == 0 {
        12
    } else if hour > 12 {
        hour - 12
    } else {
        hour
    };

    // Create the log file name
    let log_file_name = format!(
        "{:02}-{:02}-{:02}_{:02}-{:02}-{:02}{}_log.txt",
        month,
        day,
        year % 100,
        formatted_hour,
        minute,
        second,
        period
    );

    // Create a log file with the formatted date and time
    let log_file_path = log_dir.join(log_file_name);
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file_path)?;

    // Write header to the log file
    writeln!(log_file, "Log File Title: Timaeus Log")?;
    writeln!(
        log_file,
        "Date and Time: {:02}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        month,
        day,
        year % 100,
        formatted_hour,
        minute,
        second,
        period
    )?;
    writeln!(log_file, "----------------------------------------")?;

    Ok(log_file)
}

pub fn log_event(log_file: &mut std::fs::File, event: &str) -> std::io::Result<()> {
    let now = SystemTime::now();
    let datetime = now.duration_since(UNIX_EPOCH).unwrap();
    let timestamp = datetime.as_secs() as i64;

    // Convert timestamp to EST (UTC-5)
    let est_offset = -5 * SECONDS_IN_AN_HOUR;
    let est_timestamp = timestamp + est_offset;

    // Calculate date and time components
    let days_since_epoch = est_timestamp / SECONDS_IN_A_DAY;
    let seconds_in_day = est_timestamp % SECONDS_IN_A_DAY;

    let mut year = 1970;
    let mut days_remaining = days_since_epoch;

    while days_remaining >= 365 {
        if is_leap_year(year) {
            if days_remaining >= 366 {
                days_remaining -= 366;
                year += 1;
            }
        } else {
            days_remaining -= 365;
            year += 1;
        }
    }

    let (month, day) = calculate_month_and_day(days_remaining as u32, year);

    let hour = (seconds_in_day / SECONDS_IN_AN_HOUR) % 24;
    let minute = (seconds_in_day % SECONDS_IN_AN_HOUR) / 60;
    let second = seconds_in_day % 60;

    // Format the hour and period (am/pm)
    let period = if hour < 12 { "am" } else { "pm" };
    let formatted_hour = if hour == 0 {
        12
    } else if hour > 12 {
        hour - 12
    } else {
        hour
    };

    // Create the timestamp string
    let timestamp_str = format!(
        "{:02}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        month,
        day,
        year % 100,
        formatted_hour,
        minute,
        second,
        period
    );

    writeln!(log_file, "[{}] {}", timestamp_str, event)?;
    Ok(())
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn calculate_month_and_day(day_of_year: u32, year: i64) -> (u32, u32) {
    let days_in_month = [
        31,
        28 + if is_leap_year(year) { 1 } else { 0 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0;
    let mut day = day_of_year;

    while day >= days_in_month[month] {
        day -= days_in_month[month];
        month += 1;
    }

    (month as u32 + 1, day + 1)
}

pub fn log_player_state(log_file: &mut std::fs::File, player: &PlayerInfo) -> std::io::Result<()> {
    let event = format!(
        "Player position: {:?}, Player angle_h: {:?}",
        player.position, player.angle_h_index
    );
    log_event(log_file, &event)
}
