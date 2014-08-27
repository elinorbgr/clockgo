use std::collections::{DList, TreeSet, SmallIntMap, Deque};
use std::collections::treemap::SetItems;

macro_rules! single_match(
    ($mtch:expr : $ptrn:pat => $blk:block) => (
    match $mtch {
        $ptrn => $blk,
        _ => {}
    }
    )
)

static board_maxsize : uint = 25;

#[deriving(PartialEq)]
pub enum Colour {
    Black,
    White
}

// structs needed for board representation

#[deriving(PartialEq)]
pub enum Intersection {
    Stone(Colour, uint),
    Empty
}

#[deriving(Clone)]
pub struct Group {
    stones: TreeSet<(uint, uint)>,
    liberties: TreeSet<(uint, uint)>
}

impl Group {
    pub fn new() -> Group {
        Group {
            stones: TreeSet::new(),
            liberties: TreeSet::new()
        }
    }

    pub fn is_dead(&self) -> bool {
        self.liberties.is_empty()
    }

    pub fn add_stone(&mut self, x:uint, y: uint) {
        self.stones.insert((x,y));
        self.liberties.remove(&(x,y));
    }

    pub fn add_liberty(&mut self, x: uint,y:uint) {
        self.liberties.insert((x,y));
    }

    pub fn remove_liberty(&mut self, x: uint, y: uint) {
        self.liberties.remove(&(x,y));
    }

    pub fn absorb(&mut self, other: Group) {
        self.stones.extend(other.stones.move_iter());
        self.liberties.extend(other.liberties.move_iter());
        for &(x,y) in self.liberties.intersection(&self.stones) {
            self.liberties.remove(&(x,y));
        }
    }

    pub fn stone_count(&self) -> uint {
        self.stones.len()
    }

    pub fn get_stones<'a>(&'a self) -> SetItems<'a, (uint, uint)> {
        self.stones.iter()
    }

    pub fn liberty_count(&self) -> uint {
        self.liberties.len()
    }

    pub fn get_liberties<'a>(&'a self) -> SetItems<'a, (uint, uint)> {
        self.liberties.iter()
    }

    pub fn dismantle(self) -> TreeSet<(uint, uint)> {
        self.stones
    }
}

// structs needed for history

pub enum Vertex {
    Put(uint, uint),
    Pass
}

pub struct Move {
    pub player: Colour,
    pub move: Vertex,
    pub removed: Vec<Group>
}

// board itself

/// This struct represents a board. It stores information about
/// groups to automatically remove dead stones, allow undoing
/// and detect simple kos.
#[allow(dead_code)]
pub struct Board {
    stones: [[Intersection, ..board_maxsize], ..board_maxsize],
    history: DList<Move>,
    groups: SmallIntMap<Group>,
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
    pub fn get_groups<'a>(&'a self) -> &'a SmallIntMap<Group> {
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

    /// Option to the coordinates of current ko.
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
        self.stones = [[Empty, ..board_maxsize], ..board_maxsize];
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
        if x > 1 { func(x-1, y); }
        if y > 1 { func(x, y-1); }
        if x < size { func(x+1, y); }
        if y < size { func(x, y+1); }
    }

    // shall be called only if you KNOW a stone is there
    fn gid_of_stone(&self, x:uint, y:uint) -> uint {
        match self.stones[x-1][y-1] {
            Stone(_, gid) => gid,
            _ => unreachable!()
        }
    }

    // we want to reuse old keys
    fn next_gid(&self) -> uint {
        let mut key = 0u;
        while self.groups.contains_key(&key) { key += 1u; }
        key
    }

    fn split_group(&mut self, gid: uint, unput: (uint,uint)) {
        self.stones[unput.val0()-1][unput.val1()-1] = Empty;
        let mut oldstones = self.groups.pop(&gid).unwrap().dismantle();
        oldstones.remove(&unput);
        while !oldstones.is_empty() {
            // retrieve a random item
            let &(x, y) = oldstones.iter().next().unwrap();
            let newgid = self.next_gid();
            self.groups.insert(newgid, Group::new());
            // targetted references for closures
            let ref mut newgroup = self.groups.find_mut(&newgid).unwrap();
            let &mut mystones = &self.stones;
            let mut to_loop = DList::new();
            // loop preparation
            to_loop.push((x,y));
            oldstones.remove(&(x,y));
            // loop on the whole groupe
            while !to_loop.is_empty() {
                let (v, w) = to_loop.pop().unwrap();
                mystones[v-1][w-1] = match self.stones[v-1][w-1] {
                    Stone(col, id) if id == gid => Stone(col, newgid),
                    _ => unreachable!() // same here
                };
                newgroup.add_stone(v, w);
                Board::loop_over_neighbours(v, w, self.size, |a, b| {
                    if oldstones.contains(&(a,b)) {
                        to_loop.push((a,b));
                        oldstones.remove(&(a,b));
                    } else if mystones[a-1][b-1] == Empty {
                        newgroup.add_liberty(a,b);
                    }
                });
            }
        }
    }

    /// Undo the last move.
    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            None => false,
            Some(Move{player: _, move: Pass, removed: _}) => true,
            Some(Move{player: player, move: Put(x,y), removed: removed}) => {
                let oldgid = self.gid_of_stone(x, y);
                self.split_group(oldgid, (x,y));
                // restore removed stones
                let removedcolor = match player { White => Black, Black => White };
                for mut grp in removed.move_iter() {
                    let newgid = self.next_gid();
                    for &(v,w) in grp.get_stones() {
                        self.stones[v-1][w-1] = Stone(removedcolor, newgid);
                    }
                    grp.add_liberty(x, y);
                    self.groups.insert(newgid, grp);
                }
                // check if last move was a ko
                single_match!(self.history.back() :
                    Some(&Move{player: _, move: Put(v, w), removed: ref removed}) => {
                    if removed.len() == 1 && removed[0].stone_count() == 1 &&
                       self.groups[self.gid_of_stone(v, w)].liberty_count() == 1 {
                        self.current_ko = *removed[0].get_stones().next().unwrap();
                    } else {
                        self.current_ko = (0, 0);
                    }
                });
                true
            }
        }
    }

    // Removes the liberty 'killer' of group containing stone 'stone'.
    // If it was the last liberty, the groups is removed and returned.
    fn remove_liberty(&mut self, stone: (uint, uint), killer: (uint, uint)) -> Option<Group> {
        let ((x,y),(kx,ky)) = (stone, killer);
        match self.stones[x-1][y-1] {
            Empty => None,
            Stone(_, gid) => {
                self.groups.find_mut(&gid).unwrap().remove_liberty(kx, ky);
                if self.groups[gid].is_dead() {
                    let grp = self.groups.pop(&gid).unwrap();
                    // remove stones and add liberties to neighbors
                    for &(v, w) in grp.get_stones() {
                        self.stones[v-1][w-1] = Empty;
                        Board::loop_over_neighbours(v, w, self.size, |a, b| {
                            single_match!(self.stones[a-1][b-1] : Stone(_, grpid) => {
                                if grpid != gid {
                                    self.groups.find_mut(&grpid).unwrap().add_liberty(v,w);
                                }
                            });
                        });
                    }
                    Some(grp)
                } else {
                    None
                }
            }
        }
    }

    fn fuse_groups(&mut self, x1: uint, y1: uint, x2: uint, y2: uint) {
        // gid1 shall be the biggest group
        let (gid1, gid2) = match (self.stones[x1-1][y1-1], self.stones[x2-1][y2-1]) {
            (Stone(col1, g1), Stone(col2, g2)) if g1 != g2  && col1 == col2 => {
                if self.groups[g1].stone_count() > self.groups[g2].stone_count() {
                    (g1, g2)
                } else {
                    (g2, g1)
                }
            }
            _ => { return }
        };
        let oldgroup = self.groups.pop(&gid2).unwrap();
        for &(v,w) in oldgroup.get_stones() {
            single_match!(self.stones[v-1][w-1] : Stone(col, _) => {
                self.stones[v-1][w-1] = Stone(col, gid1);
            });
        }
        self.groups.find_mut(&gid1).unwrap().absorb(oldgroup);
    }

    /// The chosen player passes his turn
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
            return false;
        }
        // put the stone
        let gid = self.next_gid();
        self.stones[x-1][y-1] = Stone(player, gid);
        self.groups.insert(gid, Group::new());
        self.groups.find_mut(&gid).unwrap().add_stone(x,y);
        // are we killing enemies_stones ?
        let mut killed = Vec::new();
        Board::loop_over_neighbours(x, y, self.size, |a, b| {
            single_match!(self.stones[a-1][b-1] : Stone(col, _) => {
                if col != player {
                    single_match!(self.remove_liberty((a,b),(x,y)) : Some(grp) => {
                        killed.push(grp);
                    });
                }
            });
        });
        if killed.len() == 0 {
            // the move might be invalid, we must be more careful
            let mut alive = false;
            Board::loop_over_neighbours(x, y, self.size, |a, b| {
                alive = alive || match self.stones[a-1][b-1] {
                Empty => true,
                Stone(col, gid) if col == player => self.groups[gid].liberty_count() > 1,
                _ => false
                }
            });
            if !alive {
                // we should not have played this
                self.stones[x-1][y-1] = Empty;
                self.groups.remove(&gid);
                // restore liberties
                Board::loop_over_neighbours(x, y, self.size, |a, b| {
                    single_match!(self.stones[a-1][b-1]: Stone(col, tmpid) => {
                        if col != player {
                            self.groups.find_mut(&tmpid).unwrap().add_liberty(x,y);
                        }
                    });
                });
                return false;
            }
        }
        // okay, we live, let's clean up
        // does this stone have liberties ?
        Board::loop_over_neighbours(x, y, self.size, |a, b| {
            single_match!(self.stones[a-1][b-1] : Empty => {
                self.groups.find_mut(&gid).unwrap().add_liberty(a,b);
            });
        });
        // fuse groups
        Board::loop_over_neighbours(x, y, self.size, |a, b| {
            single_match!(self.stones[a-1][b-1] : Stone(col, _) => {
                if col == player { self.fuse_groups(x,y,a,b); }
            });
        });
        //count dead stones
        for grp in killed.iter() {
            match player {
                White => { self.black_dead += grp.stone_count(); }
                Black => { self.white_dead += grp.stone_count(); }
            }
        }
        // check ko
        // as we fused groups, our gid is no longer known
        let mygid = self.gid_of_stone(x, y);
        if killed.len() == 1 && killed[0].stone_count() == 1 &&
           self.groups[mygid].stone_count() == 1 {
            self.current_ko = *killed[0].get_stones().next().unwrap();
        } else {
            self.current_ko = (0, 0);
        }
        // save history
        self.history.push(Move{
            player: player,
            move: Put(x,y),
            removed: killed
        });
        true
    }
}
