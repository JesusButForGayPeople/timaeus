use timaeus::grid::*;

use timaeus::renderer::{DrawMode::*, Renderer};

// git commit ./
// git push origin main

fn main() -> Result<(), String> {
    //initialization:
    let sdl_context = sdl2::init()?;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut _frame_count = 0;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"fonts/BigBlueTermPlusNerdFont-Regular.ttf");
    let font = ttf_context.load_font(font_path, 128)?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut player = PlayerInfo::new();
    let mut grid = Grid::new();
    grid.selection = Selection::from_level(&player.level);
    // let mut debug2: Option<Debug> = None;

    let window = video_subsystem
        .window("Timaeus W.I.P.", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut renderer = Renderer::new(window)?;

    'running: loop {
        if renderer.draw_mode == Draw3D {
            sdl_context.mouse().warp_mouse_in_window(
                renderer.canvas.window(),
                HALF_WIDTH as i32,
                HALF_HEIGHT as i32,
            );
        }

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
                    match mouse_btn {
                        MouseButton::Left => {
                            if renderer.draw_mode == renderer::DrawMode::Draw2D {
                                grid.mouse_status.relative_x = Some(relative_state.x());
                                grid.mouse_status.relative_y = Some(relative_state.y());
                                if grid.selected_point.is_none() && grid.state == State::Free {
                                    //grid.selection.sectors = Vec::new();
                                    grid.selection.walls = Vec::new();
                                    grid.selection.points = Vec::new();
                                    grid.highlight_x = Some(x);
                                    grid.highlight_y = Some(y)
                                }

                                if grid.selected_sector.is_some() {
                                    if grid.selected_wall.is_some() {
                                        // next texture with left mouse by clicking on preview with left mouse
                                        if state.y() >= 735 && state.y() <= 825 {
                                            if state.x() >= 445 && state.x() <= 535 {
                                                Wall::next_texture(
                                                    &mut player.level.walls
                                                        [grid.selected_wall.unwrap()],
                                                )
                                            }
                                        }
                                    }
                                    // floor height plus and minus 1 with left mouse
                                    if state.y() >= 770 && state.y() <= 795 {
                                        if state.x() >= 150 && state.x() <= 175 {
                                            player.level.sectors[grid.selected_sector.unwrap()]
                                                .bottom_height += 1;
                                        }
                                        if state.x() >= 190 && state.x() <= 215 {
                                            player.level.sectors[grid.selected_sector.unwrap()]
                                                .bottom_height -= 1;
                                        }
                                    }
                                    // ceiling  height plus and minus 1 with left mouse
                                    if state.y() >= 790 && state.y() <= 815 {
                                        if state.x() >= 160 && state.x() <= 185 {
                                            player.level.sectors[grid.selected_sector.unwrap()]
                                                .top_height += 1;
                                        }
                                        if state.x() >= 190 && state.x() <= 215 {
                                            player.level.sectors[grid.selected_sector.unwrap()]
                                                .top_height -= 1;
                                        }
                                    }
                                    // wall u plus and minus 1 with left mouse
                                    if state.y() >= 770 && state.y() <= 795 {
                                        if state.x() >= 370 && state.x() <= 395 {
                                            player.level.walls[grid.selected_wall.unwrap()].u +=
                                                1.0;
                                        }
                                        if state.x() >= 400 && state.x() <= 425 {
                                            player.level.walls[grid.selected_wall.unwrap()].u -=
                                                1.0;
                                        }
                                    }
                                    //wall v plus and minus 1 with left mouse
                                    if state.y() >= 790 && state.y() <= 815 {
                                        if state.x() >= 370 && state.x() <= 395 {
                                            player.level.walls[grid.selected_wall.unwrap()].v +=
                                                1.0;
                                        }
                                        if state.x() >= 400 && state.x() <= 425 {
                                            player.level.walls[grid.selected_wall.unwrap()].v -=
                                                1.0;
                                        }
                                    }
                                }
                            }
                        }
                        MouseButton::Right => {
                            if renderer.draw_mode == renderer::DrawMode::Draw2D {
                                match grid.selected_sector {
                                    Some(sector) => {
                                        if grid.selected_wall.is_some() {
                                            // prev texture with left mouse by clicking on preview with right mouse
                                            if state.y() >= 735 && state.y() <= 825 {
                                                if state.x() >= 445 && state.x() <= 535 {
                                                    Wall::prev_texture(
                                                        &mut player.level.walls
                                                            [grid.selected_wall.unwrap()],
                                                    )
                                                }
                                            }
                                        }
                                        // floor height plus and minus 10 with right mouse
                                        if state.y() >= 770 && state.y() <= 795 {
                                            if state.x() >= 160 && state.x() <= 185 {
                                                player.level.sectors[sector].bottom_height += 10;
                                            }
                                            if state.x() >= 190 && state.x() <= 215 {
                                                player.level.sectors[sector].bottom_height -= 10;
                                            }
                                        }
                                        // ceiling height plus and minus 10 with right mouse
                                        if state.y() >= 790 && state.y() <= 815 {
                                            if state.x() >= 160 && state.x() <= 185 {
                                                player.level.sectors[sector].top_height += 10;
                                            }
                                            if state.x() >= 190 && state.x() <= 215 {
                                                player.level.sectors[sector].top_height -= 10;
                                            }
                                        }
                                    }
                                    _ => {} // no selected sector
                                }
                            }
                        }
                        _ => {} // no mouse button
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
                    Keycode::Up => match renderer.draw_mode {
                        Draw3D => PlayerInfo::move_up(&mut player),
                        Draw2D => Grid::view_up(&mut grid),
                    },
                    Keycode::Left => match renderer.draw_mode {
                        Draw3D => PlayerInfo::move_left(&mut player),
                        Draw2D => Grid::view_left(&mut grid),
                    },
                    Keycode::Down => match renderer.draw_mode {
                        Draw3D => PlayerInfo::move_down(&mut player),
                        Draw2D => Grid::view_down(&mut grid),
                    },
                    Keycode::Right => match renderer.draw_mode {
                        Draw3D => PlayerInfo::move_right(&mut player),
                        Draw2D => Grid::view_right(&mut grid),
                    },

                    Keycode::W => match renderer.draw_mode {
                        Draw3D => PlayerInfo::move_fowward(&mut player),
                        Draw2D => Grid::next_wall(&mut grid, &mut player),
                    },
                    Keycode::A => PlayerInfo::look_left(&mut player),
                    Keycode::S => PlayerInfo::move_backward(&mut player),
                    Keycode::D => PlayerInfo::look_right(&mut player),
                    Keycode::J => save(&mut player),
                    Keycode::M => match renderer.draw_mode {
                        Draw3D => renderer.draw_mode = Draw2D,
                        Draw2D => renderer.draw_mode = Draw3D,
                    },
                    Keycode::N => Grid::new_sector(&mut grid, &mut player),
                    Keycode::Y => grid.new_sector = Some(Vec::new()),
                    Keycode::P => {
                        println!("{:?}", player.position)
                    }

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
                if grid.mouse_status.button == Some(Button::Left) && grid.state == State::Free {
                    let wall = wall_number as usize;
                    let mut next_is_last = false;
                    let mut next = 0;
                    if wall_number + 1 < player.level.sectors[sector as usize].wall_end {
                        // this massive set of control flow statements is what dictates which points are moved when a vertex is clicked on
                        next = (wall + 1) as usize;
                    }
                    // since the selected wall does not repeat automatically if the last wall of a sector was selected
                    else if wall_number + 1 == player.level.sectors[sector as usize].wall_end {
                        next = player.level.sectors[sector as usize].wall_start as usize;
                        next_is_last = true;
                    }
                    if grid.selection.points.is_empty() {
                        for (i, j) in [(1, 2), (1, 1), (2, 1), (2, 2)] {
                            if distance(
                                grid.mouse_status.mouse_x as f32, // a shorter statement would allow you to drag the point from the next sector along with the intended point
                                grid.mouse_status.mouse_y as f32, // thus, since each wall has two points there is eight cases to cover all possible combinations of points
                                wall_point(&mut player, &mut grid, wall, i)?.0, // ...  and to account for the number of walls in the sector
                                wall_point(&mut player, &mut grid, wall, i)?.1,
                            ) <= 8.0
                                && distance(
                                    grid.mouse_status.mouse_x as f32,
                                    grid.mouse_status.mouse_y as f32,
                                    wall_point(&mut player, &mut grid, next, j)?.0,
                                    wall_point(&mut player, &mut grid, next, j)?.1,
                                ) <= 8.0
                            {
                                match (i, j) {
                                    (1, 2) => {
                                        grid.highlight_x = None;
                                        grid.highlight_y = None;
                                        grid.state = State::Busy;
                                        grid.selected_sector = Some(sector as usize);
                                        grid.selected_wall = Some(wall_number as usize);
                                        match next_is_last {
                                            true => grid.selected_point = Some(5),
                                            false => grid.selected_point = Some(1),
                                        }
                                    }
                                    (1, 1) => {
                                        grid.highlight_x = None;
                                        grid.highlight_y = None;
                                        grid.state = State::Busy;
                                        grid.selected_sector = Some(sector as usize);
                                        grid.selected_wall = Some(wall_number as usize);
                                        match next_is_last {
                                            true => grid.selected_point = Some(6),
                                            false => grid.selected_point = Some(2),
                                        }
                                    }
                                    (2, 1) => {
                                        grid.highlight_x = None;
                                        grid.highlight_y = None;
                                        grid.state = State::Busy;
                                        grid.selected_sector = Some(sector as usize);
                                        grid.selected_wall = Some(wall_number as usize);
                                        match next_is_last {
                                            true => grid.selected_point = Some(7),
                                            false => grid.selected_point = Some(3),
                                        }
                                    }
                                    (2, 2) => {
                                        grid.highlight_x = None;
                                        grid.highlight_y = None;
                                        grid.state = State::Busy;
                                        grid.selected_sector = Some(sector as usize);
                                        grid.selected_wall = Some(wall_number as usize);
                                        match next_is_last {
                                            true => grid.selected_point = Some(8),
                                            false => grid.selected_point = Some(4),
                                        }
                                    }
                                    _ => {
                                        println!("point match error")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        match grid.selected_point {
            Some(mut point) => {
                grid.highlight_x = None;
                grid.highlight_y = None;
                grid.state = State::Busy;
                match grid.selected_wall {
                    Some(wall) => {
                        let mut second_wall = wall;
                        if point <= 4 {
                            second_wall = wall + 1;
                        }
                        if point >= 5 {
                            second_wall = (player.level.sectors[grid.selected_sector.unwrap()]
                                .wall_start) as usize;
                            point -= 4;
                        }

                        match point {
                            1 => {
                                player.level.walls[wall].y1 = screen_y;
                                player.level.walls[wall].x1 = screen_x;
                                player.level.walls[second_wall].y2 = screen_y;
                                player.level.walls[second_wall].x2 = screen_x;
                            }
                            2 => {
                                player.level.walls[wall].y1 = screen_y;
                                player.level.walls[wall].x1 = screen_x;
                                player.level.walls[second_wall].y1 = screen_y;
                                player.level.walls[second_wall].x1 = screen_x;
                            }
                            3 => {
                                player.level.walls[wall].x2 = screen_x;
                                player.level.walls[wall].y2 = screen_y;
                                player.level.walls[second_wall].x1 = screen_x;
                                player.level.walls[second_wall].y1 = screen_y;
                            }
                            4 => {
                                player.level.walls[wall].x2 = screen_x;
                                player.level.walls[wall].y2 = screen_y;
                                player.level.walls[second_wall].x2 = screen_x;
                                player.level.walls[second_wall].y2 = screen_y;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        _frame_count += 1;
        // let player_clone = player.clone();
        // let grid_clone = grid.clone();
        // if debug2.is_some() {
        //     debug2 = Some(debug(player_clone, grid_clone, Some(debug2.unwrap())));
        // } else {
        //     debug2 = Some(debug(player_clone, grid_clone, None));
        // }

        Renderer::draw(&mut renderer, &mut player, &mut grid, &font)?;
    }

    std::thread::sleep(frame_duration);
    Ok(())
}

pub struct Debug {
    player: PlayerInfo,
    grid: grid::Grid,
}

pub fn debug(player: PlayerInfo, grid: grid::Grid, prev_frame: Option<Debug>) -> Debug {
    match prev_frame {
        Some(prev_frame) => {
            if prev_frame.player.position != player.position {
                println!(
                    "Player position: {:#?},\n Player angle_h: {:#?}",
                    player.position, player.angle_h
                );
            }
            if prev_frame.player.angle_h != player.angle_h {
                println!(
                    "Player position: {:#?},\n Player angle_h: {:#?}",
                    player.position, player.angle_h
                );
            }
            if prev_frame.grid != grid {
                println!("Grid x: {:#?}", grid);
            }
        }
        _ => (),
    }
    Debug { player, grid }
}
