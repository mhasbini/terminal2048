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

#[derive(Debug)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Slot {
    value: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentScore {
    value: usize
}

// refactor file structers (check the rust gif project in ~/dev)
// implement press c to continue
// show help box when pressing `?` with esc to close
// fix resizing and small screens

impl CurrentScore {
    fn new(value: usize) -> CurrentScore {
        CurrentScore {value}
    }

    fn get(&self) -> String {
        format!("{bc}{fc}{}{r}",
                Math::center(self.value, 9),
                bc=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
                fc=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b)),
                r=Draw::reset()
        )
    }

    fn add(&self, value: usize) -> CurrentScore {
        CurrentScore {value: self.value + value}
    }

    fn b(&self) -> String {
        // S Ś Ŝ Ş Š
        // 𝘀 𝘀 𝙨 𝚂 𝚂 𝘴 𝚜 𝙎 𝕾
        // 𝖘 𝔰 𝕊 𝐬 𝐒
        // 𝗼 𝗰 𝗲 𝗿
        format!(
"{c}{bc}{fc}  𝘀𝗰𝗼𝗿𝗲  {r}\r
{c}{bc}{fc}{}{r}\r\n",
        self.get(),
        c=Self::c(),
        bc=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
        fc=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b)),
        r=Draw::reset()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BestScore {
    value: usize
}

impl BestScore {
    fn new(value: usize) -> BestScore {
        BestScore {value}
    }

    fn get(&self) -> String {
        format!("{bc}{fc}{}{r}",
                Math::center(self.value, 10),
                bc=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
                fc=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b)),
                r=Draw::reset()
        )
    }

    fn b(&self) -> String {
        format!(
"{c}{bc}{fc}   𝗯𝗲𝘀𝘁   {r}\r
{c}{bc}{fc}{}{r}\r\n",
        self.get(),
        c=Self::c(),
        bc=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
        fc=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b)),
        r=Draw::reset()
        )
    }

}

trait Render {
    fn bcolor() -> Rgb;
    fn fcolor() -> Rgb;
    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>);
    fn c() -> cursor::Right;
    fn x() -> u16;
    fn y() -> u16;
}

impl Render for BestScore {
    fn x() -> u16 {
        CurrentScore::x() + 13
    }

    fn y() -> u16 {
        CurrentScore::y()
    }

    fn bcolor() -> Rgb {
        CurrentScore::bcolor()
    }

    fn fcolor() -> Rgb {
        CurrentScore::fcolor()
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset_pos()).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();

        write!(buffer, "{}", cursor::Down(Self::y())).unwrap();
        write!(buffer, "{}", self.b()).unwrap();

        write!(buffer, "{}", style::Reset).unwrap();
    }

    fn c() -> cursor::Right {
        cursor::Right(Self::x())
    }
}

impl Render for CurrentScore {
    fn x() -> u16 {
        State::x() + 15
    }

    fn y() -> u16 {
        State::y() - 3
    }

    fn bcolor() -> Rgb {
        Rgb {r: 187, g: 173, b: 160}
    }

    fn fcolor() -> Rgb {
        Rgb {r: 238, g: 228, b: 218}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset_pos()).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();

        write!(buffer, "{}", cursor::Down(Self::y())).unwrap();
        write!(buffer, "{}", self.b()).unwrap();

        write!(buffer, "{}", style::Reset).unwrap();
    }

    fn c() -> cursor::Right {
        cursor::Right(Self::x())
    }
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

    fn furthest_up(y: usize, col: Vec<Slot>) -> usize {
        let mut furthest = y;
        for t in (0..y).rev() {
            if col[t].is_empty() {
                furthest -= 1;
            } else {
                break;
            }
        }

        furthest
    }

    fn furthest_down(y: usize, col: Vec<Slot>) -> usize {
        let mut furthest = y;
        for t in (y+1)..4 {
            if col[t].is_empty() {
                furthest += 1;
            } else {
                break;
            }
        }

        furthest
    }

    fn furthest_left(x: usize, line: Vec<Slot>) -> usize {
        let mut furthest = x;
        for t in (0..x).rev() {
            if line[t].is_empty() {
                furthest -= 1;
            } else {
                break;
            }
        }

        furthest
    }

    fn furthest_right(x: usize, line: Vec<Slot>) -> usize {
        let mut furthest = x;
        for t in (x+1)..4 {
            if line[t].is_empty() {
                furthest += 1;
            } else {
                break;
            }
        }

        furthest
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

    fn color(&self) -> Rgb {
        match self.value {
               0 => Rgb {r: 205, g: 193, b: 181},
               2 => Rgb {r: 238, g: 228, b: 218},
               4 => Rgb {r: 237, g: 224, b: 200},
               8 => Rgb {r: 242, g: 177, b: 121},
              16 => Rgb {r: 245, g: 149, b:  99},
              32 => Rgb {r: 246, g: 124, b:  95},
              64 => Rgb {r: 246, g:  94, b:  59},
             128 => Rgb {r: 237, g: 207, b: 114},
             256 => Rgb {r: 237, g: 204, b:  97},
             512 => Rgb {r: 237, g: 200, b:  80},
            1024 => Rgb {r: 237, g: 197, b:  63},
            2048 => Rgb {r: 237, g: 194, b:  46},
               _ => Rgb {r:  60, g:  58, b:  50},
        }
    }

    fn fcolor(&self) -> Rgb {
        match self.value {
                0  => Rgb {r: 205, g: 193, b: 181},
             2 | 4 => Rgb {r: 118, g: 110, b: 101},
                _  => Rgb {r: 249, g: 246, b: 242},
        }
    }


    fn format_n(n: String) -> String {
        // 0 1 2 3 4 5 6 7 8 9
        // ０ １ ２ ３ ４ ５ ６ ７ ８ ９
        // 𝟎 𝟏 𝟐 𝟑 𝟒 𝟓 𝟔 𝟕 𝟖 𝟗
        // 𝟬 𝟭 𝟮 𝟯 𝟰 𝟱 𝟲 𝟳 𝟴 𝟵
        // ⑴ ⑵ ⑶ ⑷ ⑸ ⑹ ⑺ ⑻ ⑼

        // self.value.to_string().replace("0", "０").replace("1", "１").replace("2", "２")
        // .replace("3", "３").replace("4", "４").replace("5", "５").replace("6", "６")
        // .replace("7", "７").replace("8", "８").replace("9", "９")

        n.replace("0", "𝟎").replace("1", "𝟏").replace("2", "𝟐")
        .replace("3", "𝟑").replace("4", "𝟒").replace("5", "𝟓").replace("6", "𝟔")
        .replace("7", "𝟕").replace("8", "𝟖").replace("9", "𝟗")
    }

    fn get_h(&self) -> String {
        format!("{bc}{fc}┏━━━━━┓{r}",
        bc=self.bc(),
        fc=self.fc(),
        r=Draw::reset(),
        )
    }

    fn get_b(&self) -> String {
        format!("{bc}{fc}┃{r}{bc}{ifc}{n}{r}{bc}{fc}┃{r}",
        bc=self.bc(),
        fc=self.fc(),
        ifc=self.ifc(),
        r=Draw::reset(),
        n=Slot::format_n(Math::center(self.value, 5))
        )
    }

    fn bc(&self) -> String {
        format!("{}", color::Bg(color::Rgb(self.color().r, self.color().g, self.color().b)))
    }

    fn fc(&self) -> String {
        format!("{}", color::Fg(color::Rgb(self.color().r, self.color().g, self.color().b)))
    }
    
    fn ifc(&self) -> String {
        format!("{}", color::Fg(color::Rgb(self.fcolor().r, self.fcolor().g, self.fcolor().b)))
    }

    fn get_f(&self) -> String {
        format!("{bc}{fc}┗━━━━━┛{r}",
        bc=self.bc(),
        fc=self.fc(),
        r=Draw::reset()
        )
    }
    
    fn bc_trans(&self) -> String {
        format!("{}", color::Bg(color::Rgb(self.bcolor_trans().r, self.bcolor_trans().g, self.bcolor_trans().b)))
    }

    fn fc_trans(&self) -> String {
        format!("{}", color::Fg(color::Rgb(self.fcolor_trans().r, self.fcolor_trans().g, self.fcolor_trans().b)))
    }

    fn bcolor_trans(&self) -> Rgb {
        match self.value {
               2 => Rgb {r: 238, g: 238, b: 219},
               4 => Rgb {r: 238, g: 227, b: 214},
               8 => Rgb {r: 239, g: 214, b: 194},
              16 => Rgb {r: 240, g: 206, b: 188},
              32 => Rgb {r: 240, g: 199, b: 186},
              64 => Rgb {r: 240, g: 191, b: 178},
             128 => Rgb {r: 238, g: 222, b: 192},
             256 => Rgb {r: 238, g: 221, b: 188},
             512 => Rgb {r: 238, g: 220, b: 184},
            1024 => Rgb {r: 238, g: 219, b: 180},
            2048 => Rgb {r: 238, g: 218, b: 177},
               _ => Rgb {r: 190, g: 181, b: 173},
        }
    }

    fn fcolor_trans(&self) -> Rgb {
        match self.value {
             2 | 4 => Rgb {r: 206, g: 195, b: 187},
                _  => Rgb {r: 241, g: 233, b: 226},
        }
    }

    fn trans_h(&self) -> String {
        format!("{bc}       {r}",
        bc=self.bc_trans(),
        r=Draw::bg_r(),
        )
    }

    fn trans_b(&self) -> String {
        format!("{bc}{fc}{n}{r}",
        bc=self.bc_trans(),
        fc=self.fc_trans(),
        r=Draw::reset(),
        n=Slot::format_n(Math::center(self.value, 7))
        )
    }

    fn trans_f(&self) -> String {
        self.trans_h()
    }

    fn won_bc_trans(&self) -> String {
        format!("{}", color::Bg(color::Rgb(self.bcolor_trans_won().r, self.bcolor_trans_won().g, self.bcolor_trans_won().b)))
    }

    fn won_fc_trans(&self) -> String {
        format!("{}", color::Fg(color::Rgb(self.fcolor_trans_won().r, self.fcolor_trans_won().g, self.fcolor_trans_won().b)))
    }
    
    fn bcolor_trans_won(&self) -> Rgb {
        match self.value {
               0 => Rgb {r: 220, g: 193, b: 122},
               2 => Rgb {r: 237, g: 211, b: 141},
               4 => Rgb {r: 236, g: 209, b: 132},
               8 => Rgb {r: 238, g: 185, b:  94},
              16 => Rgb {r: 239, g: 171, b:  84},
              32 => Rgb {r: 240, g: 159, b:  81},
              64 => Rgb {r: 240, g: 144, b:  65},
             128 => Rgb {r: 236, g: 200, b:  92},
             256 => Rgb {r: 236, g: 198, b:  84},
             512 => Rgb {r: 236, g: 196, b:  77},
            1024 => Rgb {r: 236, g: 195, b:  70},
            2048 => Rgb {r: 236, g: 193, b:  64},
               _ => Rgb {r: 148, g: 126, b:  57},
        }
    }

    fn fcolor_trans_won(&self) -> Rgb {
        match self.value {
            0     => Rgb {r: 220, g: 193, b: 122},
            2 | 4 => Rgb {r: 177, g: 152, b:  82},
               _  => Rgb {r: 242, g: 220, b: 153},
        }
    }

    fn won_trans_h(&self) -> String {
        format!("{bc}       {r}",
        bc=self.won_bc_trans(),
        r=Draw::bg_r(),
        )
    }

    fn won_trans_b(&self) -> String {
        format!("{bc}{fc}{n}{r}",
        bc=self.won_bc_trans(),
        fc=self.won_fc_trans(),
        r=Draw::reset(),
        n=Slot::format_n(Math::center(self.value, 7))
        )
    }

    fn won_trans_f(&self) -> String {
        self.won_trans_h()
    }
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

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
    // score: CurrentScore,
    // buffer: termion::raw::RawTerminal<std::io::Stdout>
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        // self.value == other.value
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
                let furthest_up = Math::furthest_up(y, cols[x].clone());
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
                    for t in (y+1)..4 {
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

                let furthest_down = Math::furthest_down(y, cols[x].clone());
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

                let furthest = Math::furthest_left(x, lines[y].clone());
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

                let furthest = Math::furthest_right(x, lines[y].clone());
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

    fn is_done(&self) -> bool {
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
        } else if self.is_done() {
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

    fn bc(&self) -> String {
        format!("{}", color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)))
    }

    fn fc(&self) -> String {
        format!("{}", color::Fg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)))
    }

    fn print_box(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{c}{bc}{fc}╔════════╦════════╦════════╦═════════╗{r}\r\n", c=Self::c(), bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();

        for y in 0..4 {
            for x in 0..4 {
                if x == 0 {
                    write!(buffer, "{c}{bc} {r}", c=Self::c(), bc=self.bc(), r=Draw::reset()).unwrap();
                }

                write!(buffer, "{bc}{fc}║{r}{sh}{bc} {r}", bc=self.bc(), fc=self.fc(), r=Draw::reset(), sh=self.lines[y][x].get_h()).unwrap();

                if x == 3 {
                    write!(buffer, "{bc}{fc}║{r}", bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();
                }
            }

            write!(buffer, "\r\n").unwrap();

            for x in 0..4 {
                if x == 0 {
                    write!(buffer, "{c}{bc} {r}", c=Self::c(), bc=self.bc(), r=Draw::reset()).unwrap();
                }

                write!(buffer, "{bc}{fc}║{r}{sb}{bc} {r}", bc=self.bc(), fc=self.fc(), r=Draw::reset(), sb=self.lines[y][x].get_b()).unwrap();

                if x == 3 {
                    write!(buffer, "{bc}{fc}║{r}", bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();
                }
            }

            write!(buffer, "\r\n").unwrap();

            for x in 0..4 {
                if x == 0 {
                    write!(buffer, "{c}{bc} {r}", c=Self::c(), bc=self.bc(), r=Draw::reset()).unwrap();
                }

                write!(buffer, "{bc}{fc}║{r}{sf}{bc} {}", bc=self.bc(), fc=self.fc(), r=Draw::reset(), sf=self.lines[y][x].get_f()).unwrap();

                if x == 3 {
                    write!(buffer, "{bc}{fc}║{r}", bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();
                }
            }

            if y == 3 {
                write!(buffer, "\r\n{c}{bc}{fc}╚═════════╩════════╩═══════╩═════════╝{r}\r\n", c=Self::c(), bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();
            } else {
                write!(buffer, "\r\n{c}{bc}{fc}╠═════════╬════════╬═══════╬═════════╣{r}\r\n", c=Self::c(), bc=self.bc(), fc=self.fc(), r=Draw::reset()).unwrap();
            }
        }
    }
}

struct GameOverScreen {state: State}

impl GameOverScreen {
    fn new(state: State) -> GameOverScreen {
        GameOverScreen {state}
    }
    
    fn border_color() -> Rgb {
        Rgb {r: 224, g: 213, b: 203}
    }

    fn b() -> String {
        format!("{}  {}", color::Bg(color::Rgb(Self::border_color().r, Self::border_color().g, Self::border_color().b)), Draw::bg_r())
    }
    
    fn body(state: State) -> String {
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
, c=Self::c(), b=Self::b(),
    s00_h=state.lines[0][0].trans_h(), s01_h=state.lines[0][1].trans_h(), s02_h=state.lines[0][2].trans_h(), s03_h=state.lines[0][3].trans_h(), 
    s00_b=state.lines[0][0].trans_b(), s01_b=state.lines[0][1].trans_b(), s02_b=state.lines[0][2].trans_b(), s03_b=state.lines[0][3].trans_b(), 
    s00_f=state.lines[0][0].trans_f(), s01_f=state.lines[0][1].trans_f(), s02_f=state.lines[0][2].trans_f(), s03_f=state.lines[0][3].trans_f(), 

    s10_h=state.lines[1][0].trans_h(), s11_h=state.lines[1][1].trans_h(), s12_h=state.lines[1][2].trans_h(), s13_h=state.lines[1][3].trans_h(), 
    s10_b=state.lines[1][0].trans_b(), s11_b=state.lines[1][1].trans_b(), s12_b=state.lines[1][2].trans_b(), s13_b=state.lines[1][3].trans_b(), 
    s10_f=state.lines[1][0].trans_f(), s11_f=state.lines[1][1].trans_f(), s12_f=state.lines[1][2].trans_f(), s13_f=state.lines[1][3].trans_f(), 

    s20_h=state.lines[2][0].trans_h(), s21_h=state.lines[2][1].trans_h(), s22_h=state.lines[2][2].trans_h(), s23_h=state.lines[2][3].trans_h(), 
    s20_b=state.lines[2][0].trans_b(), s21_b=state.lines[2][1].trans_b(), s22_b=state.lines[2][2].trans_b(), s23_b=state.lines[2][3].trans_b(), 
    s20_f=state.lines[2][0].trans_f(), s21_f=state.lines[2][1].trans_f(), s22_f=state.lines[2][2].trans_f(), s23_f=state.lines[2][3].trans_f(), 

    s30_h=state.lines[3][0].trans_h(), s31_h=state.lines[3][1].trans_h(), s32_h=state.lines[3][2].trans_h(), s33_h=state.lines[3][3].trans_h(), 
    s30_b=state.lines[3][0].trans_b(), s31_b=state.lines[3][1].trans_b(), s32_b=state.lines[3][2].trans_b(), s33_b=state.lines[3][3].trans_b(), 
    s30_f=state.lines[3][0].trans_f(), s31_f=state.lines[3][1].trans_f(), s32_f=state.lines[3][2].trans_f(), s33_f=state.lines[3][3].trans_f(), 
)
    }
}

impl Render for GameOverScreen {
    fn x() -> u16 {
        State::x()
    }

    fn y() -> u16 {
        State::y()
    }

    fn c() -> cursor::Right {
        State::c()
    }

    fn bcolor() -> Rgb {
        Rgb {r: 250, g: 248, b: 239}
    }

    fn fcolor() -> Rgb {
        Rgb {r: 119, g: 110, b: 101}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset_pos()).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();
        write!(buffer, "{}", cursor::Down(Self::y())).unwrap();

        write!(buffer, "{}", Self::body(self.state.clone())).unwrap();

        write!(buffer, "{}", style::Reset).unwrap();
    }
}

struct GameOverWord {state: State}

impl GameOverWord {
    fn new(state: State) -> GameOverWord {
        GameOverWord {state}
    }
}

impl Render for GameOverWord {
    fn x() -> u16 {
        GameOverScreen::x() + 15
    }

    fn y() -> u16 {
        GameOverScreen::y() + 5
    }

    fn c() -> cursor::Right {
        GameOverScreen::c()
    }

    fn bcolor() -> Rgb {
        GameOverScreen::border_color()
    }

    fn fcolor() -> Rgb {
        Rgb {r: 119, g: 110, b: 101}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        GameOverScreen::new(self.state.clone()).render(buffer);
        
        write!(buffer, "{}", color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b))).unwrap();
        write!(buffer, "{}", color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b))).unwrap();
        write!(buffer, "{}", cursor::Goto(Self::x(), Self::y())).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();

        write!(buffer, "𝗚𝗔𝝡𝗘 𝗢𝗩𝗘𝗥!").unwrap();
        // 𝗣𝗿𝗲𝘀𝘀 `̀̀```｀𝗿` 𝘁𝗼 𝗽𝗹𝗮𝘆 𝗮𝗴𝗮𝗶𝗻．﹒․.
        write!(buffer, "\r\n\r\n\r\n\r\n{c}        𝗣𝗿𝗲𝘀𝘀 `𝗿` 𝘁𝗼 𝗽𝗹𝗮𝘆 𝗮𝗴𝗮𝗶𝗻.", c=Self::c()).unwrap();
        write!(buffer, "{}", Draw::reset()).unwrap();
        write!(buffer, "{}", style::Reset).unwrap();
    }
}

struct GameWonWord {state: State}

impl GameWonWord {
    fn new(state: State) -> GameWonWord {
        GameWonWord {state}
    }
}

impl Render for GameWonWord {
    fn x() -> u16 {
        GameWonScreen::x() + 13
    }

    fn y() -> u16 {
        GameWonScreen::y() + 5
    }

    fn c() -> cursor::Right {
        cursor::Right(Self::x() - 4)
    }

    fn bcolor() -> Rgb {
        GameWonScreen::border_color()
    }

    fn fcolor() -> Rgb {
        Rgb {r: 249, g: 246, b: 241}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        GameWonScreen::new(self.state.clone()).render(buffer);
        
        write!(buffer, "{}", color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b))).unwrap();
        write!(buffer, "{}", color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b))).unwrap();
        write!(buffer, "{}", cursor::Goto(Self::x(), Self::y())).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();
        // 𝗬𝗼𝘂 𝘄𝗶𝗻
        // 𝗞𝗲𝗲𝗽 𝗴𝗼𝗶𝗻𝗴 𝗧𝗿𝘆 𝗮𝗴𝗮𝗶𝗻 
        // 𝗔𝗕𝗖𝗗𝗮𝗯𝗰𝗱𝗲𝗳𝗴𝗩𝗨𝗬𝗧𝘀𝘁𝘂𝘃𝗸
        // & ﹠ ＆
        write!(buffer, "🎊  You Win! 🎊").unwrap();
        // 𝗣𝗿𝗲𝘀𝘀 `̀̀```｀𝗿` 𝘁𝗼 𝗽𝗹𝗮𝘆 𝗮𝗴𝗮𝗶𝗻．﹒․.
        // write!(buffer, "\r\n\r\n\r\n\r\n{c}  𝗣𝗿𝗲𝘀𝘀: `𝗰` 𝘁𝗼 𝗸𝗲𝗲𝗽 𝗴𝗼𝗶𝗻𝗴. ｀𝗿` 𝘁𝗼 𝗽𝗹𝗮𝘆 𝗮𝗴𝗮𝗶𝗻.", c=Self::c()).unwrap();
        // 142	121	103	
        // write!(buffer, "\r\n\r\n\r\n\r\n{c}{f}{b}𝗞𝗲𝗲𝗽 𝗴𝗼𝗶𝗻𝗴{rf}{rb}: 𝗽𝗿𝗲𝘀𝘀 `𝗰`",
        write!(buffer, "\r\n\r\n\r\n\r\n{c}{f}{b}Keep going{rf}{rb}: press `𝗰`",
            c=Self::c(), b=color::Bg(color::Rgb(142, 121, 103)), f=color::Bg(color::Rgb(249, 246, 242)),
            rb=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
            rf=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b))
        ).unwrap();
        // write!(buffer, "\r\n\r\n\r\n\r\n{c}{f}{b}𝗧𝗿𝘆 𝗮𝗴𝗮𝗶𝗻{rf}{rb}: 𝗽𝗿𝗲𝘀𝘀 `𝗿`",
        write!(buffer, "\r\n\r\n\r\n\r\n{c}{f}{b}Try again{rf}{rb}: press `𝗿`",
            c=Self::c(), b=color::Bg(color::Rgb(142, 121, 103)), f=color::Bg(color::Rgb(249, 246, 242)),
            rb=color::Bg(color::Rgb(Self::bcolor().r, Self::bcolor().g, Self::bcolor().b)),
            rf=color::Fg(color::Rgb(Self::fcolor().r, Self::fcolor().g, Self::fcolor().b))
        ).unwrap();

        write!(buffer, "{}", Draw::reset()).unwrap();
        write!(buffer, "{}", style::Reset).unwrap();
    }
}

struct GameWonScreen {state: State}

impl GameWonScreen {
    fn new(state: State) -> GameWonScreen {
        GameWonScreen {state}
    }
    
    fn border_color() -> Rgb {
        Rgb {r: 211, g: 183, b: 112}
    }

    fn b() -> String {
        format!("{}  {}", color::Bg(color::Rgb(Self::border_color().r, Self::border_color().g, Self::border_color().b)), Draw::bg_r())
    }
    
    fn body(state: State) -> String {
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
, c=Self::c(), b=Self::b(),
    s00_h=state.lines[0][0].won_trans_h(), s01_h=state.lines[0][1].won_trans_h(), s02_h=state.lines[0][2].won_trans_h(), s03_h=state.lines[0][3].won_trans_h(), 
    s00_b=state.lines[0][0].won_trans_b(), s01_b=state.lines[0][1].won_trans_b(), s02_b=state.lines[0][2].won_trans_b(), s03_b=state.lines[0][3].won_trans_b(), 
    s00_f=state.lines[0][0].won_trans_f(), s01_f=state.lines[0][1].won_trans_f(), s02_f=state.lines[0][2].won_trans_f(), s03_f=state.lines[0][3].won_trans_f(), 

    s10_h=state.lines[1][0].won_trans_h(), s11_h=state.lines[1][1].won_trans_h(), s12_h=state.lines[1][2].won_trans_h(), s13_h=state.lines[1][3].won_trans_h(), 
    s10_b=state.lines[1][0].won_trans_b(), s11_b=state.lines[1][1].won_trans_b(), s12_b=state.lines[1][2].won_trans_b(), s13_b=state.lines[1][3].won_trans_b(), 
    s10_f=state.lines[1][0].won_trans_f(), s11_f=state.lines[1][1].won_trans_f(), s12_f=state.lines[1][2].won_trans_f(), s13_f=state.lines[1][3].won_trans_f(), 

    s20_h=state.lines[2][0].won_trans_h(), s21_h=state.lines[2][1].won_trans_h(), s22_h=state.lines[2][2].won_trans_h(), s23_h=state.lines[2][3].won_trans_h(), 
    s20_b=state.lines[2][0].won_trans_b(), s21_b=state.lines[2][1].won_trans_b(), s22_b=state.lines[2][2].won_trans_b(), s23_b=state.lines[2][3].won_trans_b(), 
    s20_f=state.lines[2][0].won_trans_f(), s21_f=state.lines[2][1].won_trans_f(), s22_f=state.lines[2][2].won_trans_f(), s23_f=state.lines[2][3].won_trans_f(), 

    s30_h=state.lines[3][0].won_trans_h(), s31_h=state.lines[3][1].won_trans_h(), s32_h=state.lines[3][2].won_trans_h(), s33_h=state.lines[3][3].won_trans_h(), 
    s30_b=state.lines[3][0].won_trans_b(), s31_b=state.lines[3][1].won_trans_b(), s32_b=state.lines[3][2].won_trans_b(), s33_b=state.lines[3][3].won_trans_b(), 
    s30_f=state.lines[3][0].won_trans_f(), s31_f=state.lines[3][1].won_trans_f(), s32_f=state.lines[3][2].won_trans_f(), s33_f=state.lines[3][3].won_trans_f(), 
)
    }
}

impl Render for GameWonScreen {
    fn x() -> u16 {
        State::x()
    }

    fn y() -> u16 {
        State::y()
    }

    fn c() -> cursor::Right {
        State::c()
    }

    fn bcolor() -> Rgb {
        Rgb {r: 250, g: 248, b: 239}
    }

    fn fcolor() -> Rgb {
        Rgb {r: 119, g: 110, b: 101}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset_pos()).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();
        write!(buffer, "{}", cursor::Down(Self::y())).unwrap();

        write!(buffer, "{}", Self::body(self.state.clone())).unwrap();

        write!(buffer, "{}", style::Reset).unwrap();
    }
}

impl Render for State {
    fn x() -> u16 {
        termion::terminal_size().unwrap().0 / 2 - 15
    }

    fn y() -> u16 {
        termion::terminal_size().unwrap().1 / 2 - 10
    }

    fn bcolor() -> Rgb {
        Rgb {r: 187, g: 173, b: 161}
    }

    fn fcolor() -> Rgb {
        Rgb {r: 187, g: 173, b: 161}
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset_pos()).unwrap();

        write!(buffer, "{}", cursor::Down(Self::y())).unwrap();
        write!(buffer, "{}", style::Bold).unwrap();


        self.print_box(buffer);
        write!(buffer, "{}", style::Reset).unwrap();
    }

    fn c() -> cursor::Right {
        cursor::Right(Self::x())
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

#[derive(Serialize, Deserialize, PartialEq)]
enum Status {
    OnGoing,
    Won,
    Over
}

#[derive(Serialize, Deserialize)]
struct Game {
    state: State,
    current_score: CurrentScore,
    best_score: BestScore,
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
            current_score: CurrentScore::new(0),
            best_score: BestScore::new(0),
            keep_going: false
        }
    }

    fn reset(&self) -> Game {
        let game = Game {
            state: State::new(),
            current_score: CurrentScore::new(0),
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
                    Status::OnGoing
                } else {
                    Status::Won
                }
            },
            status => status
        }
    }

    fn render(&self, buffer: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        write!(buffer, "{}", Draw::reset()).unwrap();
        write!(buffer, "{}", termion::clear::All).unwrap();

        self.current_score.render(buffer);
        self.best_score.render(buffer);
        match self.status() {
            Status::OnGoing => self.state.render(buffer),
            Status::Won => GameWonWord::new(self.state.clone()).render(buffer),
            Status::Over => GameOverWord::new(self.state.clone()).render(buffer),
        }
    }
    
    fn handle_move(self, direction: Direction) -> Game {
        let (new_state, added_score) = self.state.handle_move(direction);
        let current_score = self.current_score.add(added_score);
        let best_score = if current_score.value > self.best_score.value {
            BestScore::new(current_score.value)
        } else {
            BestScore::new(self.best_score.value)
        };

        let game = Game {
            state: new_state,
            current_score: self.current_score.add(added_score),
            best_score: best_score,
            keep_going: self.keep_going
        };

        game.save();

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

    write!(screen, "{}", ToAlternateScreen).unwrap();
    write!(screen, "{}", cursor::Hide).unwrap();

    if game.status() == Status::Over {
        game = game.reset();
    }
    
    game.render(&mut screen);
    screen.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') | Key::Ctrl('c') => break,
            Key::Char('r') => {game = game.reset()},
            Key::Char('c') => {game = game.keep_going()},
            Key::Left  | Key::Char('a') => { game = game.handle_move(Direction::LEFT)},
            Key::Right | Key::Char('d') => { game = game.handle_move(Direction::RIGHT)},
            Key::Up    | Key::Char('w') => { game = game.handle_move(Direction::UP)},
            Key::Down  | Key::Char('s') => { game = game.handle_move(Direction::DOWN)},
            _              => {},
        }

        game.render(&mut screen);
        screen.flush().unwrap();
    }

    write!(screen, "{}", termion::cursor::Show).unwrap();
}
