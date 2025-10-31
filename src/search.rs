//! Search module implementing iterative deepening, alpha-beta, and quiescence search.

use crate::board::*;
use crate::eval::*;
use crate::movegen::*;
use crate::tt::*;
use crate::utils::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const MAX_DEPTH: u8 = 64;
const MATE_SCORE: i32 = 30000;
const MAX_PLY: usize = 128;

/// Search limits
pub struct SearchLimits {
    pub depth: Option<u8>,
    pub movetime: Option<u128>,
    pub nodes: Option<u64>,
}

/// Search statistics
pub struct SearchInfo {
    pub nodes: u64,
    pub depth: u8,
    pub seldepth: u8,
    pub score: i32,
    pub pv: Vec<Move>,
    pub time_ms: u128,
}

/// Move ordering scores
struct MoveScorer {
    killer_moves: [[Option<Move>; 2]; MAX_PLY],
    history: [[[i32; 64]; 64]; 2],
}

impl MoveScorer {
    fn new() -> Self {
        MoveScorer {
            killer_moves: [[None; 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
        }
    }

    fn clear(&mut self) {
        self.killer_moves = [[None; 2]; MAX_PLY];
        self.history = [[[0; 64]; 64]; 2];
    }

    fn score_move(&self, board: &Board, mov: Move, tt_move: Option<Move>, ply: usize) -> i32 {
        // TT move gets highest priority
        if Some(mov) == tt_move {
            return 10_000_000;
        }

        // MVV-LVA for captures
        if let Some((captured, _)) = board.piece_at(mov.to()) {
            if let Some((attacker, _)) = board.piece_at(mov.from()) {
                return 1_000_000 + (captured as i32) * 100 - (attacker as i32);
            }
        }

        // Promotions
        if mov.is_promotion() {
            return 900_000 + (mov.promotion() as i32) * 100;
        }

        // Killer moves
        if ply < MAX_PLY {
            if Some(mov) == self.killer_moves[ply][0] {
                return 800_000;
            }
            if Some(mov) == self.killer_moves[ply][1] {
                return 700_000;
            }
        }

        // History heuristic
        let from = mov.from() as usize;
        let to = mov.to() as usize;
        self.history[board.side as usize][from][to]
    }

    fn update_killer(&mut self, mov: Move, ply: usize) {
        if ply < MAX_PLY {
            if Some(mov) != self.killer_moves[ply][0] {
                self.killer_moves[ply][1] = self.killer_moves[ply][0];
                self.killer_moves[ply][0] = Some(mov);
            }
        }
    }

    fn update_history(&mut self, color: u8, mov: Move, depth: u8) {
        let from = mov.from() as usize;
        let to = mov.to() as usize;
        self.history[color as usize][from][to] += (depth as i32) * (depth as i32);
    }
}

/// Chess engine searcher
pub struct Searcher {
    pub tt: TranspositionTable,
    scorer: MoveScorer,
    stop_flag: Arc<AtomicBool>,
    timer: Timer,
    pub info: SearchInfo,
    ply: usize,
}

impl Searcher {
    pub fn new(tt_size_mb: usize) -> Self {
        Searcher {
            tt: TranspositionTable::new(tt_size_mb),
            scorer: MoveScorer::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            timer: Timer::new(),
            info: SearchInfo {
                nodes: 0,
                depth: 0,
                seldepth: 0,
                score: 0,
                pv: Vec::new(),
                time_ms: 0,
            },
            ply: 0,
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }

    /// Search with iterative deepening
    pub fn search(&mut self, board: &mut Board, limits: SearchLimits) -> Move {
        self.stop_flag.store(false, Ordering::Relaxed);
        self.timer = Timer::new();
        self.info.nodes = 0;
        self.scorer.clear();
        self.ply = 0;

        let max_depth = limits.depth.unwrap_or(MAX_DEPTH).min(MAX_DEPTH);
        let mut best_move = Move::new(0, 0);

        // Iterative deepening
        for depth in 1..=max_depth {
            if self.is_stopped() {
                break;
            }

            self.info.depth = depth;
            self.info.seldepth = depth;

            let score = self.alpha_beta(board, depth, 0, -MATE_SCORE, MATE_SCORE, true);

            if self.is_stopped() {
                break;
            }

            self.info.score = score;
            self.info.time_ms = self.timer.elapsed_ms();

            // Extract PV from TT
            self.info.pv = self.extract_pv(board, depth);
            
            if !self.info.pv.is_empty() {
                best_move = self.info.pv[0];
            }

            // Print info
            self.print_info();

            // Check time limit
            if let Some(movetime) = limits.movetime {
                if self.timer.elapsed_ms() >= movetime {
                    break;
                }
            }

            // Check mate score
            if score.abs() > MATE_SCORE - 100 {
                break;
            }
        }

        best_move
    }

    /// Alpha-beta search with transposition table
    fn alpha_beta(
        &mut self,
        board: &mut Board,
        depth: u8,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        pv_node: bool,
    ) -> i32 {
        if ply > 0 && (self.is_stopped() || ply >= MAX_PLY) {
            return evaluate(board);
        }

        // Update selective depth
        if ply as u8 > self.info.seldepth {
            self.info.seldepth = ply as u8;
        }

        // Check for draw by repetition or 50-move rule
        if board.halfmove >= 100 {
            return 0;
        }

        // Probe transposition table
        let tt_entry = self.tt.probe(board.hash);
        let tt_move = tt_entry.and_then(|e| e.best_move);

        if let Some(entry) = tt_entry {
            if !pv_node && entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower if entry.score >= beta => return entry.score,
                    Bound::Upper if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
        }

        // Quiescence search at leaf nodes
        if depth == 0 {
            return self.quiescence(board, ply, alpha, beta);
        }

        // Generate moves
        let mut moves = Vec::with_capacity(64);
        generate_moves(board, &mut moves);

        // Filter to legal moves
        let mut legal_moves = Vec::with_capacity(moves.len());
        for mov in moves {
            board.make_move(mov);
            if !in_check(board) {
                legal_moves.push(mov);
            }
            board.unmake_move();
        }

        // Checkmate or stalemate
        if legal_moves.is_empty() {
            return if in_check(board) {
                -MATE_SCORE + ply as i32
            } else {
                0
            };
        }

        // Order moves
        self.order_moves(board, &mut legal_moves, tt_move, ply);

        let mut best_score = -MATE_SCORE;
        let mut best_move = legal_moves[0];
        let mut bound = Bound::Upper;

        for mov in legal_moves {
            board.make_move(mov);
            self.info.nodes += 1;

            let score = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, pv_node && best_score == -MATE_SCORE);

            board.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mov;

                if score > alpha {
                    alpha = score;
                    bound = Bound::Exact;

                    if score >= beta {
                        bound = Bound::Lower;
                        // Beta cutoff - update move ordering heuristics
                        if board.piece_at(mov.to()).is_none() {
                            self.scorer.update_killer(mov, ply);
                            self.scorer.update_history(board.side, mov, depth);
                        }
                        break;
                    }
                }
            }
        }

        // Store in transposition table
        self.tt.store(board.hash, depth, best_score, Some(best_move), bound);

        best_score
    }

    /// Quiescence search (only captures)
    fn quiescence(&mut self, board: &mut Board, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        self.info.nodes += 1;

        let stand_pat = evaluate(board);

        if stand_pat >= beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // Generate captures only
        let mut moves = Vec::with_capacity(32);
        generate_captures(board, &mut moves);

        // Filter to legal moves
        let mut legal_moves = Vec::with_capacity(moves.len());
        for mov in moves {
            board.make_move(mov);
            if !in_check(board) {
                legal_moves.push(mov);
            }
            board.unmake_move();
        }

        // Order moves by MVV-LVA
        self.order_moves(board, &mut legal_moves, None, ply);

        for mov in legal_moves {
            board.make_move(mov);

            let score = -self.quiescence(board, ply + 1, -beta, -alpha);

            board.unmake_move();

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Order moves for better alpha-beta pruning
    fn order_moves(&self, board: &Board, moves: &mut [Move], tt_move: Option<Move>, ply: usize) {
        let mut scored: Vec<(Move, i32)> = moves
            .iter()
            .map(|&mov| (mov, self.scorer.score_move(board, mov, tt_move, ply)))
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        for (i, (mov, _)) in scored.into_iter().enumerate() {
            moves[i] = mov;
        }
    }

    /// Extract principal variation from transposition table
    fn extract_pv(&self, board: &mut Board, max_depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut seen = std::collections::HashSet::new();
        seen.insert(board.hash);

        for _ in 0..max_depth {
            if let Some(entry) = self.tt.probe(board.hash) {
                if let Some(mov) = entry.best_move {
                    // Make the move
                    let original_side = board.side;
                    board.make_move(mov);
                    
                    // Check if it's legal (didn't leave our king in check)
                    let king_sq = lsb(board.pieces[original_side as usize][KING as usize]);
                    let is_legal = if king_sq < 64 {
                        !is_square_attacked(board, king_sq, board.side)
                    } else {
                        false
                    };
                    
                    if is_legal && !seen.contains(&board.hash) {
                        seen.insert(board.hash);
                        pv.push(mov);
                    } else {
                        board.unmake_move();
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Unmake all moves
        for _ in 0..pv.len() {
            board.unmake_move();
        }

        pv
    }

    /// Print search information
    fn print_info(&self) {
        let nps = if self.info.time_ms > 0 {
            (self.info.nodes as u128 * 1000) / self.info.time_ms
        } else {
            0
        };

        print!(
            "info depth {} seldepth {} score cp {} nodes {} time {} nps {}",
            self.info.depth, self.info.seldepth, self.info.score, 
            self.info.nodes, self.info.time_ms, nps
        );

        if !self.info.pv.is_empty() {
            print!(" pv");
            for mov in &self.info.pv {
                print!(" {}", mov.to_string());
            }
        }

        println!();
    }
}
