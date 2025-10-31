//! In-game chess tips and hints

use crate::board::*;
use crate::movegen::*;
use crate::utils::*;

pub struct TipsEngine;

impl TipsEngine {
    /// Get a tip based on the current position
    pub fn get_tip(board: &Board) -> String {
        let tips = vec![
            Self::check_development_tip(board),
            Self::check_center_control_tip(board),
            Self::check_king_safety_tip(board),
            Self::check_piece_activity_tip(board),
            Self::check_tactical_tip(board),
        ];

        // Return the first relevant tip, or a general one
        for tip in tips {
            if let Some(t) = tip {
                return t;
            }
        }

        Self::get_general_tip(board)
    }

    fn check_development_tip(board: &Board) -> Option<String> {
        let color = board.side;
        let back_rank = if color == WHITE { 0 } else { 7 };

        // Check if knights and bishops are still on starting squares
        let mut undeveloped = 0;

        // Check knights
        if color == WHITE {
            if is_set(board.pieces[color as usize][KNIGHT as usize], square(0, 1)) {
                undeveloped += 1;
            }
            if is_set(board.pieces[color as usize][KNIGHT as usize], square(0, 6)) {
                undeveloped += 1;
            }
        } else {
            if is_set(board.pieces[color as usize][KNIGHT as usize], square(7, 1)) {
                undeveloped += 1;
            }
            if is_set(board.pieces[color as usize][KNIGHT as usize], square(7, 6)) {
                undeveloped += 1;
            }
        }

        // Check bishops
        if color == WHITE {
            if is_set(board.pieces[color as usize][BISHOP as usize], square(0, 2)) {
                undeveloped += 1;
            }
            if is_set(board.pieces[color as usize][BISHOP as usize], square(0, 5)) {
                undeveloped += 1;
            }
        } else {
            if is_set(board.pieces[color as usize][BISHOP as usize], square(7, 2)) {
                undeveloped += 1;
            }
            if is_set(board.pieces[color as usize][BISHOP as usize], square(7, 5)) {
                undeveloped += 1;
            }
        }

        if undeveloped >= 2 && board.fullmove >= 5 {
            return Some("💡 TIP: Develop your knights and bishops before moving the same piece twice!".to_string());
        }

        None
    }

    fn check_center_control_tip(board: &Board) -> Option<String> {
        let color = board.side;
        let center_squares = [
            square(3, 3),
            square(3, 4),
            square(4, 3),
            square(4, 4),
        ];

        let mut center_control = 0;
        for &sq in &center_squares {
            if is_set(board.pieces[color as usize][PAWN as usize], sq) {
                center_control += 2;
            } else if is_square_attacked(board, sq, color) {
                center_control += 1;
            }
        }

        if center_control == 0 && board.fullmove <= 10 {
            return Some("💡 TIP: Control the center with your pawns (e4, d4, e5, d5)!".to_string());
        }

        None
    }

    fn check_king_safety_tip(board: &Board) -> Option<String> {
        let color = board.side;

        // Check if king has castled
        let king_bb = board.pieces[color as usize][KING as usize];
        let king_sq = lsb(king_bb);

        if king_sq < 64 {
            let king_file = file_of(king_sq);

            // King still in center
            if king_file >= 3 && king_file <= 4 {
                // Check if can still castle
                if color == WHITE {
                    if (board.castling & (CASTLE_WK | CASTLE_WQ)) != 0 && board.fullmove >= 8 {
                        return Some("💡 TIP: Consider castling to get your king to safety!".to_string());
                    }
                } else if (board.castling & (CASTLE_BK | CASTLE_BQ)) != 0 && board.fullmove >= 8 {
                    return Some("💡 TIP: Consider castling to get your king to safety!".to_string());
                }
            }
        }

        None
    }

    fn check_piece_activity_tip(board: &Board) -> Option<String> {
        let color = board.side;

        // Count pieces on back rank
        let back_rank = if color == WHITE { 0 } else { 7 };
        let mut pieces_on_back_rank = 0;

        for file in 0..8 {
            let sq = square(back_rank, file);
            if let Some((piece, piece_color)) = board.piece_at(sq) {
                if piece_color == color && piece != KING && piece != ROOK {
                    pieces_on_back_rank += 1;
                }
            }
        }

        if pieces_on_back_rank >= 3 && board.fullmove >= 15 {
            return Some("💡 TIP: Activate your pieces! Move them to better squares.".to_string());
        }

        None
    }

    fn check_tactical_tip(board: &Board) -> Option<String> {
        if in_check(board) {
            return Some("⚠️  You're in CHECK! You must move your king, block, or capture the attacking piece.".to_string());
        }

        // Check if opponent's king is in check (you're giving check)
        let enemy = board.side ^ 1;
        let enemy_king_sq = lsb(board.pieces[enemy as usize][KING as usize]);
        if enemy_king_sq < 64 && is_square_attacked(board, enemy_king_sq, board.side) {
            return Some("✅ CHECK! You're attacking the opponent's king!".to_string());
        }

        None
    }

    fn get_general_tip(board: &Board) -> String {
        let tips = [
            "♟️ Opening principles: Control center, develop pieces, castle early!",
            "♞ Knights are best on the rim when they're dim - keep them central!",
            "♗ Bishops love open diagonals - don't trap them behind pawns!",
            "♜ Rooks belong on open files and the 7th rank!",
            "♛ Don't bring your queen out too early - she can be attacked!",
            "♚ Castle early to keep your king safe!",
            "⚔️ Look for tactics: forks, pins, skewers, and discoveries!",
            "🎯 Always check for captures, checks, and threats before moving!",
            "📊 Every move should have a purpose - improve your position!",
            "🔄 When you see a good move, look for a better one!",
        ];

        let index = (board.fullmove as usize + board.halfmove as usize) % tips.len();
        tips[index].to_string()
    }

    /// Get hint about possible good moves
    pub fn get_hint(board: &mut Board) -> String {
        let legal_moves = generate_legal_moves(board);

        if legal_moves.is_empty() {
            return "No legal moves available!".to_string();
        }

        // Simple heuristic: prioritize captures, checks, and center moves
        let mut best_moves = Vec::new();

        for mov in &legal_moves {
            let mut score = 0;

            // Check if it's a capture
            if board.piece_at(mov.to()).is_some() {
                score += 10;
            }

            // Check if it gives check
            board.make_move(*mov);
            if in_check(board) {
                score += 5;
            }
            board.unmake_move();

            // Check if it controls center
            let to_sq = mov.to();
            if (to_sq == 27 || to_sq == 28 || to_sq == 35 || to_sq == 36) {
                score += 3;
            }

            if score > 0 {
                best_moves.push((*mov, score));
            }
        }

        if !best_moves.is_empty() {
            best_moves.sort_by(|a, b| b.1.cmp(&a.1));
            let hint_move = best_moves[0].0;
            format!("💭 HINT: Consider {} (captures center, or gives check)", hint_move.to_string())
        } else {
            let random_move = legal_moves[board.fullmove as usize % legal_moves.len()];
            format!("💭 HINT: {} is a decent move", random_move.to_string())
        }
    }
}
