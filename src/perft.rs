//! Perft (performance test) for validating move generation.

use crate::board::Board;
use crate::movegen::*;
use crate::utils::Timer;

/// Perft node counter
pub fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let legal_moves = generate_legal_moves(board);
    
    if depth == 1 {
        return legal_moves.len() as u64;
    }

    let mut nodes = 0u64;
    
    for mov in legal_moves {
        board.make_move(mov);
        nodes += perft(board, depth - 1);
        board.unmake_move();
    }

    nodes
}

/// Perft divide - shows node count for each move
pub fn perft_divide(board: &mut Board, depth: u8) {
    let timer = Timer::new();
    let legal_moves = generate_legal_moves(board);
    
    let mut total_nodes = 0u64;
    
    for mov in legal_moves {
        board.make_move(mov);
        let nodes = if depth <= 1 { 1 } else { perft(board, depth - 1) };
        board.unmake_move();
        
        println!("{}: {}", mov.to_string(), nodes);
        total_nodes += nodes;
    }

    let elapsed = timer.elapsed_secs();
    let nps = if elapsed > 0.0 {
        (total_nodes as f64 / elapsed) as u64
    } else {
        0
    };

    println!();
    println!("Nodes: {} Time: {:.3}s NPS: {}", total_nodes, elapsed, nps);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft_starting_position() {
        // Known perft results for starting position
        // Depth 1: 20, Depth 2: 400, Depth 3: 8902, Depth 4: 197281, Depth 5: 4865609
        let mut board = Board::starting_position();
        
        assert_eq!(perft(&mut board, 1), 20);
        assert_eq!(perft(&mut board, 2), 400);
        assert_eq!(perft(&mut board, 3), 8902);
        assert_eq!(perft(&mut board, 4), 197281);
    }

    #[test]
    fn test_perft_position_2() {
        // Position 2: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -
        let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        
        assert_eq!(perft(&mut board, 1), 48);
        assert_eq!(perft(&mut board, 2), 2039);
    }

    #[test]
    fn test_perft_position_3() {
        // Position 3: 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        
        assert_eq!(perft(&mut board, 1), 14);
        assert_eq!(perft(&mut board, 2), 191);
    }

    #[test]
    fn test_perft_position_4() {
        // Position 4: r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1
        let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
        
        assert_eq!(perft(&mut board, 1), 6);
        assert_eq!(perft(&mut board, 2), 264);
    }

    #[test]
    fn test_perft_position_5() {
        // Position 5: rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8
        let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        
        assert_eq!(perft(&mut board, 1), 44);
        assert_eq!(perft(&mut board, 2), 1486);
    }
}
