
use std::{sync::OnceLock, collections::HashSet, error::Error};

use fixtures::test_board;
use sdl2::{event, keyboard::Keycode, pixels::Color, rect::{Point, Rect}, sys::xdg_surface, EventPump};
use sys::{SdlContext, LOGICAL_HEIGHT, LOGICAL_WIDTH, SCALE, TILE_SIZE};

mod sys;
mod fixtures;

type Board = [[Tile; 9]; 9];

fn numbers() -> &'static HashSet<u8> {
    static NUMBERS: OnceLock<HashSet<u8>> = OnceLock::new();
    NUMBERS.get_or_init(|| {
       HashSet::from_iter(1..10)
    })
}

fn main() -> Result<(), Box<dyn Error>>{
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let mut ctx = sys::init_sdl_systems(&sdl, &video)?;
    let ttf = sdl2::ttf::init()?;
    let font = sys::load_font(&ttf)?;
    
    let mut board = [[Tile::Empty; 9]; 9];
    let mut cursor_index = (0, 0);
    
    let mut running = true;
    let mut solving = false;
    let mut visual_solving = true;
    let mut solving_idx = 0;
    
    while running {
        if solving {
            match solve(&mut board, solving_idx) {
                BoardState::Solving(idx) => {
                    if solving_idx != 80 {
                        solving_idx = idx;
                    } else {
                        solving = false;
                    }
                },
                BoardState::Finished => {
                    solving = false;
                }
            }
        }
        
        let mut render = true;
        match handle_input(&mut ctx.events, &mut running) {
            Action::Move(x, y) => cursor_index = ((cursor_index.0 + x).clamp(0, 8), (cursor_index.1 + y).clamp(0, 8)),
            Action::Solve => {
                if valid_board(&board) {
                    solving = !solving;
                }
            },
            Action::Write(num) => board[cursor_index.1 as usize][cursor_index.0 as usize] = Tile::Hard(num),
            Action::Remove => board[cursor_index.1 as usize][cursor_index.0 as usize] = Tile::Empty,
            Action::ToggleVisual => visual_solving = dbg!(!visual_solving),
            Action::PrintBoard => { dbg!(&board); },
            Action::LoadTest => board = test_board(),
            Action::Nothing => render = false
        }
        
        if visual_solving || !solving || render {
            render_board(&board, cursor_index, &mut ctx, &font, solving);
        }
    }
    Ok(())
}

fn solve(board: &mut Board, solving_idx: usize) -> BoardState {
    let pos = get_pos(solving_idx);
    let prev = match board[pos.1][pos.0] {
        Tile::Hard(_) => if solving_idx != 80 {
            return BoardState::Solving(solving_idx + 1)
        } else {
            return BoardState::Finished
        },
        Tile::Soft(num) => num,
        Tile::Empty => 0
    };
    
    let mut possible: Vec<u8> = numbers().difference(&taken_values(board, pos)).map(|n| *n).collect();
    possible.retain(|&n| n > prev);
    possible.sort();
    
    match possible.first() {
        Some(num) => {
            board[pos.1][pos.0] = Tile::Soft(*num);
            BoardState::Solving(solving_idx + 1)
        },
        None => {
            if solving_idx != 0 {
                board[pos.1][pos.0] = Tile::Empty;
                BoardState::Solving(decrement_until_soft(solving_idx, board))
            } else {
                panic!("Trying to backtrack off the board");
            }
        }
    }
}

fn decrement_until_soft(idx: usize, board: &Board) -> usize {
    let mut idx = idx;
    loop {
        idx -= 1;
        let pos = get_pos(idx);
        if let Tile::Hard(_) = board[pos.1][pos.0] {
            continue;
        }
        return idx;
    }
}

fn taken_values(board: &Board, pos: (usize, usize)) -> HashSet<u8> {
    let mut numbers = HashSet::new();
    
    // Column
    for y in 0..9 {
        match board[y][pos.0] {
            Tile::Soft(num) | Tile::Hard(num) => {
                numbers.insert(num);
            },
            _ => ()
        }
    }
    
    // Row
    for x in 0..9 {
        match board[pos.1][x] {
            Tile::Soft(num) | Tile::Hard(num) => {
                numbers.insert(num);
            },
            _ => ()
        }
    }
    
    // Section
    let top_left = ((pos.0 / 3) * 3, (pos.1 / 3) * 3);
    for y in top_left.1..top_left.1 + 3 {
        for x in top_left.0..top_left.0 + 3 {
            match board[y][x] {
                Tile::Soft(num) | Tile::Hard(num) => {
                    numbers.insert(num);
                },
                _ => continue
            }
        }
    }
    
    numbers
}

fn get_pos(idx: usize) -> (usize, usize) {
    (idx % 9, idx / 9)
}

enum BoardState {
    // index of solving position
    Solving(usize),
    Finished
}

fn render_board(board: &Board, cursor_index: (i8, i8), ctx: &mut SdlContext, font: &sdl2::ttf::Font, solving: bool) {
    let bg_color = if solving || valid_board(board) {
        Color::WHITE
    } else {
        Color::RGB(255, 220, 220)
    };
    ctx.canvas.set_draw_color(bg_color);
    ctx.canvas.clear();
    
    draw_square(cursor_index, ctx, Color::RGB(200, 200, 200));
    render_numbers(board, cursor_index, ctx, font);
    
    ctx.canvas.set_draw_color(Color::BLACK);
    render_grid(ctx);
    
    ctx.canvas.present();
}

fn valid_board(board: &Board) -> bool {
    for y in 0..3 {
        for x in 0..3 {
            if !valid_section((x, y), board) { return false }
        }
    }
    
    for y in 0..9 {
        if !valid_row(y, board) { return false }
    }
    
    for x in 0..9 {
        if !valid_column(x, board) { return false }
    }
    
    true
}

fn valid_column(x: usize, board: &Board) -> bool {
    let mut numbers_hash: HashSet<u8> = HashSet::new();
    let mut numbers_vec: Vec<u8> = Vec::with_capacity(9);
    for y in 0..9 {
        match board[y][x] {
            Tile::Soft(num) | Tile::Hard(num) => {
                numbers_vec.push(num);
                numbers_hash.insert(num);
            },
            _ => continue
        }
    }
    numbers_vec.len() == numbers_hash.len()
}

fn valid_row(y: usize, board: &Board) -> bool {
    let mut numbers_hash: HashSet<u8> = HashSet::new();
    let mut numbers_vec: Vec<u8> = Vec::with_capacity(9);
    for tile in board[y] {
        match tile {
            Tile::Soft(num) | Tile::Hard(num) => {
                numbers_vec.push(num);
                numbers_hash.insert(num);
            },
            _ => continue
        }
    }
    numbers_vec.len() == numbers_hash.len()
}

fn valid_section(pos: (usize, usize), board: &Board) -> bool {
    let top_left = (pos.0 * 3, pos.1 * 3);
    let mut numbers_hash: HashSet<u8> = HashSet::new();
    let mut numbers_vec: Vec<u8> = Vec::with_capacity(9);
    for y in top_left.1..top_left.1 + 3 {
        for x in top_left.0..top_left.0 + 3 {
            match board[y][x] {
                Tile::Soft(num) | Tile::Hard(num) => {
                    numbers_vec.push(num);
                    numbers_hash.insert(num);
                },
                _ => continue
            }
        }
    }
    numbers_vec.len() == numbers_hash.len()
}

fn draw_square(pos: (i8, i8), ctx: &mut SdlContext, color: Color) {
    ctx.canvas.set_draw_color(color);
    let _ = ctx.canvas.fill_rect(Rect::new((pos.0 as u32 * TILE_SIZE) as _, (pos.1 as u32 * TILE_SIZE) as _, TILE_SIZE, TILE_SIZE));
}

fn render_numbers(board: &Board, cursor_index: (i8, i8), ctx: &mut SdlContext, font: &sdl2::ttf::Font) {
    for y in 0..board.len() {
        for (x, tile) in board[y].iter().enumerate() {
            match tile {
                Tile::Hard(num) => {
                    let color = if x == cursor_index.0 as _ && y == cursor_index.1 as _ {
                        Color::RGB(200, 200, 0)
                    } else {
                        Color::YELLOW
                    };
                    draw_square((x as _, y as _), ctx, color);
                    render_number(*num, (x as u32, y as u32), ctx, font);
                },
                Tile::Soft(num) => {
                    render_number(*num, (x as u32, y as u32), ctx, font);
                },
                _ => ()
            }
        }
    }
}

fn render_number(number: u8, pos: (u32, u32), ctx: &mut SdlContext, font: &sdl2::ttf::Font) {
    let surface = font.render(&number.to_string()).blended(Color::BLACK).unwrap();

    let texture = ctx
        .texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

    let sdl2::render::TextureQuery { width, height, .. } = texture.query();

    let target = Rect::new((pos.0 * TILE_SIZE + TILE_SIZE / 2 - width / 2 + 1) as i32, (pos.1 * TILE_SIZE + TILE_SIZE / 2 - height / 2 + 2) as i32, width, height);
    let _ = ctx.canvas.copy(&texture, None, Some(target));
}

fn render_grid(ctx: &mut SdlContext) {
    for x in 0..9 {
        if x % 3 == 0 {
            let _ = ctx.canvas.draw_line(Point::new((x * TILE_SIZE - 1) as _, 0), Point::new((x * TILE_SIZE - 1) as _, (LOGICAL_HEIGHT) as _));
            let _ = ctx.canvas.draw_line(Point::new((x * TILE_SIZE + 1) as _, 0), Point::new((x * TILE_SIZE + 1) as _, (LOGICAL_HEIGHT) as _));
        }
        let _ = ctx.canvas.draw_line(Point::new((x * TILE_SIZE) as _, 0), Point::new((x * TILE_SIZE) as _, (LOGICAL_HEIGHT) as _));
    }
    
    for y in 0..9 {
        if y % 3 == 0 {
            let _ = ctx.canvas.draw_line(Point::new(0, (y * TILE_SIZE - 1) as _), Point::new((LOGICAL_WIDTH) as _, (y * TILE_SIZE - 1) as _));
            let _ = ctx.canvas.draw_line(Point::new(0, (y * TILE_SIZE + 1) as _), Point::new((LOGICAL_WIDTH) as _, (y * TILE_SIZE + 1) as _));
        }
        let _ = ctx.canvas.draw_line(Point::new(0, (y * TILE_SIZE) as _), Point::new((LOGICAL_WIDTH) as _, (y * TILE_SIZE) as _));
    }
}

#[derive(Clone, Copy, Debug)]
enum Tile {
    Hard(u8),
    Soft(u8),
    Empty
}

enum Action {
    Write(u8),
    Remove,
    Move(i8, i8),
    Solve,
    ToggleVisual,
    PrintBoard,
    LoadTest,
    Nothing
}

fn handle_input(
    events: &mut EventPump,
    running: &mut bool,
) -> Action {
    if let Some(event) = events.poll_iter().next() {
        use sdl2::event::Event as Ev;

        return match event {
            Ev::Quit { .. } => {
                *running = false;
                Action::Nothing
            },
            Ev::KeyDown {
                keycode: Some(kc),
                repeat: false,
                ..
            } => match kc {
                Keycode::Num1 => Action::Write(1),
                Keycode::NUM_2 => Action::Write(2),
                Keycode::NUM_3 => Action::Write(3),
                Keycode::NUM_4 => Action::Write(4),
                Keycode::NUM_5 => Action::Write(5),
                Keycode::NUM_6 => Action::Write(6),
                Keycode::NUM_7 => Action::Write(7),
                Keycode::NUM_8 => Action::Write(8),
                Keycode::NUM_9 => Action::Write(9),
                Keycode::Backspace => Action::Remove,
                Keycode::Right => Action::Move(1, 0),
                Keycode::Left => Action::Move(-1, 0),
                Keycode::Up => Action::Move(0, -1),
                Keycode::Down => Action::Move(0, 1),
                Keycode::Space => Action::Solve,
                Keycode::V => Action::ToggleVisual,
                Keycode::T => Action::LoadTest,
                Keycode::P => Action::PrintBoard,
                _ => Action::Nothing,
            },
            Ev::KeyDown {
                keycode: Some(kc),
                repeat: true,
                ..
            } => match kc {
                Keycode::Right => Action::Move(1, 0),
                Keycode::Left => Action::Move(-1, 0),
                Keycode::Up => Action::Move(0, -1),
                Keycode::Down => Action::Move(0, 1),
                _ => Action::Nothing,
            },
            _ => Action::Nothing,
        }
    }
    Action::Nothing
}
