//! Enhanced Chess Engine v2.0 with user authentication, save/load, and tips

mod auth;
mod board;
mod eval;
mod gamesave;
mod movegen;
mod perft;
mod search;
mod tips;
mod tt;
mod ui;
mod utils;
mod zobrist;

use auth::{AuthManager, User};
use board::*;
use gamesave::{GameManager, SavedGame};
use movegen::*;
use perft::*;
use search::*;
use tips::TipsEngine;
use ui::UI;
use utils::*;

use std::io::{self, Write};

struct GameSession {
    board: Board,
    searcher: Searcher,
    game_manager: GameManager,
    user: User,
    move_history: Vec<Move>,
    white_player: String,
    black_player: String,
    show_tips: bool,
    last_move: Option<Move>,
}

impl GameSession {
    fn new(user: User) -> Self {
        GameSession {
            board: Board::starting_position(),
            searcher: Searcher::new(64),
            game_manager: GameManager::new(),
            user,
            move_history: Vec::new(),
            white_player: String::from("Human"),
            black_player: String::from("Human"),
            show_tips: true,
            last_move: None,
        }
    }

    fn display_board(&self) {
        UI::display_board_fancy(&self.board, true, self.last_move);

        if self.show_tips {
            let tip = TipsEngine::get_tip(&self.board);
            UI::print_tip(&tip);
        }
    }

    fn make_move(&mut self, move_str: &str) -> bool {
        if let Some(mov) = Move::from_string(move_str) {
            let legal_moves = generate_legal_moves(&mut self.board);

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
                self.board.make_move(legal_mov);
                self.move_history.push(legal_mov);
                self.last_move = Some(legal_mov);
                UI::print_success(&format!("Move made: {}", legal_mov.to_string()));
                return true;
            } else {
                UI::print_error(&format!("Illegal move: {}", move_str));
            }
        } else {
            UI::print_error(&format!("Invalid move format: {}", move_str));
        }
        false
    }

    fn undo_move(&mut self) -> bool {
        if !self.board.history.is_empty() {
            self.board.unmake_move();
            self.move_history.pop();
            if !self.move_history.is_empty() {
                self.last_move = Some(*self.move_history.last().unwrap());
            } else {
                self.last_move = None;
            }
            UI::print_success("Move undone");
            true
        } else {
            UI::print_error("No moves to undo");
            false
        }
    }

    fn save_game(&self) -> Result<(), String> {
        let saved_game = SavedGame::new(
            self.user.username.clone(),
            &self.board,
            &self.move_history,
            self.white_player.clone(),
            self.black_player.clone(),
        );

        match self.game_manager.save_game(&saved_game) {
            Ok(filename) => {
                UI::print_success(&format!("Game saved as: {}", filename));
                Ok(())
            }
            Err(e) => {
                UI::print_error(&e);
                Err(e)
            }
        }
    }

    fn load_game(&mut self) {
        let saves = self.game_manager.list_saves(&self.user.username);

        if saves.is_empty() {
            UI::print_info("No saved games found");
            return;
        }

        println!("\n╔══════════════════ SAVED GAMES ═══════════════════╗");
        for (i, (filename, game)) in saves.iter().enumerate() {
            println!("║ {}. {} vs {}                    ", i + 1, game.white_player, game.black_player);
            println!("║    {} moves - {}              ", game.moves.len(), filename);
        }
        println!("╚═════════════════════════════════════════════════╝\n");

        let choice = UI::prompt("Enter game number to load (0 to cancel): ");

        if let Ok(index) = choice.parse::<usize>() {
            if index > 0 && index <= saves.len() {
                let (filename, _) = &saves[index - 1];
                match self.game_manager.load_game(filename) {
                    Ok(saved_game) => {
                        match Board::from_fen(&saved_game.fen) {
                            Ok(board) => {
                                self.board = board;
                                self.white_player = saved_game.white_player;
                                self.black_player = saved_game.black_player;
                                self.move_history.clear();
                                // Restore move history if possible
                                UI::print_success(&format!("Game loaded: {} vs {}", 
                                    self.white_player, self.black_player));
                            }
                            Err(e) => UI::print_error(&e),
                        }
                    }
                    Err(e) => UI::print_error(&e),
                }
            }
        }
    }

    fn show_stats(&self) {
        UI::print_user_info(&self.user);
    }
}

fn login_or_register(auth_manager: &mut AuthManager) -> Option<User> {
    UI::clear_screen();
    UI::print_banner();

    loop {
        println!("\n1. Login");
        println!("2. Register");
        println!("3. Exit\n");

        let choice = UI::prompt("Choose an option: ");

        match choice.as_str() {
            "1" => {
                let username = UI::prompt("Username: ");
                let password = UI::prompt_password("Password: ");

                match auth_manager.login(&username, &password) {
                    Ok(user) => {
                        UI::print_success(&format!("Welcome back, {}!", user.username));
                        return Some(user);
                    }
                    Err(e) => UI::print_error(&e),
                }
            }
            "2" => {
                let username = UI::prompt("Choose username (min 3 chars): ");
                let password = UI::prompt_password("Choose password (min 4 chars): ");
                let confirm = UI::prompt_password("Confirm password: ");

                if password != confirm {
                    UI::print_error("Passwords don't match!");
                    continue;
                }

                match auth_manager.register(username, password) {
                    Ok(user) => {
                        UI::print_success(&format!("Account created! Welcome, {}!", user.username));
                        return Some(user);
                    }
                    Err(e) => UI::print_error(&e),
                }
            }
            "3" => return None,
            _ => UI::print_error("Invalid choice"),
        }
    }
}

fn main() {
    let mut auth_manager = AuthManager::new();

    loop {
        let user = match login_or_register(&mut auth_manager) {
            Some(u) => u,
            None => {
                println!("Goodbye!");
                return;
            }
        };

        run_game_loop(&mut auth_manager, user);
    }
}

fn run_game_loop(auth_manager: &mut AuthManager, mut user: User) {
    let mut session = GameSession::new(user.clone());

    UI::clear_screen();
    UI::print_banner();
    UI::print_user_info(&session.user);
    UI::print_menu();

    session.display_board();

    loop {
        print!("\n{}> ", session.user.username);
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
                auth_manager.update_user(&session.user);
                UI::print_success("Saved progress. Logging out...");
                break;
            }

            "help" | "h" | "menu" => {
                UI::print_menu();
            }

            "new" => {
                session.board = Board::starting_position();
                session.searcher.tt.clear();
                session.move_history.clear();
                session.last_move = None;
                UI::print_success("New game started");
                session.display_board();
            }

            "show" | "display" | "d" | "board" => {
                session.display_board();
            }

            "move" | "m" => {
                if parts.len() > 1 {
                    if session.make_move(parts[1]) {
                        session.display_board();
                    }
                } else {
                    UI::print_error("Usage: move <move>");
                }
            }

            "undo" | "u" => {
                if session.undo_move() {
                    session.display_board();
                }
            }

            "hint" => {
                let hint = TipsEngine::get_hint(&mut session.board);
                UI::print_tip(&hint);
            }

            "tip" | "tips" => {
                session.show_tips = !session.show_tips;
                if session.show_tips {
                    UI::print_success("Tips enabled");
                } else {
                    UI::print_info("Tips disabled");
                }
            }

            "save" => {
                let _ = session.save_game();
            }

            "load" => {
                session.load_game();
                session.display_board();
            }

            "stats" | "profile" => {
                session.show_stats();
            }

            "logout" => {
                auth_manager.update_user(&session.user);
                UI::print_success("Logging out...");
                break;
            }

            "go" => {
                if parts.len() < 3 {
                    UI::print_error("Usage: go depth <n> | go movetime <ms>");
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
                        UI::print_error(&format!("Unknown go option: {}", parts[1]));
                        continue;
                    }
                };

                let best_move = session.searcher.search(&mut session.board, limits);
                println!("bestmove {}", best_move.to_string());
            }

            "perft" => {
                if parts.len() < 2 {
                    UI::print_error("Usage: perft <depth>");
                    continue;
                }

                let depth = parts[1].parse::<u8>().unwrap_or(5);
                let timer = Timer::new();
                let nodes = perft(&mut session.board, depth);
                let elapsed = timer.elapsed_secs();
                let nps = if elapsed > 0.0 {
                    (nodes as f64 / elapsed) as u64
                } else {
                    0
                };

                UI::print_info(&format!("Nodes: {} Time: {:.3}s NPS: {}", nodes, elapsed, nps));
            }

            "eval" | "e" => {
                let score = eval::evaluate(&session.board);
                UI::print_info(&format!("Evaluation: {} centipawns (from {} perspective)",
                    score,
                    if session.board.side == WHITE { "white" } else { "black" }));
            }

            "legal" => {
                let legal_moves = generate_legal_moves(&mut session.board);
                println!("\n Legal moves ({}):", legal_moves.len());
                for (i, mov) in legal_moves.iter().enumerate() {
                    print!("{} ", mov.to_string());
                    if (i + 1) % 8 == 0 {
                        println!();
                    }
                }
                println!("\n");
            }

            "fen" => {
                if parts.len() < 2 {
                    println!("Current FEN: {}", session.board.to_fen());
                } else {
                    let fen = parts[1..].join(" ");
                    match Board::from_fen(&fen) {
                        Ok(new_board) => {
                            session.board = new_board;
                            session.searcher.tt.clear();
                            session.move_history.clear();
                            session.last_move = None;
                            UI::print_success("Position loaded");
                            session.display_board();
                        }
                        Err(e) => UI::print_error(&e),
                    }
                }
            }

            _ => {
                // Try to parse as a move
                if let Some(_mov) = Move::from_string(&command) {
                    if session.make_move(&command) {
                        session.display_board();
                    }
                } else {
                    UI::print_error(&format!("Unknown command: {}", command));
                    UI::print_info("Type 'help' for available commands");
                }
            }
        }
    }
}
