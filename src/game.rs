use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
pub struct GoGame {
    board_size: usize,
    move_history: Vec<Move>,
    position: Vec<Player>,
    first_turn: usize,
    turn: usize,
    half_turn: usize,
    pub first_player: Player,
    pub next_player: Player,
}

impl GoGame {
    pub fn new(board_size: usize) -> Self {
        GoGame {
            board_size,
            move_history: vec![],
            position: vec![Player::default(); board_size * board_size],
            first_turn: 0,
            turn: 0,
            half_turn: 0,
            first_player: Player::Black,
            next_player: Player::Black,
        }
    }

    pub fn board_size(&self) -> usize {
        self.board_size
    }

    pub fn position(&self) -> &Vec<Player> {
        &self.position
    }

    pub fn play_move(&mut self, x: usize, y: usize) -> Result<(), String> {
        match self.position.get(self.coord_to_index(x, y)) {
            Some(piece) => {
                match piece {
                    Player::None => {
                        self.process_move(x, y, self.next_player);

                        // Handle player and turn data
                        self.next_player = match self.next_player {
                            Player::White => Player::Black,
                            Player::Black => Player::White,
                            Player::None => Player::None,
                        };
                        if self.next_player == self.first_player {
                            self.turn += 1;
                        }
                        self.half_turn += 1;

                        Ok(())
                    }
                    _ => Err(String::from("A piece is already at that coordinate")),
                }
            }
            None => Err(format!(
                "Invalid coordinate for board size {}",
                self.board_size
            )),
        }
    }

    /// Play the move with player color `player` on the board at the coordinates (`x`, `y`)
    /// (`x` and `y` should be in `0..=18`) and propogates the effect of it to the entire board
    ///
    /// When calling this, there are no checks to see if a move at the coordinates are valid.
    fn process_move(&mut self, x: usize, y: usize, player: Player) {
        let index = self.coord_to_index(x, y);
        self.position[index] = player;
        self.move_history.push(Move {
            player,
            square: Square { x, y },
            half_turn: self.half_turn,
        });
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
        // Get coordinates of the adjacent positions of played piece
        let sides = self.get_surrounding_valid_indicies(self.coord_to_index(x, y));
        for s in sides {
            let to_remove = self.check_for_capture(s);
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
        // check for self-capture
        // 1: play the move
        self.position[index] = player;
        // 2: check for self-capture
        let check = self.check_for_capture(index);
        // 2.5: check for captures on surrounding tiles
        let check_surrounding = self
            .get_surrounding_valid_indicies(index)
            .into_iter()
            .map(|i| self.position[index] == player || self.check_for_capture(i).is_empty())
            .all(|x| x);
        // 3: remove the move
        self.position[index] = Player::None;
        // 4: enforce check
        if check_surrounding && !check.is_empty() {
            return false;
        }
        // TODO: check for ko

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
            (x.saturating_sub(1), y),
            (x.saturating_add(1), y),
            (x, y.saturating_add(1)),
            (x, y.saturating_sub(1)),
        ]
        .into_iter()
        .filter_map(|(x, y)| {
            let index = self.coord_to_index(x, y);
            if self.index_is_valid(index) {
                Some(index)
            } else {
                None
            }
        })
        .collect()
    }

    // Unused function
    // /// Returns the player of the piece at the coordinate specified. Panics
    // /// on out of bounds.
    // pub fn piece_at_coord(&self, x: usize, y: usize) -> Player {
    //     self.position[self.coord_to_index(x, y)]
    // }

    pub fn coord_to_index(&self, x: usize, y: usize) -> usize {
        y * self.board_size + x
    }

    pub fn index_to_coord(&self, index: usize) -> (usize, usize) {
        (index % self.board_size, index / self.board_size)
    }

    /// Returns `true` if the coordinate is on the board
    fn coord_is_valid(&self, x: usize, y: usize) -> bool {
        self.index_is_valid(self.coord_to_index(x, y))
    }

    /// Returns `true if the position specified by the index is on the board`
    fn index_is_valid(&self, index: usize) -> bool {
        (0..self.position.len()).contains(&index)
    }
}

impl Default for GoGame {
    fn default() -> Self {
        Self::new(19)
    }
}

#[derive(Debug)]
struct Move {
    pub player: Player,
    pub square: Square,
    pub half_turn: usize,
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
