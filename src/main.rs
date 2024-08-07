use timaeus::grid::*;
use timaeus::renderer::Renderer;

fn main() -> Result<(), String> {
    println!("{:?}", debug::double_degree_test());
    //initialization:
    for i in 0..2 {
        println!("{:?}", i);
    }

    let sdl_context = sdl2::init()?;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut frame_count = 0;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"fonts/BigBlueTermPlusNerdFont-Regular.ttf");
    let font = ttf_context.load_font(font_path, 128)?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut player = PlayerInfo::new();
    let mut grid = Grid::new();
    grid.selection = Selection::from_level(&player.level);

    let window = video_subsystem
        .window("Rume W.I.P.", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut renderer = Renderer::new(window)?;

    'running: loop {
        let state = event_pump.mouse_state(); // offset mouse position so that it reflects its position within the actual grid
        let relative_state = event_pump.relative_mouse_state();
        let screen_x = ((state.x()) as f32 / (grid.scale as f32)) - grid.view_shift_x as f32;
        let screen_y = ((state.y()) as f32 / (grid.scale as f32)) - grid.view_shift_y as f32;
        grid.get_mouse_status(state);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        if grid.mouse_status.click_toggle {
                            grid.mouse_status.click_toggle = false;
                        } else if grid.mouse_status.click_toggle == false {
                            grid.mouse_status.click_toggle = true;
                        }
                        grid.mouse_status.relative_x = Some(relative_state.x());
                        grid.mouse_status.relative_y = Some(relative_state.y());
                        if grid.selected_point.is_none() && grid.state == State::Free {
                            grid.selection.sectors = Vec::new();
                            grid.selection.walls = Vec::new();
                            grid.selection.points = Vec::new();
                            grid.highlight_x = Some(x);
                            grid.highlight_y = Some(y)
                        }
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        grid.state = State::Free;
                        grid.highlight_x = None;
                        grid.highlight_y = None;
                    }
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod: Mod::NOMOD,
                    ..
                } => match keycode {
                    Keycode::Q => break 'running,
                    Keycode::Escape => Grid::deselect(&mut grid),
                    Keycode::Equals => grid.scale += 1,
                    Keycode::Minus => grid.scale -= 1,
                    Keycode::Up => PlayerInfo::move_up(&mut player),
                    Keycode::Left => PlayerInfo::move_left(&mut player),
                    Keycode::Down => PlayerInfo::move_down(&mut player),
                    Keycode::Right => PlayerInfo::move_right(&mut player),

                    Keycode::W => PlayerInfo::move_fowward(&mut player),
                    Keycode::A => PlayerInfo::look_left(&mut player),
                    Keycode::S => PlayerInfo::move_backward(&mut player),
                    Keycode::D => PlayerInfo::look_right(&mut player),
                    Keycode::J => save(&mut player),
                    Keycode::N => Grid::new_sector(&mut grid, &mut player),
                    Keycode::Y => grid.new_sector = true,

                    _ => {}
                },
                _ => {}
            }
        }

        if grid.mouse_status.button.is_none() == true {
            Grid::deselect(&mut grid)
        } // deselect all points if there is no mouse button (note: this does not clear grid.selection)

        if grid.selected_point.is_some() {
            grid.selection.sectors = Vec::new();
            grid.selection.walls = Vec::new();
            grid.selection.points = Vec::new();
        } // if a point is selected clear grid.selection as to not highlight any points besides the one being moved

        for sector in 0..player.level.number_of_sectors {
            for wall_number in player.level.sectors[sector as usize].wall_start
                ..player.level.sectors[sector as usize].wall_end
            {
                if grid.highlight_x.is_some() {
                    let init_x = grid.highlight_x.unwrap();
                    let init_y = grid.highlight_y.unwrap();
                    let highghlight =
                        renderer.highlight_rectangle(&mut grid, init_x, init_y, false)?;

                    if highghlight.contains_point((
                        ((player.level.walls[wall_number as usize].x1 + grid.view_shift_x as f32)
                            * grid.scale as f32) as i32,
                        ((player.level.walls[wall_number as usize].y1 + grid.view_shift_y as f32)
                            * grid.scale as f32) as i32,
                    )) {
                        if wall_number > 0 {
                            let point_number = (2 * wall_number as usize) - 1;
                            grid.selection.points.push(point_number); // if a point is contained in a highlighted regin it is added to selection.points which
                        } // is then used to render the little white selection cirlces on all of the points in selection.points
                    }
                }

                if wall_number + 1 < player.level.sectors[sector as usize].wall_end {
                    let wall = wall_number as usize; // this massive set of control flow statements is what dictates which points are moved when a vertex is clicked on
                    let next = (wall + 1) as usize; // since the selected wall does not repeat automatically if the last wall of a sector was selected
                    if distance(
                        grid.mouse_status.mouse_x as f32, // a shorter statement would allow you to drag the point from the next sector along with the intended point
                        grid.mouse_status.mouse_y as f32, // thus, since each wall has two points there is eight cases to cover all possible combinations of points
                        wall_point(&mut player, &mut grid, wall, 1)?.0, // ...  and to account for the number of walls in the sector
                        wall_point(&mut player, &mut grid, wall, 1)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 2)?.0,
                            wall_point(&mut player, &mut grid, next, 2)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(1);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 1)?.0,
                        wall_point(&mut player, &mut grid, wall, 1)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 1)?.0,
                            wall_point(&mut player, &mut grid, next, 1)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(2);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 2)?.0,
                        wall_point(&mut player, &mut grid, wall, 2)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 1)?.0,
                            wall_point(&mut player, &mut grid, next, 1)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(3);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 2)?.0,
                        wall_point(&mut player, &mut grid, wall, 2)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 2)?.0,
                            wall_point(&mut player, &mut grid, next, 2)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(4);
                        }
                    }
                } else if wall_number + 1 == player.level.sectors[sector as usize].wall_end {
                    let wall = wall_number as usize;
                    let next = player.level.sectors[sector as usize].wall_start as usize;

                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 1)?.0,
                        wall_point(&mut player, &mut grid, wall, 1)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 2)?.0,
                            wall_point(&mut player, &mut grid, next, 2)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(5);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 1)?.0,
                        wall_point(&mut player, &mut grid, wall, 1)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 1)?.0,
                            wall_point(&mut player, &mut grid, next, 1)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(6);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 2)?.0,
                        wall_point(&mut player, &mut grid, wall, 2)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 1)?.0,
                            wall_point(&mut player, &mut grid, next, 1)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(7);
                        }
                    }
                    if distance(
                        grid.mouse_status.mouse_x as f32,
                        grid.mouse_status.mouse_y as f32,
                        wall_point(&mut player, &mut grid, wall, 2)?.0,
                        wall_point(&mut player, &mut grid, wall, 2)?.1,
                    ) <= 8.0
                        && distance(
                            grid.mouse_status.mouse_x as f32,
                            grid.mouse_status.mouse_y as f32,
                            wall_point(&mut player, &mut grid, next, 2)?.0,
                            wall_point(&mut player, &mut grid, next, 2)?.1,
                        ) <= 8.0
                        && grid.mouse_status.button == Some(Button::Left)
                        && grid.state == State::Free
                    {
                        if grid.selection.points.is_empty() {
                            grid.highlight_x = None;
                            grid.highlight_y = None;

                            grid.state = State::Busy;

                            grid.selected_sector = Some(sector as usize);
                            grid.selected_wall = Some(wall_number as usize);
                            grid.selected_point = Some(8);
                        }
                    }
                }
            }
        }

        if grid.selected_point.is_some() {
            grid.highlight_x = None;
            grid.highlight_y = None;
            if grid.selected_point == Some(1) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].y1 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap()].x1 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap() + 1].y2 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap() + 1].x2 = screen_x;
                }
            }
            if grid.selected_point == Some(2) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].y1 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap()].x1 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap() + 1].y1 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap() + 1].x1 = screen_x;
                }
            }
            if grid.selected_point == Some(3) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].x2 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap()].y2 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap() + 1].x1 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap() + 1].y1 = screen_y;
                }
            }
            if grid.selected_point == Some(4) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].x2 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap()].y2 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap() + 1].x2 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap() + 1].y2 = screen_y;
                }
            }

            let second_wall =
                (player.level.sectors[grid.selected_sector.unwrap()].wall_start) as usize;
            if grid.selected_point == Some(5) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].y1 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap()].x1 = screen_x;
                    player.level.walls[second_wall].y2 = screen_y;
                    player.level.walls[second_wall].x2 = screen_x;
                }
            }
            if grid.selected_point == Some(6) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].y1 = screen_y;
                    player.level.walls[grid.selected_wall.unwrap()].x1 = screen_x;
                    player.level.walls[second_wall].y1 = screen_y;
                    player.level.walls[second_wall].x1 = screen_x;
                }
            }
            if grid.selected_point == Some(7) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].x2 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap()].y2 = screen_y;
                    player.level.walls[second_wall].x1 = screen_x;
                    player.level.walls[second_wall].y1 = screen_y;
                }
            }
            if grid.selected_point == Some(8) {
                if grid.selected_wall == None {
                } else {
                    grid.state = State::Busy;
                    player.level.walls[grid.selected_wall.unwrap()].x2 = screen_x;
                    player.level.walls[grid.selected_wall.unwrap()].y2 = screen_y;
                    player.level.walls[second_wall].x2 = screen_x;
                    player.level.walls[second_wall].y2 = screen_y;
                }
            }
        }
        frame_count += 1;
        if frame_count == 10 {
            frame_count -= 10;
        }

        Renderer::draw(&mut renderer, &mut player, &mut grid, &font)?;
    }

    std::thread::sleep(frame_duration);
    Ok(())
}
