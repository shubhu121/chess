//! Enhanced terminal UI with colors and styling

use crate::auth::User;
use crate::board::*;
use crate::utils::*;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

pub struct UI;

impl UI {
    pub fn clear_screen() {
        // Simple clear without terminal control
        print!("\n\n\n");
    }

    pub fn print_banner() {
        println!("\n{}", "═".repeat(70));
        Self::print_colored("        ♔ ♕ CHESS ENGINE 2.0 ♛ ♚        \n", Color::Cyan, true);
        println!("{}\n", "═".repeat(70));
    }

    pub fn print_colored(text: &str, color: Color, bold: bool) {
        let _ = execute!(
            stdout(),
            SetForegroundColor(color),
        );
        if bold {
            print!("{}", text);
        } else {
            print!("{}", text);
        }
        let _ = execute!(stdout(), ResetColor);
        stdout().flush().unwrap();
    }

    pub fn print_user_info(user: &User) {
        println!("\n┌─────────────────────────────────────────────────┐");
        Self::print_colored(&format!("│ Player: {:<39} │\n", user.username), Color::Green, true);
        println!("│ Rating: {:<39} │", user.rating);
        println!("│ Games: {} (W:{} D:{} L:{}) Win Rate: {:.1}%  │",
                 user.games_played, user.games_won, user.games_drawn,
                 user.games_lost, user.win_rate());
        println!("└─────────────────────────────────────────────────┘\n");
    }

    pub fn display_board_fancy(board: &Board, show_coordinates: bool, highlight_last_move: Option<Move>) {
        println!("\n    ╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗");

        for rank in (0..8).rev() {
            print!("  {} ║", rank + 1);

            for file in 0..8 {
                let sq = square(rank, file);

                // Draw piece or empty square
                if let Some((piece, color)) = board.piece_at(sq) {
                    let ch = match (piece, color) {
                        (PAWN, WHITE) => " ♙",
                        (KNIGHT, WHITE) => " ♘",
                        (BISHOP, WHITE) => " ♗",
                        (ROOK, WHITE) => " ♖",
                        (QUEEN, WHITE) => " ♕",
                        (KING, WHITE) => " ♔",
                        (PAWN, BLACK) => " ♟",
                        (KNIGHT, BLACK) => " ♞",
                        (BISHOP, BLACK) => " ♝",
                        (ROOK, BLACK) => " ♜",
                        (QUEEN, BLACK) => " ♛",
                        (KING, BLACK) => " ♚",
                        _ => "  ",
                    };
                    print!("{} ", ch);
                } else {
                    let is_highlighted = if let Some(last_move) = highlight_last_move {
                        sq == last_move.from() || sq == last_move.to()
                    } else {
                        false
                    };
                    
                    if is_highlighted {
                        print!(" * ");
                    } else {
                        print!("   ");
                    }
                }

                if file < 7 {
                    print!("│");
                }
            }

            println!("║");

            if rank > 0 {
                println!("    ╟───┼───┼───┼───┼───┼───┼───┼───╢");
            }
        }

        println!("    ╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝");

        if show_coordinates {
            println!("      a   b   c   d   e   f   g   h\n");
        }

        // Status line
        if board.side == WHITE {
            println!("    ● White to move");
        } else {
            println!("    ● Black to move");
        }

        if crate::movegen::in_check(board) {
            println!("    ⚠️  CHECK!");
        }

        println!();
    }

    pub fn print_tip(tip: &str) {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        Self::print_colored(&format!("║ {} \n", tip), Color::Yellow, false);
        println!("╚═══════════════════════════════════════════════════════════════╝\n");
    }

    pub fn print_menu() {
        println!("\n╔════════════════════ COMMANDS ══════════════════════╗");
        println!("║ new         - Start a new game                     ║");
        println!("║ show/d      - Display the board                    ║");
        println!("║ <move>      - Make a move (e.g., e2e4, e7e8q)      ║");
        println!("║ undo/u      - Undo last move                       ║");
        println!("║ hint        - Get a move suggestion                ║");
        println!("║ tip         - Get a chess tip                      ║");
        println!("║ save        - Save current game                    ║");
        println!("║ load        - Load a saved game                    ║");
        println!("║ stats       - Show your statistics                 ║");
        println!("║ go depth N  - Computer search to depth N           ║");
        println!("║ perft N     - Run perft test                       ║");
        println!("║ eval        - Show position evaluation             ║");
        println!("║ logout      - Logout and switch user               ║");
        println!("║ help/h      - Show this menu                       ║");
        println!("║ quit/q      - Exit                                 ║");
        println!("╚════════════════════════════════════════════════════╝\n");
    }

    pub fn print_error(msg: &str) {
        Self::print_colored(&format!("❌ ERROR: {}\n", msg), Color::Red, true);
    }

    pub fn print_success(msg: &str) {
        Self::print_colored(&format!("✅ {}\n", msg), Color::Green, true);
    }

    pub fn print_info(msg: &str) {
        Self::print_colored(&format!("ℹ️  {}\n", msg), Color::Cyan, false);
    }

    pub fn prompt(msg: &str) -> String {
        Self::print_colored(msg, Color::Cyan, false);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    pub fn prompt_password(msg: &str) -> String {
        Self::print_colored(msg, Color::Cyan, false);
        // For simplicity, just use normal input
        // In production, use rpassword crate for hidden input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }
}
