use std::collections::{HashSet, VecDeque};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct GoGame {
    move_history: Vec<Move>,
    position: GoPosition,
    first_turn: usize,
    turn: usize,
    half_turn: usize,
    pub first_player: Player,
    pub next_player: Player,
}

#[derive(Debug)]
/// Holds the actual position as well as ko marks.
pub struct GoPosition {
    board_size: usize,
    position: Vec<Player>,
    /// The index of a just-captured singular piece.
    ///
    /// The next player will be unable to play here because of ko rules.
    /// This will be set to `board_size * board_size + 1` for safe measure
    /// when no such condition exists.
    ko: usize,
}

impl GoPosition {
    pub fn new(board_size: usize) -> Self {
        GoPosition {
            board_size,
            position: vec![Player::default(); board_size * board_size],
            ko: board_size * board_size + 1,
        }
    }

    pub fn board_size(&self) -> usize {
        self.board_size
    }

    pub fn position(&self) -> &Vec<Player> {
        &self.position
    }

    /// Play the move with player color `player` on the board at the coordinates (`x`, `y`)
    /// (`x` and `y` should be in `0..=18`) and propogates the effect of it to the entire board
    ///
    /// When calling this, there are no checks to see if a move at the coordinates are valid.
    fn process_move(&mut self, x: usize, y: usize, player: Player) {
        let index = self.coord_to_index(x, y);
        self.position[index] = player;
        // TODO: actually process the move
        let opp_player = match player {
            Player::White => Player::Black,
            Player::Black => Player::White,
            Player::None => Player::None,
        };
        // Don't check for capture as it doesn't make sense to.
        if let Player::None = opp_player {
            return;
        }
        // Reset ko
        self.ko = self.board_size * self.board_size + 1;
        // Get coordinates of the adjacent positions of played piece
        let sides: Vec<_> = self
            .get_surrounding_valid_indicies(self.coord_to_index(x, y))
            .into_iter()
            .filter(|i| self.position[*i] == opp_player)
            .collect();
        for s in sides {
            let to_remove = self.check_for_capture(s);
            // Set ko if necessary
            if to_remove.len() == 1 {
                self.ko = to_remove[0];
            }
            for index in to_remove {
                // indicies should already be verified
                self.position[index] = Player::None;
            }
        }
    }

    /// Check that a move is valid
    ///
    /// Currently really computationally expensive, probably.
    pub fn is_valid_move(&mut self, x: usize, y: usize, player: Player) -> bool {
        // validate coordinates
        if !self.coord_is_valid(x, y) {
            return false;
        }
        let index = self.coord_to_index(x, y);
        // check for piece at position
        if Player::None != self.position[index] {
            return false;
        }
        // check that player is not Player::None
        if let Player::None = player {
            return false;
        }
        // ko possible?
        let mut ko_pos = false;
        // check for ko
        if index == self.ko {
            ko_pos = true;
        }
        // check for self-capture
        // 1: play the move
        self.position[index] = player;
        // 2: check for self-capture
        let check = self.check_for_capture(index);
        // 2.5: check for captures on surrounding tiles
        let check_surrounding = self
            .get_surrounding_valid_indicies(index)
            .into_iter()
            .map(|i| {
                let captures = self.check_for_capture(i);
                let capturing = if !ko_pos {
                    captures.is_empty()
                } else {
                    captures.len() <= 1
                };
                self.position[i] == player || capturing
            })
            .all(|x| x);
        // 3: remove the move
        self.position[index] = Player::None;
        // 4: enforce check
        if check_surrounding && !check.is_empty() {
            return false;
        }

        // All checks passed
        true
    }

    /// Returns a Vec of all position indicies that would be removed as a result
    /// of a capture on a group including the position at the specified
    /// coordinates. Does not check for valid indicies.
    fn check_for_capture(&self, index: usize) -> Vec<usize> {
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        let mut to_remove: Vec<usize> = Vec::new();
        queue.push_front(index);

        let this = self.position[index];
        let opp = match this {
            // no piece at index, so do nothing
            Player::None => return vec![],
            Player::White => Player::Black,
            Player::Black => Player::White,
        };
        // loop through queue, adding to queue as necessary
        loop {
            // remove indirection (copy next index instead of referencing)
            let next = queue.pop_back();
            match next {
                Some(index) => {
                    // index should be verified already
                    match self.position[index] {
                        // continue matching group
                        piece if this == piece => {
                            // add to queue
                            let sides = self.get_surrounding_valid_indicies(index);
                            queue.extend(sides.into_iter().filter(|s| !visited.contains(s)));
                            to_remove.push(index);
                        }
                        // match on a boundary made by opp piece
                        piece if opp == piece => (),
                        // matched on Player::None, no capture
                        _ => {
                            return vec![];
                        }
                    }
                    visited.insert(index);
                }
                // if no more elements in queue, this side is captured
                None => {
                    // so remove all of those pieces
                    return to_remove;
                }
            }
        }
    }

    /// Returns a Vec of all the indicies of the positions surrounding the position
    /// specified by index. If a surrounding position would be off the game board,
    /// this function does not return that index.
    pub fn get_surrounding_valid_indicies(&self, index: usize) -> Vec<usize> {
        let (x, y) = self.index_to_coord(index);
        vec![
            (x.overflowing_sub(1).0, y),
            (x.overflowing_add(1).0, y),
            (x, y.overflowing_sub(1).0),
            (x, y.overflowing_add(1).0),
        ]
        .into_iter()
        .filter_map(|(x, y)| {
            let index = self.coord_to_index(x, y);
            if self.coord_is_valid(x, y) {
                Some(index)
            } else {
                None
            }
        })
        .collect()
    }

    pub fn coord_to_index(&self, x: usize, y: usize) -> usize {
        y * self.board_size + x
    }

    pub fn index_to_coord(&self, index: usize) -> (usize, usize) {
        (index % self.board_size, index / self.board_size)
    }

    /// Returns `true` if the coordinate is on the board
    fn coord_is_valid(&self, x: usize, y: usize) -> bool {
        if x >= self.board_size || y >= self.board_size {
            false
        } else {
            true
        }
    }
}

impl Deref for GoPosition {
    type Target = Vec<Player>;

    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

impl GoGame {
    pub fn new(board_size: usize) -> Self {
        GoGame {
            move_history: vec![],
            position: GoPosition::new(board_size),
            first_turn: 0,
            turn: 0,
            half_turn: 0,
            first_player: Player::Black,
            next_player: Player::Black,
        }
    }

    fn incr_turn(&mut self) {
        if self.next_player == self.first_player {
            self.turn += 1;
        }
        self.half_turn += 1;

        // Handle player and turn data
        self.next_player = match self.next_player {
            Player::White => Player::Black,
            Player::Black => Player::White,
            Player::None => Player::None,
        };
    }

    pub fn pass(&mut self) {
        self.move_history.push(Move::Pass {
            player: self.next_player,
            half_turn: self.half_turn,
        });
        self.incr_turn();
    }

    pub fn play_move(&mut self, x: usize, y: usize) -> Result<(), String> {
        match self.position.get(self.position.coord_to_index(x, y)) {
            Some(piece) => match piece {
                Player::None => {
                    self.move_history.push(Move::Play {
                        player: self.next_player,
                        square: Square { x, y },
                        half_turn: self.half_turn,
                    });
                    self.position.process_move(x, y, self.next_player);
                    self.incr_turn();

                    Ok(())
                }
                _ => Err(String::from("A piece is already at that coordinate")),
            },
            None => Err(format!(
                "Invalid coordinate for board size {}",
                self.position.board_size
            )),
        }
    }
}

impl Default for GoGame {
    fn default() -> Self {
        Self::new(19)
    }
}

impl Deref for GoGame {
    type Target = GoPosition;

    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

impl DerefMut for GoGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.position
    }
}

#[derive(Debug)]
enum Move {
    Pass {
        player: Player,
        half_turn: usize,
    },
    Play {
        player: Player,
        square: Square,
        half_turn: usize,
    },
}

#[derive(Debug)]
struct Square {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    Black,
    White,
    None,
}

impl Default for Player {
    fn default() -> Self {
        Player::None
    }
}
