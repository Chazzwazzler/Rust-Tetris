use std::f32::consts::PI;
use std::io::{stdout, Stdout};
use std::io;
use std::io::Read;
use std::time::Duration;

use crossterm::{self, terminal};
use crossterm::{execute,cursor};
use crossterm::terminal::{Clear,ClearType};
use crossterm::cursor::Hide;

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

use rand::Rng;

fn main() {
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, Hide).unwrap();

    let input_channel = spawn_input_channel();

    let mut map: [[char; 10]; 20] = [
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.'],
        ['.','.','.','.','.','.','.','.','.','.']
    ];

    let i_block: [(f32,f32);5] = [(0.5, -0.5), (0.0,0.0), (-1.0,0.0), (1.0,0.0), (2.0,0.0)];
    let r_block: [(f32,f32);5] = [(0.0,0.0), (0.0,0.0), (1.0,0.0), (-1.0,0.0), (-1.0,-1.0)];
    let l_block: [(f32,f32);5] = [(0.0,0.0), (0.0,0.0), (1.0,0.0), (-1.0,0.0), (1.0,-1.0)];
    let o_block: [(f32,f32);5] = [(0.5,-0.5), (0.0,0.0), (0.0,-1.0), (1.0,0.0), (1.0,-1.0)];
    let s_block: [(f32,f32);5] = [(0.0,0.0), (0.0,0.0), (-1.0,0.0), (0.0,-1.0), (1.0,-1.0)];
    let t_block: [(f32,f32);5] = [(0.0,0.0), (0.0,0.0), (-1.0,0.0), (1.0,0.0), (0.0,-1.0)];
    let z_block: [(f32,f32);5] = [(0.0,0.0), (0.0,0.0), (-1.0,-1.0), (0.0,-1.0), (1.0,0.0)];

    let tetriminos: [[(f32,f32);5];7] = [i_block,r_block,l_block,o_block,s_block,t_block,z_block];
    let tetrimino_characters: [char;7] = ['S', 'G', '@', '#', 'Q','X','$'];

    let mut rand_index = rand::thread_rng().gen_range(0..tetriminos.len());
    let mut current_block: [(f32,f32);5] = tetriminos[rand_index];
    let mut position: (f32, f32) = (4.0, 3.0);
    
    draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);

    let mut tick: i32 = 0; 
    let mut lock_tick: i32 = 0;
    let mut drop_rate: i32 = 15;

    let mut placed_tiles: Vec<(f32,f32)> = vec![];

    let mut kill_program: bool = false;

    let mut score: f32 = 0.0;

    loop {
        tick += 1;
        lock_tick += 1;

        if lock_tick >= 30{
            tick = 1;
            lock_tick = 0;
            for tile in current_block{
                placed_tiles.push((tile.0 + position.0, tile.1 + position.1));
            }
            for i in 1..current_block.len(){
                placed_tiles.push((current_block[i].0 + position.0, current_block[i].1 + position.1));
            }
            rand_index = rand::thread_rng().gen_range(0..tetriminos.len());
            current_block = tetriminos[rand_index];

            for row in 0..map.len(){
                if line_full(map, row){
                    let mut map_vect = map.to_vec();
                    map_vect.remove(row);
                    map_vect.insert(0, ['.','.','.','.','.','.','.','.','.','.']);
                    score += 100.0;
                    
                    placed_tiles = vec![];
                    for row2 in 0..map.len(){
                        map[row2] = map_vect[row2];
                        for column in 0..map[row2].len(){
                            if map[row2][column] != '.'{
                                placed_tiles.push((column as f32, row2 as f32));
                            }
                        }
                    }
                }
            }

            position = (4.0, 3.0);
            for tilething in current_block{
                if map[(tilething.1+position.1) as usize][(tilething.0+position.0) as usize] != '.'{
                    kill_program = true;
                    break;
                }
            }
            draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);
        }

        if kill_program {
            break;
        }

        match input_channel.try_recv() {
            Ok(key) => match key{
                [b'w'] => {
                    clear_piece(current_block, &mut map, position);
                    let rotated_piece = rotate_piece(current_block, 90.0, position, &placed_tiles);
                    current_block = rotated_piece.0;
                    lock_tick = 0;
                    position = (position.0 + rotated_piece.1.0, position.1 + rotated_piece.1.1);
                    draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);
                },
                [b'a'] => {
                    if in_bounds(current_block, (position.0 - 1.0, position.1), &placed_tiles){
                        clear_piece(current_block, &mut map, position);
                        position.0 -= 1.0;
                        lock_tick = 0;
                        draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);
                    }
                },
                [b'd'] => {
                    if in_bounds(current_block, (position.0 + 1.0, position.1), &placed_tiles){
                        clear_piece(current_block, &mut map, position);
                        position.0 += 1.0;
                        lock_tick = 0;
                        draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);
                    }
                },
                [b's'] => {
                    drop_rate = 2;
                },
                [b'r'] => {
                    clear_piece(current_block, &mut map, position);
                    rand_index = rand::thread_rng().gen_range(0..tetriminos.len());
                    current_block = tetriminos[rand_index];
                    position = (4.0, 3.0);
                    lock_tick = 0;
                    draw_piece(current_block, &mut map, position,  tetrimino_characters, rand_index);
                },
                [b'q'] => {
                    break;
                }
                _ => ()
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }

        if tick % drop_rate == 0 && in_bounds(current_block, (position.0, position.1 + 1.0), &placed_tiles) {
            clear_piece(current_block, &mut map, position);
            position.1 += 1.0;
            lock_tick = 0;
            draw_piece(current_block, &mut map, position, tetrimino_characters, rand_index);
            score += 1.0;
        }

        drop_rate = 15;

        update(&stdout, map);
        let score_string: String = format!("Tetris  Score: {}",score);
        execute!(stdout,crossterm::terminal::SetTitle(score_string)).unwrap();
        std::thread::sleep(Duration::from_millis(20));
    }

    loop {
        update(&stdout, map);
        println!("{}", score.round());

        match input_channel.try_recv() {
            Ok(key) => match key{
                [b'q'] => {
                    break;
                }
                _ => ()
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        std::thread::sleep(Duration::from_millis(20)); 
    }
}

fn update (mut stdout: &Stdout, map: [[char; 10]; 20]){
    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, Clear(ClearType::FromCursorDown), cursor::MoveTo(0, 0)).unwrap();

    print!("  ");
    for slice in map {
        for tile in slice {
            print!("{tile} ");
        }
        println!();
        print!("  ");
    }
}

fn line_full (map: [[char; 10]; 20], index: usize) -> bool{
    for column in 0..map[index].len(){
        if map[index][column] == '.'{
            return false
        }
    }
    return true
}

fn clear_piece (piece:[(f32,f32);5], map: &mut[[char; 10]; 20], position: (f32,f32)){
    for i in 1..piece.len(){
        map[(piece[i].1 + position.1) as usize][(piece[i].0 + position.0) as usize] = '.';
    }
}

fn draw_piece (piece:[(f32,f32);5], map: &mut[[char; 10]; 20], position: (f32,f32), tetrimino_characters: [char;7], rand_index:usize){
    for i in 1..piece.len(){
        map[(piece[i].1 + position.1) as usize][(piece[i].0 + position.0) as usize] = tetrimino_characters[rand_index];
    }
}

fn rotate_piece (piece:[(f32,f32);5], angle:f32, position: (f32,f32), placed_tiles: &Vec<(f32,f32)>) -> ([(f32,f32);5], (f32,f32)){
    let mut rotated_piece = piece;

    for i in 1..rotated_piece.len(){
        let angle_rad = angle * (PI / 180.0);
        let point = (rotated_piece[i].0,rotated_piece[i].1);
        let point_origin = (rotated_piece[0].0,rotated_piece[0].1);
        let x_rot = ((point.0-point_origin.0)*angle_rad.cos() - (point.1-point_origin.1)*angle_rad.sin()) + point_origin.0;  
        let y_rot = ((point.0-point_origin.0)*angle_rad.sin() + (point.1-point_origin.1)*angle_rad.cos()) + point_origin.1;
        rotated_piece[i] = (x_rot.round(), y_rot.round());
    }

    let mut offset: (f32, f32) = (0.0,0.0);

    if !in_bounds(rotated_piece, position, &placed_tiles){
        for i in 1..rotated_piece.len(){
            if rotated_piece[i].1+position.1 > 19.0{
                if (rotated_piece[i].1+position.1-19.0).abs() > (offset.0.abs() + offset.1.abs()){
                    offset.1 = -1.0 * (rotated_piece[i].1+position.1-19.0);
                }
            }
            else if rotated_piece[i].1+position.1 < 0.0{
                if (rotated_piece[i].1+position.1).abs() > (offset.0.abs() + offset.1.abs()){
                    offset.1 = -1.0 * (rotated_piece[i].1+position.1);
                }
            }
            else if rotated_piece[i].0+position.0 > 9.0{
                if (rotated_piece[i].0+position.0-9.0).abs() > (offset.0.abs() + offset.1.abs()){
                    offset.0 = -1.0 * (rotated_piece[i].0+position.0-9.0);
                }
            }
            else if rotated_piece[i].0+position.0 < 0.0{
                if (rotated_piece[i].0+position.0).abs() > (offset.0.abs() + offset.1.abs()){
                    offset.0 = -1.0 * (rotated_piece[i].0+position.0);
                }
            }
        }

        if !in_bounds(rotated_piece, (position.0+offset.0, position.1+offset.1), &placed_tiles){
            return (piece, (0.0,0.0))
        }
    }
        
    (rotated_piece, offset)
}

fn in_bounds (piece:[(f32,f32);5], position: (f32,f32), placed_tiles: &Vec<(f32,f32)>) -> bool{
    for i in 1..piece.len(){
        if piece[i].1+position.1 > 19.0 || piece[i].1+position.1 < 0.0{
            return false
        }
        if piece[i].0+position.0 > 9.0 || piece[i].0+position.0 < 0.0{
            return false
        }
        if placed_tiles.contains(&(piece[i].0+position.0, piece[i].1+position.1)){
            return false
        }
    }
    true
}

fn spawn_input_channel() -> Receiver<[u8; 1]>{
    let (tx, rx) = mpsc::channel::<[u8; 1]>();
    std::thread::spawn(move || loop {
        let mut buffer = [0; 1];
        io::stdin().read(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}