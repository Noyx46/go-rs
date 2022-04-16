pub struct GoGame {
    board_size: usize,
    move_history: Vec<Move>,
    position: Vec<Player>,
    _first_turn: usize,
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
            _first_turn: 0,
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
    /// When calling this, there are no checks to see if the coordinates are in-bounds,
    /// nor if the player is not None nor if the there is already a piece on the
    /// specified coordinates.
    fn process_move(&mut self, x: usize, y: usize, player: Player) {
        let index = self.coord_to_index(x, y);
        self.position[index] = player;
        self.move_history.push(Move {
            _player: player,
            _square: Square { _x: x, _y: y },
            _half_turn: self.half_turn,
        })
        // TODO: actually process the move
    }

    pub fn coord_to_index(&self, x: usize, y: usize) -> usize {
        y * self.board_size + x
    }

    pub fn index_to_coord(&self, index: usize) -> (usize, usize) {
        (index % self.board_size, index / self.board_size)
    }
}

impl Default for GoGame {
    fn default() -> Self {
        Self::new(19)
    }
}

struct Move {
    pub _player: Player,
    pub _square: Square,
    pub _half_turn: usize,
}

struct Square {
    pub _x: usize,
    pub _y: usize,
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
