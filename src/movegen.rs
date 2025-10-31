//! Move generation for all piece types including special moves.

use crate::board::*;
use crate::utils::*;

// Precomputed attack tables
static mut KNIGHT_ATTACKS: [u64; 64] = [0; 64];
static mut KING_ATTACKS: [u64; 64] = [0; 64];

static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize attack tables
fn init_attacks() {
    unsafe {
        // Knight attacks
        for sq in 0..64 {
            let mut attacks = 0u64;
            let rank = rank_of(sq) as i8;
            let file = file_of(sq) as i8;
            
            let knight_moves = [
                (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                (1, -2), (1, 2), (2, -1), (2, 1),
            ];
            
            for &(dr, df) in &knight_moves {
                let new_rank = rank + dr;
                let new_file = file + df;
                if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                    attacks |= bit_at(square(new_rank as u8, new_file as u8));
                }
            }
            
            KNIGHT_ATTACKS[sq as usize] = attacks;
        }

        // King attacks
        for sq in 0..64 {
            let mut attacks = 0u64;
            let rank = rank_of(sq) as i8;
            let file = file_of(sq) as i8;
            
            for dr in -1..=1 {
                for df in -1..=1 {
                    if dr == 0 && df == 0 {
                        continue;
                    }
                    let new_rank = rank + dr;
                    let new_file = file + df;
                    if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                        attacks |= bit_at(square(new_rank as u8, new_file as u8));
                    }
                }
            }
            
            KING_ATTACKS[sq as usize] = attacks;
        }
    }
}

/// Get knight attacks from a square
#[inline]
pub fn knight_attacks(sq: u8) -> u64 {
    INIT.call_once(init_attacks);
    unsafe { KNIGHT_ATTACKS[sq as usize] }
}

/// Get king attacks from a square
#[inline]
pub fn king_attacks(sq: u8) -> u64 {
    INIT.call_once(init_attacks);
    unsafe { KING_ATTACKS[sq as usize] }
}

/// Get pawn attacks from a square
#[inline]
pub fn pawn_attacks(sq: u8, color: u8) -> u64 {
    let rank = rank_of(sq);
    let file = file_of(sq);
    let mut attacks = 0u64;
    
    if color == WHITE {
        if rank < 7 {
            if file > 0 {
                attacks |= bit_at(square(rank + 1, file - 1));
            }
            if file < 7 {
                attacks |= bit_at(square(rank + 1, file + 1));
            }
        }
    } else {
        if rank > 0 {
            if file > 0 {
                attacks |= bit_at(square(rank - 1, file - 1));
            }
            if file < 7 {
                attacks |= bit_at(square(rank - 1, file + 1));
            }
        }
    }
    
    attacks
}

/// Get sliding attacks (rook-like) using classical approach
#[inline]
pub fn rook_attacks(sq: u8, occupied: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = rank_of(sq);
    let file = file_of(sq);
    
    // North
    for r in (rank + 1)..8 {
        let target = square(r, file);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // South
    for r in (0..rank).rev() {
        let target = square(r, file);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // East
    for f in (file + 1)..8 {
        let target = square(rank, f);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // West
    for f in (0..file).rev() {
        let target = square(rank, f);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    attacks
}

/// Get sliding attacks (bishop-like) using classical approach
#[inline]
pub fn bishop_attacks(sq: u8, occupied: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = rank_of(sq) as i8;
    let file = file_of(sq) as i8;
    
    // North-East
    for i in 1..8 {
        let r = rank + i;
        let f = file + i;
        if r >= 8 || f >= 8 {
            break;
        }
        let target = square(r as u8, f as u8);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // North-West
    for i in 1..8 {
        let r = rank + i;
        let f = file - i;
        if r >= 8 || f < 0 {
            break;
        }
        let target = square(r as u8, f as u8);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // South-East
    for i in 1..8 {
        let r = rank - i;
        let f = file + i;
        if r < 0 || f >= 8 {
            break;
        }
        let target = square(r as u8, f as u8);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    // South-West
    for i in 1..8 {
        let r = rank - i;
        let f = file - i;
        if r < 0 || f < 0 {
            break;
        }
        let target = square(r as u8, f as u8);
        attacks |= bit_at(target);
        if occupied & bit_at(target) != 0 {
            break;
        }
    }
    
    attacks
}

/// Get queen attacks (combination of rook and bishop)
#[inline]
pub fn queen_attacks(sq: u8, occupied: u64) -> u64 {
    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
}

/// Check if a square is attacked by the given side
pub fn is_square_attacked(board: &Board, sq: u8, by_color: u8) -> bool {
    let occupied = board.all_occupancy();
    
    // Pawn attacks
    let pawn_atk = pawn_attacks(sq, by_color ^ 1);
    if pawn_atk & board.pieces[by_color as usize][PAWN as usize] != 0 {
        return true;
    }
    
    // Knight attacks
    let knight_atk = knight_attacks(sq);
    if knight_atk & board.pieces[by_color as usize][KNIGHT as usize] != 0 {
        return true;
    }
    
    // King attacks
    let king_atk = king_attacks(sq);
    if king_atk & board.pieces[by_color as usize][KING as usize] != 0 {
        return true;
    }
    
    // Bishop/Queen diagonal attacks
    let bishop_atk = bishop_attacks(sq, occupied);
    if bishop_atk & (board.pieces[by_color as usize][BISHOP as usize] | 
                     board.pieces[by_color as usize][QUEEN as usize]) != 0 {
        return true;
    }
    
    // Rook/Queen straight attacks
    let rook_atk = rook_attacks(sq, occupied);
    if rook_atk & (board.pieces[by_color as usize][ROOK as usize] | 
                   board.pieces[by_color as usize][QUEEN as usize]) != 0 {
        return true;
    }
    
    false
}

/// Check if current side is in check
pub fn in_check(board: &Board) -> bool {
    let king_sq = lsb(board.pieces[board.side as usize][KING as usize]);
    if king_sq >= 64 {
        return false; // No king, invalid position
    }
    is_square_attacked(board, king_sq, board.side ^ 1)
}

/// Generate pseudo-legal moves
pub fn generate_moves(board: &Board, moves: &mut Vec<Move>) {
    let color = board.side;
    let enemy = color ^ 1;
    let occupied = board.all_occupancy();
    let our_pieces = board.occupancy[color as usize];
    let their_pieces = board.occupancy[enemy as usize];
    
    // Pawn moves
    let mut pawns = board.pieces[color as usize][PAWN as usize];
    while pawns != 0 {
        let from = pop_lsb(&mut pawns);
        let rank = rank_of(from);
        let file = file_of(from);
        
        if color == WHITE {
            // Single push
            let to = from + 8;
            if to < 64 && !is_set(occupied, to) {
                if rank == 6 {
                    // Promotion
                    moves.push(Move::with_promotion(from, to, 1)); // Knight
                    moves.push(Move::with_promotion(from, to, 2)); // Bishop
                    moves.push(Move::with_promotion(from, to, 3)); // Rook
                    moves.push(Move::with_promotion(from, to, 4)); // Queen
                } else {
                    moves.push(Move::new(from, to));
                }
            }
            
            // Double push
            if rank == 1 && !is_set(occupied, from + 8) && !is_set(occupied, from + 16) {
                moves.push(Move::new(from, from + 16));
            }
            
            // Captures
            if rank < 7 {
                if file > 0 {
                    let to = from + 7;
                    if is_set(their_pieces, to) {
                        if rank == 6 {
                            moves.push(Move::with_promotion(from, to, 1));
                            moves.push(Move::with_promotion(from, to, 2));
                            moves.push(Move::with_promotion(from, to, 3));
                            moves.push(Move::with_promotion(from, to, 4));
                        } else {
                            moves.push(Move::new(from, to));
                        }
                    } else if Some(to) == board.en_passant {
                        moves.push(Move::with_flags(from, to, 2)); // En-passant
                    }
                }
                if file < 7 {
                    let to = from + 9;
                    if is_set(their_pieces, to) {
                        if rank == 6 {
                            moves.push(Move::with_promotion(from, to, 1));
                            moves.push(Move::with_promotion(from, to, 2));
                            moves.push(Move::with_promotion(from, to, 3));
                            moves.push(Move::with_promotion(from, to, 4));
                        } else {
                            moves.push(Move::new(from, to));
                        }
                    } else if Some(to) == board.en_passant {
                        moves.push(Move::with_flags(from, to, 2)); // En-passant
                    }
                }
            }
        } else {
            // Black pawns (similar but moving down)
            // Single push
            if from >= 8 {
                let to = from - 8;
                if !is_set(occupied, to) {
                    if rank == 1 {
                        // Promotion
                        moves.push(Move::with_promotion(from, to, 1));
                        moves.push(Move::with_promotion(from, to, 2));
                        moves.push(Move::with_promotion(from, to, 3));
                        moves.push(Move::with_promotion(from, to, 4));
                    } else {
                        moves.push(Move::new(from, to));
                    }
                }
            }
            
            // Double push
            if rank == 6 && from >= 16 && !is_set(occupied, from - 8) && !is_set(occupied, from - 16) {
                moves.push(Move::new(from, from - 16));
            }
            
            // Captures
            if rank > 0 {
                if file > 0 && from >= 9 {
                    let to = from - 9;
                    if is_set(their_pieces, to) {
                        if rank == 1 {
                            moves.push(Move::with_promotion(from, to, 1));
                            moves.push(Move::with_promotion(from, to, 2));
                            moves.push(Move::with_promotion(from, to, 3));
                            moves.push(Move::with_promotion(from, to, 4));
                        } else {
                            moves.push(Move::new(from, to));
                        }
                    } else if Some(to) == board.en_passant {
                        moves.push(Move::with_flags(from, to, 2));
                    }
                }
                if file < 7 && from >= 7 {
                    let to = from - 7;
                    if is_set(their_pieces, to) {
                        if rank == 1 {
                            moves.push(Move::with_promotion(from, to, 1));
                            moves.push(Move::with_promotion(from, to, 2));
                            moves.push(Move::with_promotion(from, to, 3));
                            moves.push(Move::with_promotion(from, to, 4));
                        } else {
                            moves.push(Move::new(from, to));
                        }
                    } else if Some(to) == board.en_passant {
                        moves.push(Move::with_flags(from, to, 2));
                    }
                }
            }
        }
    }
    
    // Knight moves
    let mut knights = board.pieces[color as usize][KNIGHT as usize];
    while knights != 0 {
        let from = pop_lsb(&mut knights);
        let mut attacks = knight_attacks(from) & !our_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Bishop moves
    let mut bishops = board.pieces[color as usize][BISHOP as usize];
    while bishops != 0 {
        let from = pop_lsb(&mut bishops);
        let mut attacks = bishop_attacks(from, occupied) & !our_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Rook moves
    let mut rooks = board.pieces[color as usize][ROOK as usize];
    while rooks != 0 {
        let from = pop_lsb(&mut rooks);
        let mut attacks = rook_attacks(from, occupied) & !our_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Queen moves
    let mut queens = board.pieces[color as usize][QUEEN as usize];
    while queens != 0 {
        let from = pop_lsb(&mut queens);
        let mut attacks = queen_attacks(from, occupied) & !our_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // King moves
    let king_sq = lsb(board.pieces[color as usize][KING as usize]);
    let mut attacks = king_attacks(king_sq) & !our_pieces;
    while attacks != 0 {
        let to = pop_lsb(&mut attacks);
        moves.push(Move::new(king_sq, to));
    }
    
    // Castling
    if color == WHITE {
        // Kingside
        if board.castling & CASTLE_WK != 0 {
            if !is_set(occupied, 5) && !is_set(occupied, 6) {
                if !is_square_attacked(board, 4, BLACK) &&
                   !is_square_attacked(board, 5, BLACK) &&
                   !is_square_attacked(board, 6, BLACK) {
                    moves.push(Move::with_flags(4, 6, 1));
                }
            }
        }
        // Queenside
        if board.castling & CASTLE_WQ != 0 {
            if !is_set(occupied, 1) && !is_set(occupied, 2) && !is_set(occupied, 3) {
                if !is_square_attacked(board, 4, BLACK) &&
                   !is_square_attacked(board, 3, BLACK) &&
                   !is_square_attacked(board, 2, BLACK) {
                    moves.push(Move::with_flags(4, 2, 1));
                }
            }
        }
    } else {
        // Kingside
        if board.castling & CASTLE_BK != 0 {
            if !is_set(occupied, 61) && !is_set(occupied, 62) {
                if !is_square_attacked(board, 60, WHITE) &&
                   !is_square_attacked(board, 61, WHITE) &&
                   !is_square_attacked(board, 62, WHITE) {
                    moves.push(Move::with_flags(60, 62, 1));
                }
            }
        }
        // Queenside
        if board.castling & CASTLE_BQ != 0 {
            if !is_set(occupied, 57) && !is_set(occupied, 58) && !is_set(occupied, 59) {
                if !is_square_attacked(board, 60, WHITE) &&
                   !is_square_attacked(board, 59, WHITE) &&
                   !is_square_attacked(board, 58, WHITE) {
                    moves.push(Move::with_flags(60, 58, 1));
                }
            }
        }
    }
}

/// Generate only capture moves (for quiescence search)
pub fn generate_captures(board: &Board, moves: &mut Vec<Move>) {
    let color = board.side;
    let enemy = color ^ 1;
    let occupied = board.all_occupancy();
    let their_pieces = board.occupancy[enemy as usize];
    
    // Pawn captures
    let mut pawns = board.pieces[color as usize][PAWN as usize];
    while pawns != 0 {
        let from = pop_lsb(&mut pawns);
        let mut attacks = pawn_attacks(from, color) & their_pieces;
        let rank = rank_of(from);
        
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            if (color == WHITE && rank == 6) || (color == BLACK && rank == 1) {
                moves.push(Move::with_promotion(from, to, 4)); // Queen only for captures
            } else {
                moves.push(Move::new(from, to));
            }
        }
        
        // En-passant
        if let Some(ep) = board.en_passant {
            if pawn_attacks(from, color) & bit_at(ep) != 0 {
                moves.push(Move::with_flags(from, ep, 2));
            }
        }
    }
    
    // Knight captures
    let mut knights = board.pieces[color as usize][KNIGHT as usize];
    while knights != 0 {
        let from = pop_lsb(&mut knights);
        let mut attacks = knight_attacks(from) & their_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Bishop captures
    let mut bishops = board.pieces[color as usize][BISHOP as usize];
    while bishops != 0 {
        let from = pop_lsb(&mut bishops);
        let mut attacks = bishop_attacks(from, occupied) & their_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Rook captures
    let mut rooks = board.pieces[color as usize][ROOK as usize];
    while rooks != 0 {
        let from = pop_lsb(&mut rooks);
        let mut attacks = rook_attacks(from, occupied) & their_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // Queen captures
    let mut queens = board.pieces[color as usize][QUEEN as usize];
    while queens != 0 {
        let from = pop_lsb(&mut queens);
        let mut attacks = queen_attacks(from, occupied) & their_pieces;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from, to));
        }
    }
    
    // King captures
    let king_sq = lsb(board.pieces[color as usize][KING as usize]);
    let mut attacks = king_attacks(king_sq) & their_pieces;
    while attacks != 0 {
        let to = pop_lsb(&mut attacks);
        moves.push(Move::new(king_sq, to));
    }
}

/// Filter pseudo-legal moves to only legal moves
pub fn generate_legal_moves(board: &mut Board) -> Vec<Move> {
    let mut pseudo_legal = Vec::with_capacity(64);
    generate_moves(board, &mut pseudo_legal);
    
    let original_side = board.side;
    let mut legal = Vec::with_capacity(pseudo_legal.len());
    for mov in pseudo_legal {
        board.make_move(mov);
        // After make_move, side has switched, so check if original side's king is attacked
        let king_sq = lsb(board.pieces[original_side as usize][KING as usize]);
        let is_legal = if king_sq < 64 {
            !is_square_attacked(board, king_sq, board.side)
        } else {
            false
        };
        if is_legal {
            legal.push(mov);
        }
        board.unmake_move();
    }
    
    legal
}
