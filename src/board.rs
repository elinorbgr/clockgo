use std::collections::{DList, HashSet, SmallIntMap};
use std::iter::FromIterator;

static board_maxsize : uint = 25;

#[deriving(PartialEq)]
pub enum Colour {
    Black,
    White
}

// structs needed for history

pub enum Vertex {
    Put(uint, uint),
    Pass
}

pub struct Move {
    pub player: Colour,
    pub move: Vertex,
    pub removed: Vec<(uint, uint)>
}

// structs needed for board representation

#[deriving(PartialEq)]
pub enum Intersection {
    Stone(Colour, uint),
    Empty
}

// board itself

/// This struct represents a board. It stores information about
/// groups to automatically remove dead stones, allow undoing
/// and detect simple kos.
#[allow(dead_code)]
pub struct Board {
    stones: [[Intersection, ..board_maxsize], ..board_maxsize],
    history: DList<Move>,
    groups: SmallIntMap< HashSet<(uint, uint)> >,
    size: uint,
    white_dead: uint,
    black_dead: uint,
    current_ko: (uint, uint)
}

impl Board {

    /// Creates a new Board.
    pub fn new() -> Board {
        Board {
            stones: [[Empty, ..board_maxsize], ..board_maxsize],
            history: DList::new(),
            groups: SmallIntMap::new(),
            size: 19,
            white_dead: 0,
            black_dead: 0,
            current_ko: (0, 0)
        }
    }

    /// Allows read-only access to the board
    pub fn get_board<'a>(&'a self) -> &'a [[Intersection, ..board_maxsize], ..board_maxsize] {
        &self.stones
    }

    /// Allows read-only access to the history
    pub fn get_history<'a>(&'a self) -> &'a DList<Move> {
        &self.history
    }

    /// Allow read-only access to the groups data
    pub fn get_groups<'a>(&'a self) -> &'a SmallIntMap< HashSet<(uint, uint)> > {
        &self.groups
    }

    /// Board current size
    pub fn get_size(&self) -> uint {
        self.size
    }

    /// Current dead stones (black, white)
    pub fn get_deads(&self) -> (uint, uint) {
        (self.black_dead, self.white_dead)
    }

    /// Option tothe coordinates of current ko.
    pub fn get_current_ko(&self) -> Option<(uint, uint)> {
        if self.current_ko == (0, 0) {
            None
        } else {
            Some(self.current_ko)
        }
    }

    /// Resets the board and clear the history. The board is then
    /// ready for a new game.
    pub fn clear(&mut self) {
        self.history.clear();
        self.groups.clear();
        for i in range(0, board_maxsize) {
            for j in range(0, board_maxsize) {
                self.stones[i][j] = Empty;
            }
        }
    }

    /// Change the size of the board, must be between 1 and 25 inclusive.
    pub fn resize(&mut self, newsize: uint) -> bool {
        if newsize > 0 && newsize <= board_maxsize {
            self.clear();
            self.size = newsize;
            true
        } else {
            false
        }
    }

    /// Returns a copy of the board without history, can thus be used to think,
    /// experiment and prepare the next move.
    pub fn clone_without_history(&self) -> Board {
        Board {
            stones: {
                let mut array = [[Empty, ..board_maxsize], ..board_maxsize];
                for i in range(0, self.size) {
                    for j in range(0, self.size) {
                        array[i][j] = self.stones[i][j];
                    }
                }
            array },
            history: DList::new(),
            groups: self.groups.clone(),
            size: self.size,
            white_dead: self.white_dead,
            black_dead: self.black_dead,
            current_ko: self.current_ko
        }
    }

    fn loop_over_neighbours(x:uint, y:uint, size:uint, func: |uint, uint|  -> ()) {
        if x > 1 {
            func(x-1, y);
        }
        if y > 1 {
            func(x, y-1);
        }
        if x < size - 1 {
            func(x+1, y);
        }
        if y < size - 1 {
            func(x, y+1);
        }
    }

    /// Returns a vec of the liberties of the groups containing given stone.
    /// Will return empty vec is there is no stone.
    pub fn liberties_of(&mut self, x: uint, y: uint) -> Vec<(uint,uint)> {
        match self.stones[x-1][y-1] {
            Empty => Vec::new(),
            Stone(_, gid) => {
                let mut liberties = HashSet::new();
                for &(v, w) in self.groups[gid].iter() {
                    Board::loop_over_neighbours(v, w, self.size, |a, b| {
                        if self.stones[a-1][b-1] == Empty {
                            liberties.insert((a,b));
                        }
                    });
                }
                FromIterator::from_iter(liberties.move_iter())
            }
        }
    }

    fn next_gid(&self) -> uint {
        let mut last = 0u;
        if self.groups.is_empty() {
            0
        } else {
            for i in self.groups.keys() {
                if i > last + 1 {
                    break;
                }
                last = i;
            }
            last + 1
        }
    }

    fn split_group(&mut self, gid: uint) {
        if !self.groups.contains_key(&gid) {
            return;
        }
        while !self.groups[gid].is_empty() {
            // retrieve a random item
            let &(x, y) = self.groups[gid].iter().next().unwrap();
            let newgid = self.next_gid();
            self.groups.insert(newgid, HashSet::new());
            self.stones[x-1][y-1] = match self.stones[x-1][y-1] {
                Stone(col, id) if id == gid => Stone(col, newgid),
                // if we reach this point, it means the internal data
                // was inconsistent
                _ => unreachable!()
            };
            self.groups.find_mut(&gid).unwrap().remove(&(x, y));
            self.groups.find_mut(&newgid).unwrap().insert((x, y));
            // let's find all connected stones
            let mut to_loop = DList::new();
            Board::loop_over_neighbours(x, y, self.size, |a, b| {
                if self.groups[gid].contains(&(a,b)) {
                    to_loop.push((a,b));
                    self.groups.find_mut(&gid).unwrap().remove(&(a,b));
                }
            });
            while !to_loop.is_empty() {
                let (v, w) = to_loop.pop().unwrap();
                self.stones[v-1][w-1] = match self.stones[v-1][w-1] {
                    Stone(col, id) if id == gid => Stone(col, newgid),
                    _ => unreachable!() // same here
                };
                self.groups.find_mut(&newgid).unwrap().insert((v, w));
                Board::loop_over_neighbours(v, w, self.size, |a, b| {
                    if self.groups[gid].contains(&(a,b)) {
                        to_loop.push((a,b));
                        self.groups.find_mut(&gid).unwrap().remove(&(a,b));
                    }
                });
            }
        }
        self.groups.remove(&gid);
    }

    /// Undo the last move.
    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            None => false,
            Some(Move{player: player, move: move, removed: removed}) => {
                match move {
                    Pass => true,
                    Put(x, y) => {
                        let oldgid = match self.stones[x-1][y-1] {
                            Stone(_,id) => id,
                            _ => unreachable!() // how could there be no stone ??
                        };
                        self.split_group(oldgid);
                        self.stones[x-1][y-1] = Empty;
                        // restore removed stones
                        let tmpgid = self.next_gid();
                        let removedcolor = match player {
                            White => Black,
                            Black => White
                        };
                        self.groups.insert(tmpgid, HashSet::new());
                        for &(v,w) in removed.iter() {
                            self.stones[v-1][w-1] = Stone(removedcolor, tmpgid);
                            self.groups.find_mut(&tmpgid).unwrap().insert((v, w));
                        }
                        self.split_group(tmpgid);
                        true
                    }
                }
            }
        }
    }

    fn remove_if_dead(&mut self, x: uint, y: uint) -> Vec<(uint,uint)> {
        match self.stones[x-1][y-1] {
            Empty => {Vec::new()},
            Stone(_, gid) => if self.liberties_of(x,y).len() == 0 {
                let mut deads = Vec::new();
                for &(v,w) in self.groups[gid].iter() {
                    self.stones[v-1][w-1] = Empty;
                    deads.push((v,w));
                }
                self.groups.remove(&gid);
                deads
            } else {
                Vec::new()
            }
        }
    }

    fn fuse_groups(&mut self, x1: uint, y1: uint, x2: uint, y2: uint) {
        // gid1 shall be the biggest group
        let (gid1, gid2) = match (self.stones[x1-1][y1-1], self.stones[x2-1][y2-1]) {
            (Stone(_, g1), Stone(_, g2)) if g1 != g2 => {
                if self.groups[g1].len() > self.groups[g2].len() {
                    (g1, g2)
                } else {
                    (g2, g1)
                }
            }
            _ => { return }
        };
        for &(v,w) in self.groups[gid2].iter() {
            match self.stones[v-1][w-1] {
                Stone(col, _) => { self.stones[v-1][w-1] = Stone(col, gid1); }
                Empty => { unreachable!(); }
            }
            self.groups.find_mut(&gid1).unwrap().insert((v,w));
        }
        self.groups.remove(&gid2);
    }

    /// The chosen play passes his turn
    pub fn pass(&mut self, player: Colour) {
        self.history.push(Move{
                player: player,
                move: Pass,
                removed: Vec::new()
            });
    }

    /// Plays the given move, will return false if the move cannot be played
    /// (either because there is already a stone, or the stone would be dead,
    /// or it is a simple ko).
    pub fn play(&mut self, player: Colour, x: uint, y: uint) -> bool {
        if self.stones[x-1][y-1] != Empty || (x, y) == self.current_ko {
            // move is not possible
            false
        } else {
            let gid = self.next_gid();
            self.stones[x-1][y-1] = Stone(player, gid);
            // are we killing enemies_stones ?
            let mut killed = Vec::new();
            Board::loop_over_neighbours(x, y, self.size, |a, b| {
                match self.stones[a-1][b-1] {
                    Stone(col, _) if col != player => {
                        killed.push_all(self.remove_if_dead(a,b).as_slice());
                    },
                    _ => {}
                }
            });
            if killed.len() == 0 {
                // the move might be invalid, we must be more careful
                let mut alive = false;
                Board::loop_over_neighbours(x, y, self.size, |a, b| {
                    alive = alive || match self.stones[a-1][b-1] {
                    Empty => true,
                    Stone(col, _) if col == player => self.liberties_of(a,b).len() > 0,
                    _ => false
                    }
                });
                if !alive {
                    // we should not have played this
                    self.stones[x-1][y-1] = Empty;
                    return false;
                }
            }
            // okay, we live, let's clean up
            self.groups.insert(gid, HashSet::new());
            self.groups.find_mut(&gid).unwrap().insert((x,y));
            Board::loop_over_neighbours(x, y, self.size, |a, b| {
                match self.stones[a-1][b-1] {
                    Stone(col, _) if col == player => { self.fuse_groups(x,y,a,b); }
                    _ => {}
                }
            });
            match player {
                White => { self.black_dead += killed.len(); }
                Black => { self.white_dead += killed.len(); }
            }
            if killed.len() == 1 && self.liberties_of(x, y).len() == 1 {
                self.current_ko = killed[0];
            } else {
                self.current_ko = (0, 0);
            }
            self.history.push(Move{
                player: player,
                move: Put(x,y),
                removed: killed
            });
            true
        }
    }
}
