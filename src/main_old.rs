//! Chess Engine - A complete terminal-based chess engine
//!
//! Example session:
//! ```text
//! $ cargo run --release
//! Chess Engine v0.1.0
//! Type 'help' for commands
//! 
//! > new
//! New game started
//! 
//! > show
//!  8 │ ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
//!  7 │ ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
//!  6 │ . . . . . . . .
//!  5 │ . . . . . . . .
//!  4 │ . . . . . . . .
//!  3 │ . . . . . . . .
//!  2 │ ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
//!  1 │ ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
//!    └─────────────────
//!      a b c d e f g h
//! 
//! > e2e4
//! Move made: e2e4
//! 
//! > go depth 6
//! info depth 1 seldepth 1 score cp 15 nodes 24 time 0 nps 0 pv e7e5
//! info depth 2 seldepth 2 score cp 8 nodes 123 time 1 nps 123000 pv e7e5 g1f3
//! ...
//! Best move: e7e5
//! 
//! > perft 4
//! Nodes: 197281 Time: 0.123s NPS: 1603089
//! 
//! > quit
//! ```

mod board;
mod eval;
mod movegen;
mod perft;
mod search;
mod tt;
mod utils;
mod zobrist;

use board::*;
use movegen::*;
use perft::*;
use search::*;
use utils::*;

use std::io::{self, Write};

/// Display the board with Unicode pieces
fn display_board(board: &Board, use_unicode: bool) {
    println!();
    for rank in (0..8).rev() {
        print!(" {} │", rank + 1);
        for file in 0..8 {
            let sq = square(rank, file);
            if let Some((piece, color)) = board.piece_at(sq) {
                let ch = if use_unicode {
                    match (piece, color) {
                        (PAWN, WHITE) => '♙',
                        (KNIGHT, WHITE) => '♘',
                        (BISHOP, WHITE) => '♗',
                        (ROOK, WHITE) => '♖',
                        (QUEEN, WHITE) => '♕',
                        (KING, WHITE) => '♔',
                        (PAWN, BLACK) => '♟',
                        (KNIGHT, BLACK) => '♞',
                        (BISHOP, BLACK) => '♝',
                        (ROOK, BLACK) => '♜',
                        (QUEEN, BLACK) => '♛',
                        (KING, BLACK) => '♚',
                        _ => '?',
                    }
                } else {
                    match (piece, color) {
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
                    }
                };
                print!(" {}", ch);
            } else {
                print!(" .");
            }
        }
        println!();
    }
    println!("   └─────────────────");
    println!("     a b c d e f g h");
    println!();
    println!("FEN: {}", board.to_fen());
    println!("Side to move: {}", if board.side == WHITE { "White" } else { "Black" });
    
    if in_check(board) {
        println!("CHECK!");
    }
    
    println!();
}

/// Print help message
fn print_help() {
    println!("Available commands:");
    println!("  new              - Start a new game");
    println!("  show             - Display the board");
    println!("  fen <fen>        - Load position from FEN");
    println!("  move <move>      - Make a move (e.g., e2e4, e7e8q)");
    println!("  <move>           - Make a move (shorthand)");
    println!("  undo             - Undo the last move");
    println!("  go depth <n>     - Search to depth n");
    println!("  go movetime <ms> - Search for ms milliseconds");
    println!("  perft <depth>    - Run perft test");
    println!("  divide <depth>   - Run perft divide");
    println!("  eval             - Show static evaluation");
    println!("  legal            - Show all legal moves");
    println!("  ascii            - Toggle ASCII/Unicode display");
    println!("  help             - Show this help");
    println!("  quit             - Exit the program");
}

/// Main REPL
fn main() {
    println!("Chess Engine v0.1.0");
    println!("Type 'help' for commands");
    println!();

    let mut board = Board::starting_position();
    let mut searcher = Searcher::new(64); // 64 MB transposition table
    let mut use_unicode = true;

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }

            "help" | "h" => {
                print_help();
            }

            "new" => {
                board = Board::starting_position();
                searcher.tt.clear();
                println!("New game started");
            }

            "show" | "display" | "d" => {
                display_board(&board, use_unicode);
            }

            "ascii" => {
                use_unicode = !use_unicode;
                println!("Display mode: {}", if use_unicode { "Unicode" } else { "ASCII" });
            }

            "fen" => {
                if parts.len() < 2 {
                    println!("Usage: fen <fen-string>");
                    continue;
                }
                let fen = parts[1..].join(" ");
                match Board::from_fen(&fen) {
                    Ok(new_board) => {
                        board = new_board;
                        searcher.tt.clear();
                        println!("Position loaded");
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }

            "move" | "m" => {
                let move_str = if parts.len() > 1 {
                    parts[1]
                } else {
                    println!("Usage: move <move>");
                    continue;
                };

                if let Some(mov) = Move::from_string(move_str) {
                    let legal_moves = generate_legal_moves(&mut board);
                    
                    // Find matching legal move (need to check flags)
                    let mut found = None;
                    for &legal_mov in &legal_moves {
                        if legal_mov.from() == mov.from() && legal_mov.to() == mov.to() {
                            // Check if promotion matches
                            if mov.is_promotion() {
                                if legal_mov.is_promotion() && legal_mov.promotion() == mov.promotion() {
                                    found = Some(legal_mov);
                                    break;
                                }
                            } else {
                                found = Some(legal_mov);
                                break;
                            }
                        }
                    }

                    if let Some(legal_mov) = found {
                        board.make_move(legal_mov);
                        println!("Move made: {}", legal_mov.to_string());
                    } else {
                        println!("Illegal move: {}", move_str);
                    }
                } else {
                    println!("Invalid move format: {}", move_str);
                }
            }

            "undo" | "u" => {
                if !board.history.is_empty() {
                    board.unmake_move();
                    println!("Move undone");
                } else {
                    println!("No moves to undo");
                }
            }

            "go" => {
                if parts.len() < 3 {
                    println!("Usage: go depth <n> | go movetime <ms>");
                    continue;
                }

                let limits = match parts[1] {
                    "depth" => {
                        let depth = parts[2].parse::<u8>().unwrap_or(6);
                        SearchLimits {
                            depth: Some(depth),
                            movetime: None,
                            nodes: None,
                        }
                    }
                    "movetime" => {
                        let movetime = parts[2].parse::<u128>().unwrap_or(1000);
                        SearchLimits {
                            depth: None,
                            movetime: Some(movetime),
                            nodes: None,
                        }
                    }
                    _ => {
                        println!("Unknown go option: {}", parts[1]);
                        continue;
                    }
                };

                let best_move = searcher.search(&mut board, limits);
                println!("bestmove {}", best_move.to_string());
            }

            "perft" => {
                if parts.len() < 2 {
                    println!("Usage: perft <depth>");
                    continue;
                }

                let depth = parts[1].parse::<u8>().unwrap_or(5);
                let timer = Timer::new();
                let nodes = perft(&mut board, depth);
                let elapsed = timer.elapsed_secs();
                let nps = if elapsed > 0.0 {
                    (nodes as f64 / elapsed) as u64
                } else {
                    0
                };

                println!("Nodes: {} Time: {:.3}s NPS: {}", nodes, elapsed, nps);
            }

            "divide" => {
                if parts.len() < 2 {
                    println!("Usage: divide <depth>");
                    continue;
                }

                let depth = parts[1].parse::<u8>().unwrap_or(4);
                perft_divide(&mut board, depth);
            }

            "eval" | "e" => {
                let score = eval::evaluate(&board);
                println!("Evaluation: {} (from {} perspective)", 
                         score, 
                         if board.side == WHITE { "white" } else { "black" });
            }

            "legal" => {
                let legal_moves = generate_legal_moves(&mut board);
                println!("Legal moves ({}):", legal_moves.len());
                for mov in legal_moves {
                    print!("{} ", mov.to_string());
                }
                println!();
            }

            _ => {
                // Try to parse as a move
                if let Some(mov) = Move::from_string(&command) {
                    let legal_moves = generate_legal_moves(&mut board);
                    
                    let mut found = None;
                    for &legal_mov in &legal_moves {
                        if legal_mov.from() == mov.from() && legal_mov.to() == mov.to() {
                            if mov.is_promotion() {
                                if legal_mov.is_promotion() && legal_mov.promotion() == mov.promotion() {
                                    found = Some(legal_mov);
                                    break;
                                }
                            } else {
                                found = Some(legal_mov);
                                break;
                            }
                        }
                    }

                    if let Some(legal_mov) = found {
                        board.make_move(legal_mov);
                        println!("Move made: {}", legal_mov.to_string());
                    } else {
                        println!("Unknown command or illegal move: {}", command);
                        println!("Type 'help' for available commands");
                    }
                } else {
                    println!("Unknown command: {}", command);
                    println!("Type 'help' for available commands");
                }
            }
        }
    }
}
