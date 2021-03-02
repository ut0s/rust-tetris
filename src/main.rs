use rand::Rng;

use std::mem::swap;
use std::vec::Vec;
use std::{thread, time};

const WIDTH: usize = 25;
const HEIGHT: usize = 60;
// const WIDTH: usize = 10;
// const HEIGHT: usize = 20;

const MINO_WIDTH: usize = 4;
const MINO_HEIGHT: usize = 4;
const MINO_KIND: usize = 7;

const CHAR_WALL: char = '#';
const CHAR_EMPTY: char = ' ';

const FALL_INTERVAL_MS: u64 = 300;
const MOVE_INTERVAL_MS: u64 = 150;
const ROT_INTERVAL_MS: u64 = 150;

enum FieldState {
  WALL,
  EMPTY,
}

enum Tetrimino {
  I,
  O,
  L,
  J,
  S,
  Z,
  T,
}

struct Console {
  mino_posx: usize,
  mino_posy: usize,
  mino_shape: Tetrimino,
  mino_rot: u32,
  score: u32,
}

impl Console {
  fn clear(&self) {
    // print!("\x1B[2J");
    print!("{}[2J", 27 as char);
  }

  fn init_field(&self, f: &mut Vec<Vec<bool>>) {
    for y in 1..HEIGHT {
      for x in 1..WIDTH {
        // side wall
        if x == 1 || x == WIDTH - 1 {
          f[y][x] = true;
        }
        // bottom wall
        if y == HEIGHT - 1 {
          f[y][x] = true;
        }
      }
    }
  }

  fn draw_score(&self) {
    println!("Tetris by Rust.\tSCORE : {}", self.score);
  }

  fn draw_xy(&self, f: &Vec<Vec<bool>>, x: usize, y: usize) {
    if f[y][x] == true {
      print!("{}", CHAR_WALL);
    } else {
      print!("{}", CHAR_EMPTY);
    }
  }

  fn draw_field(&self, f: &Vec<Vec<bool>>) {
    print!("{esc}[0J", esc = 27 as char);
    for y in 1..HEIGHT {
      for x in 1..WIDTH {
        self.draw_xy(&f, x, y);
      }
      // newsline
      println!("");
    }
  }

  fn put_mino(&self, f: &mut Vec<Vec<bool>>, m: &Vec<Vec<bool>>) {
    for y in 0..MINO_HEIGHT {
      for x in 0..MINO_WIDTH {
        if m[y][x] == true {
          f[self.mino_posy + y][self.mino_posx + x] = true;
        } else {
          f[self.mino_posy + y][self.mino_posx + x] = false;
        }
      }
    }
  }

  fn new_mino(&self, mino: &mut Vec<Vec<bool>>) {
    let mut rng = rand::thread_rng();
    let pos = rng.gen_range(2..WIDTH - 1);
    println!("{}", pos);

    match self.mino_shape {
      Tetrimino::I => {
        mino[0] = [false, true, false, false].to_vec();
        mino[1] = [false, true, false, false].to_vec();
        mino[2] = [false, true, false, false].to_vec();
        mino[3] = [false, true, false, false].to_vec();
      }
      Tetrimino::O => {
        mino[0] = [true, true, false, false].to_vec();
        mino[1] = [true, true, false, false].to_vec();
        mino[2] = [false, false, false, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
      Tetrimino::L => {
        mino[0] = [false, true, false, false].to_vec();
        mino[1] = [false, true, false, false].to_vec();
        mino[2] = [false, true, true, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
      Tetrimino::J => {
        mino[0] = [false, true, false, false].to_vec();
        mino[1] = [false, true, false, false].to_vec();
        mino[2] = [true, true, false, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
      Tetrimino::S => {
        mino[0] = [false, true, true, false].to_vec();
        mino[1] = [true, true, false, false].to_vec();
        mino[2] = [false, false, false, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
      Tetrimino::Z => {
        mino[0] = [true, true, false, false].to_vec();
        mino[1] = [false, true, true, false].to_vec();
        mino[2] = [false, false, false, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
      Tetrimino::T => {
        mino[0] = [false, true, false, false].to_vec();
        mino[1] = [true, true, true, false].to_vec();
        mino[2] = [false, false, false, false].to_vec();
        mino[3] = [false, false, false, false].to_vec();
      }
    }
  }

  fn rot_matrix(&self, mino: &mut Vec<Vec<bool>>) {
    mino.reverse();
    for i in 1..mino.len() {
      let (left, right) = mino.split_at_mut(i);
      for j in 0..i {
        // let t = matrix[i][j];
        // matrix[i][j] = matrix[j][i];
        // matrix[j][i] = t;
        std::mem::swap(&mut left[j][i], &mut right[0][j]);
      }
    }
  }

  fn rot_mino(&self, mino: &mut Vec<Vec<bool>>) {
    let r = (4000000 + self.mino_rot) % 4;
    for _ in 0..r {
      self.rot_matrix(mino);
    }
  }
}

fn main() {
  println!("Tetris");

  let mut rng = rand::thread_rng();

  let mut console = Console {
    mino_posx: 2,
    mino_posy: 0,
    mino_shape: match rng.gen_range(0..MINO_KIND) {
      0 => Tetrimino::I,
      1 => Tetrimino::O,
      2 => Tetrimino::L,
      3 => Tetrimino::J,
      4 => Tetrimino::S,
      5 => Tetrimino::Z,
      6 => Tetrimino::T,
      _ => Tetrimino::T,
    },
    // mino_shape: Tetrimino::I,
    mino_rot: 0,
    score: 0,
  };

  // initialize field and mino
  // let mut old_field = vec![vec![false; WIDTH]; HEIGHT];
  let mut field = vec![vec![false; WIDTH]; HEIGHT];
  let mut mino = vec![vec![false; MINO_WIDTH]; MINO_HEIGHT];

  // debug
  // println!("{:?}", &f);

  console.init_field(&mut field);

  console.new_mino(&mut mino);
  // console.rot_mino(&mut mino);

  loop {
    // wait
    thread::sleep(time::Duration::from_millis(FALL_INTERVAL_MS));

    console.clear();
    console.draw_score();

    let old_field = field.clone();
    console.put_mino(&mut field, &mino);

    console.draw_field(&field);

    if console.mino_posy < HEIGHT - MINO_HEIGHT - 1 {
      console.mino_posy += 1;
    } else {
      console.mino_posy = 0;
    }

    field = old_field.clone();
  }
}
