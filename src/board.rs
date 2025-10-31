//! Board representation using bitboards and game state management.

use crate::utils::*;
use crate::zobrist::{zobrist, Zobrist};

/// Piece types
pub const PAWN: u8 = 0;
pub const KNIGHT: u8 = 1;
pub const BISHOP: u8 = 2;
pub const ROOK: u8 = 3;
pub const QUEEN: u8 = 4;
pub const KING: u8 = 5;

/// Colors
pub const WHITE: u8 = 0;
pub const BLACK: u8 = 1;

/// Castling rights bits
pub const CASTLE_WK: u8 = 1; // White kingside
pub const CASTLE_WQ: u8 = 2; // White queenside
pub const CASTLE_BK: u8 = 4; // Black kingside
pub const CASTLE_BQ: u8 = 8; // Black queenside

/// History entry for unmake
#[derive(Clone, Copy)]
pub struct History {
    pub mov: Move,
    pub captured: Option<u8>,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfmove: u16,
    pub hash: u64,
}

/// Chess board representation
pub struct Board {
    /// Bitboards for each piece type [color][piece]
    pub pieces: [[u64; 6]; 2],
    /// Combined occupancy for each color
    pub occupancy: [u64; 2],
    /// Side to move (WHITE or BLACK)
    pub side: u8,
    /// Castling rights
    pub castling: u8,
    /// En-passant target square (if any)
    pub en_passant: Option<u8>,
    /// Halfmove clock (50-move rule)
    pub halfmove: u16,
    /// Fullmove number
    pub fullmove: u16,
    /// Zobrist hash of current position
    pub hash: u64,
    /// Move history for unmake
    pub history: Vec<History>,
}

impl Board {
    /// Create a new empty board
    pub fn new() -> Self {
        Board {
            pieces: [[0; 6]; 2],
            occupancy: [0; 2],
            side: WHITE,
            castling: 0,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
            hash: 0,
            history: Vec::new(),
        }
    }

    /// Create board from starting position
    pub fn starting_position() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Parse FEN string and create board
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err("Empty FEN string".to_string());
        }

        // Parse piece placement
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err("Invalid number of ranks".to_string());
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx;
            let mut file = 0;
            
            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10).unwrap() as u8;
                } else {
                    let (piece, color) = match ch {
                        'P' => (PAWN, WHITE),
                        'N' => (KNIGHT, WHITE),
                        'B' => (BISHOP, WHITE),
                        'R' => (ROOK, WHITE),
                        'Q' => (QUEEN, WHITE),
                        'K' => (KING, WHITE),
                        'p' => (PAWN, BLACK),
                        'n' => (KNIGHT, BLACK),
                        'b' => (BISHOP, BLACK),
                        'r' => (ROOK, BLACK),
                        'q' => (QUEEN, BLACK),
                        'k' => (KING, BLACK),
                        _ => return Err(format!("Invalid piece: {}", ch)),
                    };
                    
                    if file >= 8 {
                        return Err("Too many pieces in rank".to_string());
                    }
                    
                    let sq = square(rank as u8, file);
                    board.pieces[color as usize][piece as usize] |= bit_at(sq);
                    file += 1;
                }
            }
            
            if file != 8 {
                return Err("Invalid number of squares in rank".to_string());
            }
        }

        // Parse side to move
        if parts.len() > 1 {
            board.side = match parts[1] {
                "w" => WHITE,
                "b" => BLACK,
                _ => return Err("Invalid side to move".to_string()),
            };
        }

        // Parse castling rights
        if parts.len() > 2 {
            board.castling = 0;
            for ch in parts[2].chars() {
                match ch {
                    'K' => board.castling |= CASTLE_WK,
                    'Q' => board.castling |= CASTLE_WQ,
                    'k' => board.castling |= CASTLE_BK,
                    'q' => board.castling |= CASTLE_BQ,
                    '-' => {},
                    _ => return Err(format!("Invalid castling right: {}", ch)),
                }
            }
        }

        // Parse en-passant
        if parts.len() > 3 {
            board.en_passant = if parts[3] == "-" {
                None
            } else {
                Some(parse_square(parts[3]).ok_or("Invalid en-passant square")?)
            };
        }

        // Parse halfmove clock
        if parts.len() > 4 {
            board.halfmove = parts[4].parse().unwrap_or(0);
        }

        // Parse fullmove number
        if parts.len() > 5 {
            board.fullmove = parts[5].parse().unwrap_or(1);
        }

        // Update occupancy
        board.update_occupancy();
        
        // Calculate initial hash
        board.hash = board.calculate_hash();

        Ok(board)
    }

    /// Convert board to FEN string
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        
        // Piece placement
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = square(rank, file);
                if let Some((piece, color)) = self.piece_at(sq) {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let ch = match (piece, color) {
                        (PAWN, WHITE) => 'P',
                        (KNIGHT, WHITE) => 'N',
                        (BISHOP, WHITE) => 'B',
                        (ROOK, WHITE) => 'R',
                        (QUEEN, WHITE) => 'Q',
                        (KING, WHITE) => 'K',
                        (PAWN, BLACK) => 'p',
                        (KNIGHT, BLACK) => 'n',
                        (BISHOP, BLACK) => 'b',
                        (ROOK, BLACK) => 'r',
                        (QUEEN, BLACK) => 'q',
                        (KING, BLACK) => 'k',
                        _ => '?',
                    };
                    fen.push(ch);
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if self.side == WHITE { 'w' } else { 'b' });

        // Castling rights
        fen.push(' ');
        if self.castling == 0 {
            fen.push('-');
        } else {
            if self.castling & CASTLE_WK != 0 { fen.push('K'); }
            if self.castling & CASTLE_WQ != 0 { fen.push('Q'); }
            if self.castling & CASTLE_BK != 0 { fen.push('k'); }
            if self.castling & CASTLE_BQ != 0 { fen.push('q'); }
        }

        // En-passant
        fen.push(' ');
        if let Some(ep) = self.en_passant {
            fen.push_str(&square_name(ep));
        } else {
            fen.push('-');
        }

        // Halfmove and fullmove
        fen.push_str(&format!(" {} {}", self.halfmove, self.fullmove));

        fen
    }

    /// Get piece and color at a square
    #[inline]
    pub fn piece_at(&self, sq: u8) -> Option<(u8, u8)> {
        let mask = bit_at(sq);
        for color in 0..2 {
            if self.occupancy[color] & mask != 0 {
                for piece in 0..6 {
                    if self.pieces[color][piece as usize] & mask != 0 {
                        return Some((piece, color as u8));
                    }
                }
            }
        }
        None
    }

    /// Update occupancy bitboards
    #[inline]
    fn update_occupancy(&mut self) {
        self.occupancy[0] = self.pieces[0].iter().fold(0, |acc, &bb| acc | bb);
        self.occupancy[1] = self.pieces[1].iter().fold(0, |acc, &bb| acc | bb);
    }

    /// Get all occupied squares
    #[inline]
    pub fn all_occupancy(&self) -> u64 {
        self.occupancy[0] | self.occupancy[1]
    }

    /// Calculate Zobrist hash from scratch
    fn calculate_hash(&self) -> u64 {
        let z = zobrist();
        let mut hash = 0u64;

        // Hash pieces
        for color in 0..2 {
            for piece in 0..6 {
                let mut bb = self.pieces[color][piece];
                while bb != 0 {
                    let sq = pop_lsb(&mut bb);
                    let idx = Zobrist::piece_index(piece as u8, color as u8);
                    hash ^= z.pieces[idx][sq as usize];
                }
            }
        }

        // Hash side to move
        if self.side == BLACK {
            hash ^= z.side;
        }

        // Hash castling rights
        if self.castling & CASTLE_WK != 0 { hash ^= z.castling[0]; }
        if self.castling & CASTLE_WQ != 0 { hash ^= z.castling[1]; }
        if self.castling & CASTLE_BK != 0 { hash ^= z.castling[2]; }
        if self.castling & CASTLE_BQ != 0 { hash ^= z.castling[3]; }

        // Hash en-passant
        if let Some(ep) = self.en_passant {
            hash ^= z.en_passant[file_of(ep) as usize];
        }

        hash
    }

    /// Make a move on the board
    pub fn make_move(&mut self, mov: Move) {
        let z = zobrist();
        let from = mov.from();
        let to = mov.to();
        
        // Save history
        let hist = History {
            mov,
            captured: self.piece_at(to).map(|(p, _)| p),
            castling: self.castling,
            en_passant: self.en_passant,
            halfmove: self.halfmove,
            hash: self.hash,
        };
        self.history.push(hist);

        // Get moving piece
        let (piece, color) = self.piece_at(from).expect("No piece at from square");
        let enemy = color ^ 1;

        // Remove en-passant hash
        if let Some(ep) = self.en_passant {
            self.hash ^= z.en_passant[file_of(ep) as usize];
        }
        self.en_passant = None;

        // Update halfmove clock
        if piece == PAWN || hist.captured.is_some() {
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

        // Handle captures
        if let Some(captured_piece) = hist.captured {
            // Remove captured piece
            self.pieces[enemy as usize][captured_piece as usize] &= !bit_at(to);
            let cap_idx = Zobrist::piece_index(captured_piece, enemy);
            self.hash ^= z.pieces[cap_idx][to as usize];
            
            // Update castling rights if rook captured
            if captured_piece == ROOK {
                if to == 0 && (self.castling & CASTLE_WQ) != 0 {
                    self.hash ^= z.castling[1];
                    self.castling &= !CASTLE_WQ;
                } else if to == 7 && (self.castling & CASTLE_WK) != 0 {
                    self.hash ^= z.castling[0];
                    self.castling &= !CASTLE_WK;
                } else if to == 56 && (self.castling & CASTLE_BQ) != 0 {
                    self.hash ^= z.castling[3];
                    self.castling &= !CASTLE_BQ;
                } else if to == 63 && (self.castling & CASTLE_BK) != 0 {
                    self.hash ^= z.castling[2];
                    self.castling &= !CASTLE_BK;
                }
            }
        }

        // Move piece
        let piece_idx = Zobrist::piece_index(piece, color);
        self.pieces[color as usize][piece as usize] &= !bit_at(from);
        self.hash ^= z.pieces[piece_idx][from as usize];

        // Handle special moves
        if mov.is_castle() {
            // Castling
            let (rook_from, rook_to) = if to == 6 {
                (7u8, 5u8) // White kingside
            } else if to == 2 {
                (0u8, 3u8) // White queenside
            } else if to == 62 {
                (63u8, 61u8) // Black kingside
            } else {
                (56u8, 59u8) // Black queenside
            };
            
            // Move rook
            self.pieces[color as usize][ROOK as usize] &= !bit_at(rook_from);
            self.pieces[color as usize][ROOK as usize] |= bit_at(rook_to);
            let rook_idx = Zobrist::piece_index(ROOK, color);
            self.hash ^= z.pieces[rook_idx][rook_from as usize];
            self.hash ^= z.pieces[rook_idx][rook_to as usize];
            
            // Place king
            self.pieces[color as usize][piece as usize] |= bit_at(to);
            self.hash ^= z.pieces[piece_idx][to as usize];
        } else if mov.is_en_passant() {
            // En-passant capture
            let captured_sq = square(rank_of(from), file_of(to));
            self.pieces[enemy as usize][PAWN as usize] &= !bit_at(captured_sq);
            let cap_idx = Zobrist::piece_index(PAWN, enemy);
            self.hash ^= z.pieces[cap_idx][captured_sq as usize];
            
            // Place pawn
            self.pieces[color as usize][piece as usize] |= bit_at(to);
            self.hash ^= z.pieces[piece_idx][to as usize];
        } else if mov.is_promotion() {
            // Promotion
            let promo_piece = mov.promotion();
            self.pieces[color as usize][promo_piece as usize] |= bit_at(to);
            let promo_idx = Zobrist::piece_index(promo_piece, color);
            self.hash ^= z.pieces[promo_idx][to as usize];
        } else {
            // Normal move
            self.pieces[color as usize][piece as usize] |= bit_at(to);
            self.hash ^= z.pieces[piece_idx][to as usize];
            
            // Check for pawn double push
            if piece == PAWN && distance(from, to) == 2 {
                let ep_sq = square((rank_of(from) + rank_of(to)) / 2, file_of(from));
                self.en_passant = Some(ep_sq);
                self.hash ^= z.en_passant[file_of(ep_sq) as usize];
            }
        }

        // Update castling rights if king or rook moved
        if piece == KING {
            if color == WHITE {
                if (self.castling & CASTLE_WK) != 0 {
                    self.hash ^= z.castling[0];
                    self.castling &= !CASTLE_WK;
                }
                if (self.castling & CASTLE_WQ) != 0 {
                    self.hash ^= z.castling[1];
                    self.castling &= !CASTLE_WQ;
                }
            } else {
                if (self.castling & CASTLE_BK) != 0 {
                    self.hash ^= z.castling[2];
                    self.castling &= !CASTLE_BK;
                }
                if (self.castling & CASTLE_BQ) != 0 {
                    self.hash ^= z.castling[3];
                    self.castling &= !CASTLE_BQ;
                }
            }
        } else if piece == ROOK {
            if from == 0 && (self.castling & CASTLE_WQ) != 0 {
                self.hash ^= z.castling[1];
                self.castling &= !CASTLE_WQ;
            } else if from == 7 && (self.castling & CASTLE_WK) != 0 {
                self.hash ^= z.castling[0];
                self.castling &= !CASTLE_WK;
            } else if from == 56 && (self.castling & CASTLE_BQ) != 0 {
                self.hash ^= z.castling[3];
                self.castling &= !CASTLE_BQ;
            } else if from == 63 && (self.castling & CASTLE_BK) != 0 {
                self.hash ^= z.castling[2];
                self.castling &= !CASTLE_BK;
            }
        }

        // Switch side
        self.side ^= 1;
        self.hash ^= z.side;
        
        if self.side == WHITE {
            self.fullmove += 1;
        }

        // Update occupancy
        self.update_occupancy();
    }

    /// Unmake the last move
    pub fn unmake_move(&mut self) {
        let z = zobrist();
        
        let hist = self.history.pop().expect("No move to unmake");
        let mov = hist.mov;
        let from = mov.from();
        let to = mov.to();

        // Switch side back
        self.side ^= 1;
        self.hash ^= z.side;
        
        if self.side == BLACK {
            self.fullmove -= 1;
        }

        let color = self.side;
        let enemy = color ^ 1;
        let (piece, _) = if mov.is_promotion() {
            (PAWN, color)
        } else {
            self.piece_at(to).unwrap_or((PAWN, color))
        };

        // Restore castling hash
        let old_castle_diff = self.castling ^ hist.castling;
        if old_castle_diff & CASTLE_WK != 0 { self.hash ^= z.castling[0]; }
        if old_castle_diff & CASTLE_WQ != 0 { self.hash ^= z.castling[1]; }
        if old_castle_diff & CASTLE_BK != 0 { self.hash ^= z.castling[2]; }
        if old_castle_diff & CASTLE_BQ != 0 { self.hash ^= z.castling[3]; }

        // Restore en-passant hash
        if let Some(ep) = self.en_passant {
            self.hash ^= z.en_passant[file_of(ep) as usize];
        }
        if let Some(ep) = hist.en_passant {
            self.hash ^= z.en_passant[file_of(ep) as usize];
        }

        // Unmake move
        if mov.is_castle() {
            // Unmake castling
            let (rook_from, rook_to) = if to == 6 {
                (7u8, 5u8)
            } else if to == 2 {
                (0u8, 3u8)
            } else if to == 62 {
                (63u8, 61u8)
            } else {
                (56u8, 59u8)
            };
            
            // Move rook back
            self.pieces[color as usize][ROOK as usize] &= !bit_at(rook_to);
            self.pieces[color as usize][ROOK as usize] |= bit_at(rook_from);
            let rook_idx = Zobrist::piece_index(ROOK, color);
            self.hash ^= z.pieces[rook_idx][rook_to as usize];
            self.hash ^= z.pieces[rook_idx][rook_from as usize];
            
            // Move king back
            self.pieces[color as usize][KING as usize] &= !bit_at(to);
            self.pieces[color as usize][KING as usize] |= bit_at(from);
            let king_idx = Zobrist::piece_index(KING, color);
            self.hash ^= z.pieces[king_idx][to as usize];
            self.hash ^= z.pieces[king_idx][from as usize];
        } else if mov.is_en_passant() {
            // Unmake en-passant
            let captured_sq = square(rank_of(from), file_of(to));
            self.pieces[color as usize][PAWN as usize] &= !bit_at(to);
            self.pieces[color as usize][PAWN as usize] |= bit_at(from);
            self.pieces[enemy as usize][PAWN as usize] |= bit_at(captured_sq);
            
            let pawn_idx = Zobrist::piece_index(PAWN, color);
            let cap_idx = Zobrist::piece_index(PAWN, enemy);
            self.hash ^= z.pieces[pawn_idx][to as usize];
            self.hash ^= z.pieces[pawn_idx][from as usize];
            self.hash ^= z.pieces[cap_idx][captured_sq as usize];
        } else if mov.is_promotion() {
            // Unmake promotion
            let promo_piece = mov.promotion();
            self.pieces[color as usize][promo_piece as usize] &= !bit_at(to);
            self.pieces[color as usize][PAWN as usize] |= bit_at(from);
            
            let promo_idx = Zobrist::piece_index(promo_piece, color);
            let pawn_idx = Zobrist::piece_index(PAWN, color);
            self.hash ^= z.pieces[promo_idx][to as usize];
            self.hash ^= z.pieces[pawn_idx][from as usize];
            
            // Restore captured piece
            if let Some(captured) = hist.captured {
                self.pieces[enemy as usize][captured as usize] |= bit_at(to);
                let cap_idx = Zobrist::piece_index(captured, enemy);
                self.hash ^= z.pieces[cap_idx][to as usize];
            }
        } else {
            // Unmake normal move
            self.pieces[color as usize][piece as usize] &= !bit_at(to);
            self.pieces[color as usize][piece as usize] |= bit_at(from);
            
            let piece_idx = Zobrist::piece_index(piece, color);
            self.hash ^= z.pieces[piece_idx][to as usize];
            self.hash ^= z.pieces[piece_idx][from as usize];
            
            // Restore captured piece
            if let Some(captured) = hist.captured {
                self.pieces[enemy as usize][captured as usize] |= bit_at(to);
                let cap_idx = Zobrist::piece_index(captured, enemy);
                self.hash ^= z.pieces[cap_idx][to as usize];
            }
        }

        // Restore state
        self.castling = hist.castling;
        self.en_passant = hist.en_passant;
        self.halfmove = hist.halfmove;

        // Update occupancy
        self.update_occupancy();
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
