use rand::Rng;

extern crate termion;

use std::io::{stdin, stdout, Read, Write};
use termion::async_stdin;
use termion::clear;
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::mem::swap;
use std::vec::Vec;
use std::{process, thread, time};

// const WIDTH: usize = 25;
// const HEIGHT: usize = 60;
const WIDTH: usize = 10;
const HEIGHT: usize = 20;

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

  fn is_moveable(
    &self,
    f: &Vec<Vec<bool>>,
    m: &Vec<Vec<bool>>,
    posx: &usize,
    posy: &usize,
  ) -> bool {
    let mut ret = true;
    for y in 0..MINO_HEIGHT {
      for x in 0..MINO_WIDTH {
        if m[y][x] == true && f[posy + y][posx + x] == true {
          // cannot move
          ret = false;
          return ret;
        }
      }
    }
    ret
  }

  fn put_mino(&self, f: &mut Vec<Vec<bool>>, m: &Vec<Vec<bool>>) {
    for y in 0..MINO_HEIGHT {
      for x in 0..MINO_WIDTH {
        if m[y][x] == true {
          f[self.mino_posy + y][self.mino_posx + x] = true;
        }
      }
    }
  }

  fn select_mino(&mut self) {
    let mut rng = rand::thread_rng();
    self.mino_shape = match rng.gen_range(0..MINO_KIND) {
      0 => Tetrimino::I,
      1 => Tetrimino::O,
      2 => Tetrimino::L,
      3 => Tetrimino::J,
      4 => Tetrimino::S,
      5 => Tetrimino::Z,
      6 => Tetrimino::T,
      _ => Tetrimino::T,
    };
  }

  fn new_mino(&mut self, mino: &mut Vec<Vec<bool>>) {
    let mut rng = rand::thread_rng();
    let pos = rng.gen_range(2..WIDTH - MINO_WIDTH - 1);
    println!("{}", pos);
    self.mino_posx = pos;

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

  fn rot_mino(&mut self, mino: &mut Vec<Vec<bool>>) {
    let r = self.mino_rot % 4;
    for _ in 0..r {
      self.rot_matrix(mino);
    }
    self.mino_rot = 0;
  }
}

fn main() {
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

  let stdout = stdout();
  let mut stdout = stdout.lock().into_raw_mode().unwrap();
  let mut stdin = async_stdin().bytes();

  loop {
    write!(stdout, "{}", termion::clear::CurrentLine).unwrap();

    let b = stdin.next();

    match b {
      Some(Ok(b'q')) => {
        break;
      }
      Some(Ok(b'a')) => {
        if console.is_moveable(
          &field,
          &mino,
          &(console.mino_posx - 1),
          &(console.mino_posy),
        ) {
          console.mino_posx -= 1;
        }
      }
      Some(Ok(b'd')) => {
        if console.is_moveable(
          &field,
          &mino,
          &(console.mino_posx + 1),
          &(console.mino_posy),
        ) {
          console.mino_posx += 1;
        }
      }
      Some(Ok(b'w')) => {
        console.mino_rot += 1;
      }
      Some(Ok(b's')) => {
        console.mino_rot += 3;
      }
      _ => {
        if console.is_moveable(
          &field,
          &mino,
          &(console.mino_posx),
          &(console.mino_posy + 1),
        ) {
          console.mino_posy += 1;
        } else {
          console.put_mino(&mut field, &mino);

          console.mino_posy = 0;

          console.select_mino();
          console.new_mino(&mut mino);
        }
      }
    }
    stdout.flush().unwrap();

    {
      // 画面全体をクリアする
      write!(stdout, "{}", clear::All);
      // カーソルを左上に設定する(1-indexed)
      write!(stdout, "{}", cursor::Goto(1, 1));

      // title and score
      write!(stdout, "Tetris by Rust.\tSCORE : {}\r\n", console.score);

      let old_field = field.clone();

      console.rot_mino(&mut mino);

      // put mino
      for y in 0..MINO_HEIGHT {
        for x in 0..MINO_WIDTH {
          if mino[y][x] == true {
            field[console.mino_posy + y][console.mino_posx + x] = true;
          } else {
            // field[console.mino_posy + y][console.mino_posx + x] = false;
          }
        }
      }

      //draw_field
      for y in 1..HEIGHT {
        for x in 1..WIDTH {
          if field[y][x] == true {
            write!(stdout, "{}", CHAR_WALL);
          } else {
            write!(stdout, "{}", CHAR_EMPTY);
          }
        }
        // newsline
        write!(stdout, "\r\n");
      }

      // if console.mino_posy < HEIGHT - MINO_HEIGHT - 1 {
      //   console.mino_posy += 1;
      // } else {
      //   console.mino_posy = 0;

      //   console.select_mino();
      //   console.new_mino(&mut mino);
      // }
      field = old_field.clone();

      // write!("{:?}", mino);

      // 最後にフラッシュする
      stdout.flush().unwrap();

      // wait
      thread::sleep(time::Duration::from_millis(FALL_INTERVAL_MS));
    }
  }
}
