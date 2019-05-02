extern crate termion;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate serde;

use termion::color;
use termion::style;
use termion::cursor;
use std::fmt;
use std::io::{Write, stdout, stdin};
use rand::{thread_rng, Rng};
use termion::screen::*;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::fs::File;
use bincode::{serialize, deserialize, Infinite};
use std::io::Read;
use std::num::Wrapping;

#[derive(Debug)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Slot {
    value: usize
}

macro_rules! write_buffer {
    ($w:expr, ) => (());
    
    ($w:expr, $($arg:tt)+) => {{
        if let Err(e) = write!($w, $($arg)+) {
            panic!("Error writing to buffer: {}", e);
        }
    }};
}


#[derive(Debug, Clone, Serialize, Deserialize)]
enum Score {
    Current(usize),
    Best(usize)
}

use Score::*;

impl Score {
    fn add(&self, value: usize) -> Score {
        match *self {
            Current(n) => Current(n + value),
            Best(n)    => Best(n + value)
        }
    }

    fn value(&self) -> usize {
        match *self {
            Current(n) | Best(n) => n
        }
    }

    fn footer(&self) -> String {
        let number = match *self {
            Current(n) => Math::center(n,  9),
            Best(n)    => Math::center(n, 10)
        };
    
        format!("{bc}{fc}{number}{r}",
                number=number,
                bc=Self::bc(),
                fc=Self::fc(),
                r=Draw::reset()
        )
    }
    
    fn bcolor() -> RGB {
        RGB {r: 187, g: 173, b: 160}
    }

    fn fcolor() -> RGB {
        RGB {r: 238, g: 228, b: 218}
    }
    
    fn bc() -> String {
        format!("{}", color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)))
    }
    
    fn fc() -> String {
        format!("{}", color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b)))
    }

    fn x(&self) -> u16 {
        match *self {
            Current(_) => Grid::x() + 15,
            Best(_)    => Grid::x() + 15 + 13
        }
    }

    fn y() -> u16 {
        // println!("Grid::y(): {:?}", Grid::y());
        // println!("wrapping: {:?}", (Wrapping(Grid::y()) - Wrapping(3)).0);
        // (Wrapping(Grid::y()) - Wrapping(3)).0
        Grid::y() - 3
    }
    
    fn c(&self) -> cursor::Right {
        cursor::Right(self.x())
    }

    fn header(&self) -> String {
        match *self {
            Current(_) => "  𝘀𝗰𝗼𝗿𝗲  ".to_string(),
            Best(_)    => "   𝗯𝗲𝘀𝘁   ".to_string(),
        }
    }
    
    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write_buffer!(buffer, "{}", Draw::reset_pos());
        write_buffer!(buffer, "{}", style::Bold);

        write_buffer!(buffer, "{}", cursor::Down(Self::y()));

        write_buffer!(buffer, 
            "{c}{bc}{fc}{header}{r}\r\n{c}{bc}{fc}{footer}{r}\r\n",
            c=self.c(),
            bc=Self::bc(),
            fc=Self::fc(),
            r=Draw::reset(),
            header=self.header(),
            footer=self.footer()
        );

        write_buffer!(buffer, "{}", style::Reset);
    }
}

// refactor file structers (check the rust gif project in ~/dev)
// show help box when pressing `?` with esc to close
// fix resizing and small screens
// add ascii cinema video to the repo
// play 2048 from terminal

trait Render {
    fn bcolor() -> RGB;
    fn fcolor() -> RGB;
    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>);
    fn c() -> cursor::Right;
    fn x() -> u16;
    fn y() -> u16;
}

struct Math;

impl Math {
    fn rand<T>(vec: &Vec<T>) -> &T {
        thread_rng().choose(vec).unwrap()
    }

    fn rand_n() -> usize {
        if thread_rng().next_f64() < 0.9 { 2 } else { 4 }
    }

    fn center(n: usize, len: usize) -> String {
        let n_str = n.to_string();

        if n_str.len() >= len {
            return n_str;
        }

        let delta = len - n_str.len();
        let half = (((delta/2) as f64).ceil()) as usize;

        let ls = " ".repeat(half);
        let rs = if delta % 2 == 0 {
            " ".repeat(half)
        } else {
            " ".repeat(half + 1)
        };

        format!("{}{}{}", ls, n_str, rs)
    }
    
    fn furthest(pos: usize, points: Vec<Slot>, direction: Direction) -> usize {
        let mut furthest = pos as i32;
                
        let (range, step) = 
            match direction {
                UP | LEFT => {
                    ((0..pos).rev().collect::<Vec<usize>>(), -1)
                },
                DOWN | RIGHT => {
                    (((pos+1)..4).collect::<Vec<usize>>(), 1)
                }
            };
        
        for t in range {
            if points[t].is_empty() {
                furthest += step;
            } else {
                break;
            }
        }

        furthest as usize
    }
}


impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for Slot {
    fn eq(&self, other: &Slot) -> bool {
        self.value == other.value
    }
}

impl Slot {
    fn empty() -> Slot {
        Slot {value: 0}
    }

    fn is_empty(&self) -> bool {
        self.value == 0
    }

    fn new(n: usize) -> Slot {
        Slot {value: n}
    }

    fn bcolor(&self, status: &Status) -> RGB {
        match *status {
            OnGoing =>
                match self.value {
                       0 => RGB {r: 205, g: 193, b: 181},
                       2 => RGB {r: 238, g: 228, b: 218},
                       4 => RGB {r: 237, g: 224, b: 200},
                       8 => RGB {r: 242, g: 177, b: 121},
                      16 => RGB {r: 245, g: 149, b:  99},
                      32 => RGB {r: 246, g: 124, b:  95},
                      64 => RGB {r: 246, g:  94, b:  59},
                     128 => RGB {r: 237, g: 207, b: 114},
                     256 => RGB {r: 237, g: 204, b:  97},
                     512 => RGB {r: 237, g: 200, b:  80},
                    1024 => RGB {r: 237, g: 197, b:  63},
                    2048 => RGB {r: 237, g: 194, b:  46},
                       _ => RGB {r:  60, g:  58, b:  50},
                },
            Over =>
                match self.value {
                       2 => RGB {r: 238, g: 228, b: 219},
                       4 => RGB {r: 238, g: 227, b: 214},
                       8 => RGB {r: 239, g: 214, b: 194},
                      16 => RGB {r: 240, g: 206, b: 188},
                      32 => RGB {r: 240, g: 199, b: 186},
                      64 => RGB {r: 240, g: 191, b: 178},
                     128 => RGB {r: 238, g: 222, b: 192},
                     256 => RGB {r: 238, g: 221, b: 188},
                     512 => RGB {r: 238, g: 220, b: 184},
                    1024 => RGB {r: 238, g: 219, b: 180},
                    2048 => RGB {r: 238, g: 218, b: 177},
                    _ => RGB {r: 190, g: 181, b: 173},
                },
            Won =>
                match self.value {
                    0 => RGB {r: 220, g: 193, b: 122},
                    2 => RGB {r: 237, g: 211, b: 141},
                    4 => RGB {r: 236, g: 209, b: 132},
                    8 => RGB {r: 238, g: 185, b:  94},
                   16 => RGB {r: 239, g: 171, b:  84},
                   32 => RGB {r: 240, g: 159, b:  81},
                   64 => RGB {r: 240, g: 144, b:  65},
                  128 => RGB {r: 236, g: 200, b:  92},
                  256 => RGB {r: 236, g: 198, b:  84},
                  512 => RGB {r: 236, g: 196, b:  77},
                 1024 => RGB {r: 236, g: 195, b:  70},
                 2048 => RGB {r: 236, g: 193, b:  64},
                    _ => RGB {r: 148, g: 126, b:  57},
                }
        }
    }

    fn fcolor(&self, status: &Status) -> RGB {
        match *status {
            OnGoing => 
                match self.value {
                    0  => RGB {r: 205, g: 193, b: 181},
                 2 | 4 => RGB {r: 118, g: 110, b: 101},
                    _  => RGB {r: 249, g: 246, b: 242},
                },
            Over =>
                match self.value {
                    2 | 4 => RGB {r: 206, g: 195, b: 187},
                       _  => RGB {r: 241, g: 233, b: 226},
                },
            Won =>
                match self.value {
                    0     => RGB {r: 220, g: 193, b: 122},
                    2 | 4 => RGB {r: 177, g: 152, b:  82},
                       _  => RGB {r: 242, g: 220, b: 153},
                }
        }
    }

    fn bc(&self, status: &Status) -> String {
        format!("{}", color::Bg(color::Rgb(self.bcolor(status).r, self.bcolor(status).g, self.bcolor(status).b)))
    }

    fn fc(&self, status: &Status) -> String {
        format!("{}", color::Fg(color::Rgb(self.fcolor(status).r, self.fcolor(status).g, self.fcolor(status).b)))
    }
    
    fn format(&self) -> String {
        Math::center(self.value, 7).replace("0", "𝟎").replace("1", "𝟏").replace("2", "𝟐")
        .replace("3", "𝟑").replace("4", "𝟒").replace("5", "𝟓").replace("6", "𝟔")
        .replace("7", "𝟕").replace("8", "𝟖").replace("9", "𝟗")
    }
    
    fn header(&self, status: &Status) -> String {
        format!("{bc}       {r}",
        bc=self.bc(status),
        r=Draw::bg_r(),
        )
    }

    fn body(&self, status: &Status) -> String {
        format!("{bc}{fc}{n}{r}",
        bc=self.bc(status),
        fc=self.fc(status),
        r=Draw::reset(),
        n=self.format()
        )
    }

    fn footer(&self, status: &Status) -> String {
        self.header(status)
    }
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

use Direction::*;

struct Draw;

impl Draw {
    fn reset() -> String {
        format!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset))
    }
    
    fn bg_r() -> String {
        format!("{}", color::Bg(color::Reset))
    }

    fn reset_pos() -> String {
        format!("{}", cursor::Goto(1, 1))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    lines: Vec<Vec<Slot>>,
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        for y in 0..4 {
            for x in 0..4 {
                if self.lines[y][x] != other.lines[y][x] {
                    return false
                }
            }
        }

        true
    }
}

impl State {
    fn new() -> State {
        let mut state = State {
            lines: State::empty(),
        };

        state = state.add_random_slot();
        state = state.add_random_slot();

        state
    }

    fn empty() -> Vec<Vec<Slot>> {
        let mut lines : Vec<Vec<Slot>> = Vec::new();

        for _ in 0..4 {
            let mut line : Vec<Slot> = Vec::new();
            for _ in 0..4 {
                line.push(Slot::empty());
            }
            lines.push(line);
        }

        lines
    }

    fn zeros(&self) -> Vec<(usize, usize)> {
        let mut zeros : Vec<(usize, usize)> = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                if self.lines[y][x].is_empty() {
                    zeros.push((y, x));
                }
            }
        }

        zeros
    }

    fn add_random_slot(self) -> State {
        let zeros = self.zeros();

        if zeros.len() == 0 {
            return self;
        }

        let mut lines = self.lines;

        let rnd_index = Math::rand(&zeros);
        let rnd_n     = Math::rand_n();

        lines[rnd_index.0][rnd_index.1] = Slot::new(rnd_n);

        State {
            lines: lines,
        }
    }

    fn moves(self, direction: Direction) -> (State, usize) {
        match direction {
            Direction::UP => self.moves_up(),
            Direction::DOWN => self.moves_down(),
            Direction::LEFT => self.moves_left(),
            Direction::RIGHT => self.moves_right(),
        }
    }


    fn moves_up(self) -> (State, usize) {
        let mut lines = self.lines;
        let mut added_score = 0;

        for y in 0..4 {
            for x in 0..4 {
                if lines[y][x].is_empty() {
                    continue;
                }

                let cols = State::get_cols_from_lines(&lines);
                let furthest_up = Math::furthest(y, cols[x].clone(), UP);
                if furthest_up == y {
                    continue;
                }

                lines[furthest_up][x] = lines[y][x].clone();
                lines[y][x] = Slot::empty();
            }
        }

        for y in 0..4 {

            if y == 0 {
                continue;
            }

            for x in 0..4 {
                if lines[y][x].is_empty() {
                    continue;
                }

                if lines[y - 1][x] == lines[y][x] {
                    lines[y - 1][x] = Slot::new(lines[y][x].value * 2);
                    added_score += lines[y - 1][x].value;
                    for t in (y)..4 {
                        if t == 3 {
                            lines[t][x] = Slot::empty();
                        } else {
                            lines[t][x] = lines[t + 1][x].clone();
                        }
                    }
                }
            }
        }

        (State {lines: lines}, added_score)
    }

    fn moves_down(self) -> (State, usize) {
        let mut lines = self.lines;
        let mut added_score = 0;

        for y in (0..4).rev() {
            for x in 0..4 {
                if lines[y][x].is_empty() {
                    continue;
                }

                let cols = State::get_cols_from_lines(&lines);

                let furthest_down = Math::furthest(y, cols[x].clone(), DOWN);
                if furthest_down == y {
                    continue;
                }

                lines[furthest_down][x] = lines[y][x].clone();
                lines[y][x] = Slot::empty();
            }
        }

        for y in (0..4).rev() {

            if y == 0 {
                continue;
            }

            for x in 0..4 {
                if lines[y][x].is_empty() {
                    continue;
                }

                if lines[y - 1][x] == lines[y][x] {
                    lines[y][x] = Slot::new(lines[y - 1][x].value * 2);
                    added_score += lines[y][x].value;

                    for t in (0..y).rev() {
                        if t == 0 {
                            lines[t][x] = Slot::empty();
                        } else {
                            lines[t][x] = lines[t - 1][x].clone();
                        }
                    }
                }
            }
        }

        (State {lines: lines}, added_score)
    }

    fn moves_left(self) -> (State, usize) {
        let mut lines = self.lines;
        let mut added_score = 0;

        for y in 0..4 {
            for x in 0..4 {
                if lines[y][x].is_empty() {
                    continue;
                }

                let furthest = Math::furthest(x, lines[y].clone(), LEFT);
                if furthest == x {
                    continue;
                }

                lines[y][furthest] = lines[y][x].clone();
                lines[y][x] = Slot::empty();
            }
        }

        for x in 0..4 {

            if x == 3 {
                continue;
            }

            for y in 0..4 {

                if lines[y][x].is_empty() {
                    continue;
                }

                if lines[y][x + 1] == lines[y][x] {
                    lines[y][x] = Slot::new(lines[y][x + 1].value * 2);
                    added_score += lines[y][x].value;

                    for t in (x+1)..4 {
                        if t == 3 {
                            lines[y][t] = Slot::empty();
                        } else {
                            lines[y][t] = lines[y][t + 1].clone();
                        }
                    }
                }
            }
        }

        (State {lines: lines}, added_score)
    }

    fn moves_right(self) -> (State, usize) {
        let mut lines = self.lines;
        let mut added_score = 0;

        for y in 0..4 {
            for x in (0..4).rev() {
                if lines[y][x].is_empty() {
                    continue;
                }

                let furthest = Math::furthest(x, lines[y].clone(), RIGHT);
                if furthest == x {
                    continue;
                }
                lines[y][furthest] = lines[y][x].clone();
                lines[y][x] = Slot::empty();
            }
        }

        for x in (0..4).rev() {
            if x == 0 {
                continue;
            }

            for y in 0..4 {

                if lines[y][x].is_empty() {
                    continue;
                }

                if lines[y][x - 1] == lines[y][x] {
                    lines[y][x] = Slot::new(lines[y][x - 1].value * 2);
                    added_score += lines[y][x].value;
                    for t in (0..x).rev() {
                        if t == 0 {
                            lines[y][t] = Slot::empty();
                        } else {
                            lines[y][t] = lines[y][t - 1].clone();
                        }
                    }
                }
            }
        }

        (State {lines: lines}, added_score)
    }

    fn get_cols_from_lines(lines: &Vec<Vec<Slot>>) -> Vec<Vec<Slot>> {
        let mut cols : Vec<Vec<Slot>> = Vec::new();

        for x in 0..4 {
            cols.push(vec![lines[0][x], lines[1][x], lines[2][x], lines[3][x]])
        }

        cols
    }

    fn is_over(&self) -> bool {
        let up = self.clone().moves_up();
        let down = self.clone().moves_down();
        let left = self.clone().moves_left();
        let right = self.clone().moves_right();

        up == down &&
        left == right &&
        up == left
    }
    
    fn is_won(&self) -> bool {
        for y in 0..4 {
            for x in 0..4 {
                if self.lines[y][x].value == 2048 {
                    return true;
                }
            }
        }
        
        return false;
    }

    fn status(&self) -> Status {
        if self.is_won() {
            Status::Won
        } else if self.is_over() {
            Status::Over
        } else {
            Status::OnGoing
        }
    }

    fn handle_move(self, direction: Direction) -> (State, usize) {
        let old_state = self.clone();
        let (mut new_state, current_score) = self.moves(direction);

        if new_state != old_state {
            new_state = new_state.add_random_slot();
        }

        (new_state, current_score)
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>, status: Status) {
        match status {
            Status::OnGoing => Grid::new(self.clone(), OnGoing).render(buffer),
            Status::Won => {
                Grid::new(self.clone(), Won).render(buffer);
                OverLap::Won.render(buffer);
            },
            Status::Over => {
                Grid::new(self.clone(), Over).render(buffer);
                OverLap::Over.render(buffer);
            },
        }
    }
}

enum OverLap {
    Over,
    Won
}

impl OverLap {
    fn x(&self) -> u16 {
        match *self {
            OverLap::Over => Grid::x() + 15,
            OverLap::Won  => Grid::x() + 13
        }
    }
    
    fn y() -> u16 {
        Grid::y() + 5
    }
    
    fn bcolor(&self) -> RGB {
        match *self {
            OverLap::Over => Grid::border_color(&Over),
            OverLap::Won  => Grid::border_color(&Won)
        }
    }
    
    fn fcolor(&self) -> RGB {
        match *self {
            OverLap::Over => RGB {r: 119, g: 110, b: 101},
            OverLap::Won  => RGB {r: 249, g: 246, b: 241}
        }
    }
    
    fn c(&self) -> cursor::Right {
        match *self {
            OverLap::Over => Grid::c(),
            OverLap::Won  => cursor::Right(self.x() - 4)
        }
    }
    
    fn bc(&self) -> String {
        format!("{}", color::Bg(color::Rgb(self.bcolor().r, self.bcolor().g, self.bcolor().b)))
    }

    fn fc(&self) -> String {
        format!("{}", color::Fg(color::Rgb(self.fcolor().r, self.fcolor().g, self.fcolor().b)))
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write_buffer!(buffer, "{}", self.bc());
        write_buffer!(buffer, "{}", self.fc());
        write_buffer!(buffer, "{}", cursor::Goto(self.x(), Self::y()));
        write_buffer!(buffer, "{}", style::Bold);
        
        match *self {
            OverLap::Over => {
                write_buffer!(buffer, "𝗚𝗔𝝡𝗘 𝗢𝗩𝗘𝗥!");
                write_buffer!(buffer, "\r\n\r\n\r\n\r\n{c}        𝗣𝗿𝗲𝘀𝘀 `𝗿` 𝘁𝗼 𝗽𝗹𝗮𝘆 𝗮𝗴𝗮𝗶𝗻.", c=self.c());
            },
            OverLap::Won  => {
                write_buffer!(buffer, "🎊  You Win! 🎊");
                write_buffer!(
                    buffer,
                    "\r\n\r\n\r\n\r\n{c}{f}{b}Keep going{rf}{rb}: press `𝗰`\r\n\r\n\r\n\r\n{c}{f}{b}Try again{rf}{rb}: press `𝗿`",
                    c=self.c(), b=color::Bg(color::Rgb(142, 121, 103)), f=color::Bg(color::Rgb(249, 246, 242)),
                    rb=self.bc(),
                    rf=self.fc()
                );
            }
        }
        
        write_buffer!(buffer, "{}", Draw::reset());
        write_buffer!(buffer, "{}", style::Reset);
    }
}

struct Grid {
    state: State,
    status: Status
}

impl Grid {
    fn new(state: State, status: Status) -> Grid {
        Grid {
            state,
            status
        }
    }
    fn x() -> u16 {
        (Wrapping(termion::terminal_size().unwrap().0) / Wrapping(2) - Wrapping(15)).0
    }

    fn y() -> u16 {
        (Wrapping(termion::terminal_size().unwrap().1) / Wrapping(2) - Wrapping(10)).0
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write_buffer!(buffer, "{}", Draw::reset_pos());

        write_buffer!(buffer, "{}", cursor::Down(Self::y()));
        write_buffer!(buffer, "{}", style::Bold);

        write_buffer!(buffer, "{}", self.body());

        write_buffer!(buffer, "{}", style::Reset);
    }

    fn c() -> cursor::Right {
        cursor::Right(Self::x())
    }

    fn body(&self) -> String {
        format!(
"{c}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}\r
{c}{b}{s00_h}{b}{s01_h}{b}{s02_h}{b}{s03_h}{b}\r
{c}{b}{s00_b}{b}{s01_b}{b}{s02_b}{b}{s03_b}{b}\r
{c}{b}{s00_f}{b}{s01_f}{b}{s02_f}{b}{s03_f}{b}\r
{c}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}\r
{c}{b}{s10_h}{b}{s11_h}{b}{s12_h}{b}{s13_h}{b}\r
{c}{b}{s10_b}{b}{s11_b}{b}{s12_b}{b}{s13_b}{b}\r
{c}{b}{s10_f}{b}{s11_f}{b}{s12_f}{b}{s13_f}{b}\r
{c}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}\r
{c}{b}{s20_h}{b}{s21_h}{b}{s22_h}{b}{s23_h}{b}\r
{c}{b}{s20_b}{b}{s21_b}{b}{s22_b}{b}{s23_b}{b}\r
{c}{b}{s20_f}{b}{s21_f}{b}{s22_f}{b}{s23_f}{b}\r
{c}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}\r
{c}{b}{s30_h}{b}{s31_h}{b}{s32_h}{b}{s33_h}{b}\r
{c}{b}{s30_b}{b}{s31_b}{b}{s32_b}{b}{s33_b}{b}\r
{c}{b}{s30_f}{b}{s31_f}{b}{s32_f}{b}{s33_f}{b}\r
{c}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}{b}\r"
, c=Self::c(), b=self.b(),
    s00_h=self.state.lines[0][0].header(&self.status), s01_h=self.state.lines[0][1].header(&self.status), s02_h=self.state.lines[0][2].header(&self.status), s03_h=self.state.lines[0][3].header(&self.status), 
    s00_b=self.state.lines[0][0].body(&self.status), s01_b=self.state.lines[0][1].body(&self.status), s02_b=self.state.lines[0][2].body(&self.status), s03_b=self.state.lines[0][3].body(&self.status), 
    s00_f=self.state.lines[0][0].footer(&self.status), s01_f=self.state.lines[0][1].footer(&self.status), s02_f=self.state.lines[0][2].footer(&self.status), s03_f=self.state.lines[0][3].footer(&self.status), 

    s10_h=self.state.lines[1][0].header(&self.status), s11_h=self.state.lines[1][1].header(&self.status), s12_h=self.state.lines[1][2].header(&self.status), s13_h=self.state.lines[1][3].header(&self.status), 
    s10_b=self.state.lines[1][0].body(&self.status), s11_b=self.state.lines[1][1].body(&self.status), s12_b=self.state.lines[1][2].body(&self.status), s13_b=self.state.lines[1][3].body(&self.status), 
    s10_f=self.state.lines[1][0].footer(&self.status), s11_f=self.state.lines[1][1].footer(&self.status), s12_f=self.state.lines[1][2].footer(&self.status), s13_f=self.state.lines[1][3].footer(&self.status), 

    s20_h=self.state.lines[2][0].header(&self.status), s21_h=self.state.lines[2][1].header(&self.status), s22_h=self.state.lines[2][2].header(&self.status), s23_h=self.state.lines[2][3].header(&self.status), 
    s20_b=self.state.lines[2][0].body(&self.status), s21_b=self.state.lines[2][1].body(&self.status), s22_b=self.state.lines[2][2].body(&self.status), s23_b=self.state.lines[2][3].body(&self.status), 
    s20_f=self.state.lines[2][0].footer(&self.status), s21_f=self.state.lines[2][1].footer(&self.status), s22_f=self.state.lines[2][2].footer(&self.status), s23_f=self.state.lines[2][3].footer(&self.status), 

    s30_h=self.state.lines[3][0].header(&self.status), s31_h=self.state.lines[3][1].header(&self.status), s32_h=self.state.lines[3][2].header(&self.status), s33_h=self.state.lines[3][3].header(&self.status), 
    s30_b=self.state.lines[3][0].body(&self.status), s31_b=self.state.lines[3][1].body(&self.status), s32_b=self.state.lines[3][2].body(&self.status), s33_b=self.state.lines[3][3].body(&self.status), 
    s30_f=self.state.lines[3][0].footer(&self.status), s31_f=self.state.lines[3][1].footer(&self.status), s32_f=self.state.lines[3][2].footer(&self.status), s33_f=self.state.lines[3][3].footer(&self.status), 
)
    }
    
    fn border_color(status: &Status) -> RGB {
        match *status {
            OnGoing => RGB {r: 187, g: 173, b: 161},
            Over => RGB {r: 224, g: 213, b: 203},
            Won => RGB {r: 211, g: 183, b: 112},
        }
    }

    fn b(&self) -> String {
        format!("{}  {}", color::Bg(color::Rgb(Self::border_color(&self.status).r, Self::border_color(&self.status).g, Self::border_color(&self.status).b)), Draw::bg_r())
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "".to_string();
        for y in 0..4 {
            for x in 0..4 {
                result.push_str(&format!("{}\t", self.lines[y][x]));
            }
            result.push_str(&format!("\n"));
        }

        write!(f, "{}", result)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum Status {
    OnGoing,
    Won,
    Over
}

use Status::*;

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    state: State,
    current_score: Score,
    best_score: Score,
    keep_going: bool
}

impl Game {
    fn new() -> Game {
        match Game::load() {
            Some(game) => game,
            None       => Game::empty()
        }
    }

    fn empty() -> Game {
        Game {
            state: State::new(),
            current_score: Score::Current(0),
            best_score: Score::Best(0),
            keep_going: false
        }
    }

    fn reset(&self) -> Game {
        let game = Game {
            state: State::new(),
            current_score: Score::Current(0),
            best_score: self.best_score.clone(),
            keep_going: false
        };

        game.save();

        game
    }

    fn status(&self) -> Status {
        match self.state.status() {
            Status::Won => {
                if self.keep_going {
                    if self.state.is_over() {
                        Status::Over
                    } else {
                        Status::OnGoing
                    }
                } else {
                    Status::Won
                }
            },
            status => status
        }
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write_buffer!(buffer, "{}", Draw::reset());
        write_buffer!(buffer, "{}", termion::clear::All);

        self.current_score.render(buffer);
        self.best_score.render(buffer);

        self.state.render(buffer, self.status());

        buffer.flush().unwrap();
    }
    
    fn handle_move(self, direction: Direction, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) -> Game {
        let (new_state, added_score) = self.state.clone().handle_move(direction);
        
        if new_state == self.state {
            return self.clone();
        }

        let current_score = self.current_score.add(added_score);

        let (current, best) = (current_score.value(), self.best_score.value());
        
        let best_score = if current > best {
            Score::Best(current)
        } else {
            self.best_score
        };

        let game = Game {
            state: new_state,
            current_score: current_score,
            best_score: best_score,
            keep_going: self.keep_going
        };

        game.save();

        game.render(buffer);

        game
    }

    fn save_file() -> String {
        "game.bin".to_string()
    }

    fn save(&self) {
        if let Ok(mut file) = File::create(Game::save_file()) {
            let encoded: Vec<u8> = serialize(&self, Infinite).unwrap();
            file.write_all(&encoded[..]).unwrap();
        }
    }

    fn load() -> Option<Game> {
        match File::open(Game::save_file()) {
            Ok(mut file) => {
                let mut encoded = Vec::new();
                match file.read_to_end(&mut encoded) {
                    Ok(_) => {
                        match deserialize(&encoded[..]) {
                            Ok(game) => Some(game),
                            Err(_)   => None
                        }
                    },
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }
    
    fn keep_going(&self) -> Game {
        Game {
            state: self.state.clone(),
            current_score: self.current_score.clone(),
            best_score: self.best_score.clone(),
            keep_going: true
        }
    }
}

fn main() {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let mut game = Game::new();

    write_buffer!(screen, "{}", ToAlternateScreen);
    write_buffer!(screen, "{}", cursor::Hide);

    if game.status() == Status::Over {
        game = game.reset();
    }
    
    game.render(&mut screen);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') | Key::Ctrl('c') => break,
            Key::Char('r') => {game = game.reset(); game.render(&mut screen);},
            Key::Char('c') => {game = game.keep_going(); game.render(&mut screen);},
            Key::Left  | Key::Char('a') => { game = game.handle_move(Direction::LEFT, &mut screen)},
            Key::Right | Key::Char('d') => { game = game.handle_move(Direction::RIGHT, &mut screen)},
            Key::Up    | Key::Char('w') => { game = game.handle_move(Direction::UP, &mut screen)},
            Key::Down  | Key::Char('s') => { game = game.handle_move(Direction::DOWN, &mut screen)},
            _              => {},
        }
    }

    write_buffer!(screen, "{}", termion::cursor::Show);
}
