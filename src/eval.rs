//! Position evaluation using material and piece-square tables.

use crate::board::*;
use crate::utils::*;

/// Material values in centipawns
const PIECE_VALUES: [i32; 6] = [
    100,  // Pawn
    320,  // Knight
    330,  // Bishop
    500,  // Rook
    900,  // Queen
    20000, // King
];

/// Piece-square tables for positional evaluation
/// Values are from white's perspective, mirrored for black

const PAWN_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

const QUEEN_PST: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

const KING_PST_MG: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

/// Get piece-square table for a piece
const fn get_pst(piece: u8) -> &'static [i32; 64] {
    match piece {
        PAWN => &PAWN_PST,
        KNIGHT => &KNIGHT_PST,
        BISHOP => &BISHOP_PST,
        ROOK => &ROOK_PST,
        QUEEN => &QUEEN_PST,
        KING => &KING_PST_MG,
        _ => &[0; 64],
    }
}

/// Mirror square for black pieces
#[inline]
const fn mirror(sq: u8) -> usize {
    (sq ^ 56) as usize
}

/// Evaluate position from side to move perspective
pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;
    
    // Material and piece-square tables
    for piece in 0..6 {
        // White pieces
        let mut white_pieces = board.pieces[WHITE as usize][piece as usize];
        while white_pieces != 0 {
            let sq = pop_lsb(&mut white_pieces);
            score += PIECE_VALUES[piece as usize];
            score += get_pst(piece)[sq as usize];
        }
        
        // Black pieces
        let mut black_pieces = board.pieces[BLACK as usize][piece as usize];
        while black_pieces != 0 {
            let sq = pop_lsb(&mut black_pieces);
            score -= PIECE_VALUES[piece as usize];
            score -= get_pst(piece)[mirror(sq)];
        }
    }
    
    // Mobility bonus (simple)
    let mobility_bonus = calculate_mobility(board);
    score += mobility_bonus;
    
    // Return score from side to move perspective
    if board.side == WHITE {
        score
    } else {
        -score
    }
}

/// Calculate simple mobility bonus
fn calculate_mobility(board: &Board) -> i32 {
    let white_mobility = count_mobility(board, WHITE);
    let black_mobility = count_mobility(board, BLACK);
    (white_mobility - black_mobility) * 5
}

/// Count mobility for a side (number of squares attacked)
fn count_mobility(board: &Board, color: u8) -> i32 {
    let occupied = board.all_occupancy();
    let our_pieces = board.occupancy[color as usize];
    let mut mobility = 0;
    
    // Knight mobility
    let mut knights = board.pieces[color as usize][KNIGHT as usize];
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        mobility += popcount(crate::movegen::knight_attacks(sq) & !our_pieces) as i32;
    }
    
    // Bishop mobility
    let mut bishops = board.pieces[color as usize][BISHOP as usize];
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        mobility += popcount(crate::movegen::bishop_attacks(sq, occupied) & !our_pieces) as i32;
    }
    
    // Rook mobility
    let mut rooks = board.pieces[color as usize][ROOK as usize];
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        mobility += popcount(crate::movegen::rook_attacks(sq, occupied) & !our_pieces) as i32;
    }
    
    mobility
}

/// Check if position is likely drawn by insufficient material
pub fn is_insufficient_material(board: &Board) -> bool {
    // King vs King
    if board.all_occupancy().count_ones() == 2 {
        return true;
    }
    
    // King + minor vs King
    if board.all_occupancy().count_ones() == 3 {
        let has_knight = board.pieces[0][KNIGHT as usize] | board.pieces[1][KNIGHT as usize];
        let has_bishop = board.pieces[0][BISHOP as usize] | board.pieces[1][BISHOP as usize];
        if has_knight != 0 || has_bishop != 0 {
            return true;
        }
    }
    
    false
}
