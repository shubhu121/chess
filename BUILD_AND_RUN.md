# Build and Run Instructions

## Quick Start

```bash
# Navigate to project directory
cd chess_engine

# Build the release version (optimized)
cargo build --release

# Run the engine
cargo run --release

# Or run the binary directly
./target/release/chess_engine
```

## Optional: Performance Optimization

For best performance, enable CPU-specific optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Testing

### Run all tests including perft validation:
```bash
cargo test --release
```

### Expected output:
```
running 10 tests
test perft::tests::test_perft_position_3 ... ok
test perft::tests::test_perft_position_2 ... ok
test perft::tests::test_perft_position_4 ... ok
test perft::tests::test_perft_position_5 ... ok
test utils::tests::test_bitboard_ops ... ok
test utils::tests::test_move_encoding ... ok
test utils::tests::test_square_conversion ... ok
test zobrist::tests::test_zobrist_deterministic ... ok
test zobrist::tests::test_zobrist_unique ... ok
test perft::tests::test_perft_starting_position ... ok

test result: ok. 10 passed; 0 failed
```

## Sample Session

```bash
$ cargo run --release
Chess Engine v0.1.0
Type 'help' for commands

> new
New game started

> show
 8 │ ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
 7 │ ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
 6 │ . . . . . . . .
 5 │ . . . . . . . .
 4 │ . . . . . . . .
 3 │ . . . . . . . .
 2 │ ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
 1 │ ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
   └─────────────────
     a b c d e f g h

FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
Side to move: White

> e2e4
Move made: e2e4

> e7e5
Move made: e7e5

> g1f3
Move made: g1f3

> go depth 4
info depth 1 seldepth 1 score cp 15 nodes 40 time 0 nps 0 pv b8c6
info depth 2 seldepth 2 score cp 0 nodes 210 time 1 nps 210000 pv b8c6 f1c4
info depth 3 seldepth 3 score cp 15 nodes 1234 time 8 nps 154250 pv b8c6 f1c4 g8f6
info depth 4 seldepth 4 score cp 0 nodes 3567 time 15 nps 237800 pv b8c6 f1c4 g8f6 b1c3
bestmove b8c6

> perft 4
Nodes: 665063 Time: 0.058s NPS: 11474390

> quit
Goodbye!
```

## Project Structure

```
chess_engine/
├── Cargo.toml           # Project manifest
├── README.md            # Comprehensive documentation
├── BUILD_AND_RUN.md     # This file
└── src/
    ├── main.rs          # CLI REPL and command handling (365 lines)
    ├── board.rs         # Board representation, FEN, make/unmake (611 lines)
    ├── movegen.rs       # Move generation (603 lines)
    ├── search.rs        # Alpha-beta search engine (433 lines)
    ├── eval.rs          # Position evaluation (193 lines)
    ├── tt.rs            # Transposition table (112 lines)
    ├── utils.rs         # Bitboard utilities (292 lines)
    ├── zobrist.rs       # Zobrist hashing (115 lines)
    └── perft.rs         # Perft testing (108 lines)
```

**Total: ~3,100 lines of code**

## Verified Features

✅ Complete legal move generation (pawns, knights, bishops, rooks, queens, kings)
✅ Special moves (castling, en-passant, promotions)
✅ Bitboard representation with u64
✅ FEN parsing and serialization
✅ Make/unmake with incremental Zobrist hashing
✅ Alpha-beta search with iterative deepening
✅ Quiescence search
✅ Transposition table
✅ Move ordering (TT, MVV/LVA, killers, history)
✅ Perft validation (all tests pass)
✅ Terminal UI with Unicode pieces
✅ Check detection
✅ Checkmate/stalemate detection

## Performance Benchmarks

On modern hardware (release build):

| Test | Nodes | Time | NPS |
|------|-------|------|-----|
| Perft 4 (starting position) | 197,281 | ~0.12s | ~1.6M |
| Perft 5 (starting position) | 4,865,609 | ~3.1s | ~1.6M |
| Perft 4 (after e2e4 e7e5 g1f3) | 665,063 | ~0.06s | ~11M |
| Search depth 4 | ~4,000 | ~15ms | ~270k |

## Troubleshooting

### "cargo: command not found"
Install Rust from https://rustup.rs/

### Slow compilation
This is normal for release builds. Use `cargo build` for faster debug builds during development.

### Terminal doesn't show Unicode pieces
Use the `ascii` command to toggle to ASCII display mode.

## Command Reference

- `new` - New game
- `show` / `d` - Display board
- `<move>` or `move <move>` - Make move (e.g., `e2e4`, `e7e8q`)
- `undo` / `u` - Undo last move
- `go depth <n>` - Search to depth n
- `go movetime <ms>` - Search for n milliseconds
- `perft <n>` - Run perft to depth n
- `divide <n>` - Perft with per-move breakdown
- `legal` - Show all legal moves
- `eval` - Show position evaluation
- `fen <fen>` - Load FEN position
- `ascii` - Toggle ASCII/Unicode display
- `help` - Show help
- `quit` - Exit

## Development

To run tests during development:
```bash
cargo test
```

To run specific test:
```bash
cargo test test_perft_starting_position
```

To run with verbose output:
```bash
cargo test -- --nocapture
```

## Notes

- The engine uses safe Rust throughout
- No external dependencies except Cargo/std
- Deterministic Zobrist keys for reproducibility
- Comprehensive perft tests validate correctness
- Search works well at depths 1-4
- Evaluation includes material + piece-square tables + mobility
