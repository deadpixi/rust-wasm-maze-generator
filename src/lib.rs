use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::default::Default;
use std::fmt;
use std::ops::{Index, IndexMut};

// FIXME - unify carving and solving, it's the same basic operation
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    openings: [bool; 4],
}

impl Cell {
    fn carve(&mut self, direction: Direction) {
        self.openings[direction as usize] = true;
    }
}

#[wasm_bindgen]
impl Cell {
    pub fn has_opening(&self, direction: Direction) -> bool {
        self.openings[direction as usize]
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl Direction {
    fn op(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    fn translate(&self, position: &Position, maze: &Maze) -> Option<Position> {
        match self {
            Self::North if position.y > 0 => Some(Position::new(position.x, position.y - 1)),
            Self::South if position.y < maze.height - 1 => {
                Some(Position::new(position.x, position.y + 1))
            }
            Self::West if position.x > 0 => Some(Position::new(position.x - 1, position.y)),
            Self::East if position.x < maze.width - 1 => {
                Some(Position::new(position.x + 1, position.y))
            }
            _ => None,
        }
    }
}

#[wasm_bindgen]
pub fn translate(direction: Direction, position: &Position, maze: &Maze) -> Option<Position> {
    direction.translate(position, maze)
}

#[wasm_bindgen]
pub struct Maze {
    start: Position,
    finish: Position,
    height: usize,
    width: usize,
    cells: Vec<Cell>,
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in 0..self.width {
            let cell = self[Position::new(x, 0)];
            write!(
                f,
                "█{}",
                if cell.has_opening(Direction::North) {
                    " "
                } else {
                    "█"
                }
            )?;
        }
        writeln!(f, "█")?;
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self[Position::new(x, y)];
                write!(
                    f,
                    "{}",
                    if cell.has_opening(Direction::West) {
                        " "
                    } else {
                        "█"
                    }
                )?;
                write!(f, " ")?;
            }
            writeln!(f, "█")?;
            for x in 0..self.width {
                let cell = self[Position::new(x, y)];
                write!(
                    f,
                    "█{}",
                    if cell.has_opening(Direction::South) {
                        " "
                    } else {
                        "█"
                    }
                )?;
            }
            writeln!(f, "█")?;
        }

        Ok(())
    }
}

impl Index<Position> for Maze {
    type Output = Cell;

    fn index(&self, position: Position) -> &Cell {
        &self.cells[(position.y * self.width + position.x) as usize]
    }
}

impl IndexMut<Position> for Maze {
    fn index_mut(&mut self, position: Position) -> &mut Cell {
        &mut self.cells[(position.y * self.width + position.x) as usize]
    }
}

#[wasm_bindgen]
impl Maze {
    pub fn new() -> Self {
        Self::new_with_size_and_start(20, 20, true)
    }

    pub fn new_with_size_and_start(height: usize, width: usize, opposite_start: bool) -> Self {
        assert!(height >= 2 && width >= 2);

        let mut rng = rand::thread_rng();
        Self {
            height,
            width,
            cells: (0..height * width).map(|_| Cell::default()).collect(),
            start: if opposite_start {
                Position::new(0, 0)
            } else {
                Position::new(rng.gen_range(0..width), 0)
            },
            finish: if opposite_start {
                Position::new(width - 1, height - 1)
            } else {
                Position::new(rng.gen_range(0..width), height - 1)
            },
        }
        .generate()
    }

    pub fn at(&self, x: usize, y: usize) -> Cell {
        self[Position::new(x, y)]
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn start(&self) -> Position {
        self.start
    }

    pub fn finish(&self) -> Position {
        self.finish
    }

    // bindgen doesn't expose to_string directly
    pub fn as_string(&self) -> String {
        self.to_string()
    }
}

impl Default for Maze {
    fn default() -> Self {
        Maze::new()
    }
}

impl Maze {
    fn generate(mut self) -> Self {
        let mut rng = rand::thread_rng();
        let mut stack: Vec<Position> = vec![self.start];
        let mut visited: HashSet<Position> = HashSet::new();
        let mut directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];

        let start = self.start;
        let finish = self.finish;
        self[start].carve(Direction::North);
        self[finish].carve(Direction::South);

        while let Some(peek) = stack.last() {
            directions.shuffle(&mut rng);
            let peek = *peek;
            if let Some((dir, pos)) = directions
                .map(|dir| (dir, dir.translate(&peek, &self)))
                .iter()
                .cloned()
                .find(|(_, pos)| pos.is_some() && !visited.contains(&pos.unwrap()))
            {
                let pos = pos.unwrap();
                self[peek].carve(dir);
                self[pos].carve(dir.op());
                visited.insert(pos);
                stack.push(pos);
            } else {
                stack.pop();
            }
        }

        self
    }
}

/* Note that this would be better expressed as a single enumeration,
 * but that wouldn't be supported by wasm-bindgen.
 */
#[wasm_bindgen]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    VisitCell,
    UnvisitCell,
}

#[wasm_bindgen]
pub struct Step {
    pub action: Action,
    pub position: Position,
    pub previous: Position,
}

impl Step {
    fn new(action: Action, position: Position, previous: Position) -> Self {
        Self {
            action,
            position,
            previous,
        }
    }
}

#[wasm_bindgen]
pub struct Solver {
    stack: Vec<Position>,
    visited: HashSet<Position>,
}

#[wasm_bindgen]
impl Solver {
    pub fn new_for_maze(maze: &Maze) -> Self {
        Self {
            stack: vec![],
            visited: HashSet::with_capacity(maze.height * maze.width),
        }
    }

    pub fn step(&mut self, maze: &Maze) -> Option<Step> {
        if self.visited.contains(&maze.finish) {
            return None;
        }

        if self.stack.is_empty() {
            self.visit(maze.start);
            return Some(Step::new(Action::VisitCell, maze.start, maze.start));
        }

        let mut rng = rand::thread_rng();
        let mut directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        directions.shuffle(&mut rng);

        if let Some(position) = self.stack.last() {
            for direction in directions {
                if let Some(candidate) = direction.translate(position, maze) {
                    let position = *position;
                    if maze[position].has_opening(direction) && !self.visited.contains(&candidate) {
                        self.visit(candidate);
                        return Some(Step::new(Action::VisitCell, candidate, position));
                    }
                }
            }
        }

        self.stack
            .pop()
            .map(|old| Step::new(Action::UnvisitCell, old, old))
    }
}

impl Solver {
    fn visit(&mut self, position: Position) {
        self.stack.push(position);
        self.visited.insert(position);
    }
}
