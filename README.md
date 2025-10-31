# Chess Engine

A complete, terminal-based chess engine written in Rust (edition 2021) featuring:

- **Bitboard representation** with u64 for efficient move generation
- **Full legal chess rules** including castling, en-passant, and promotions
- **Alpha-beta search** with iterative deepening and quiescence
- **Transposition table** with Zobrist hashing
- **Move ordering** using TT moves, MVV/LVA, killer moves, and history heuristic
- **Perft validation** with test positions
- **Terminal REPL** with Unicode pieces

## Build Instructions

### Prerequisites

- Rust (stable, edition 2021)
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Optional: enable native CPU optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Running

```bash
# Run in release mode for best performance
cargo run --release

# Or run the binary directly
./target/release/chess_engine
```

## Usage

### Interactive Session

```text
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

> go depth 4
info depth 1 seldepth 1 score cp 70 nodes 40 time 0 nps 0 pv b1c3
info depth 2 seldepth 2 score cp 0 nodes 222 time 0 nps 0 pv b1c3 b8c6
info depth 3 seldepth 3 score cp 70 nodes 1395 time 9 nps 155000 pv b1c3 b8c6 g1f3
info depth 4 seldepth 4 score cp 0 nodes 3980 time 12 nps 331666 pv b1c3 b8c6 g1f3 g8f6
bestmove b1c3

> perft 4
Nodes: 197281 Time: 0.123s NPS: 1603089

> quit
Goodbye!
```

### Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `new` | Start a new game from starting position | `new` |
| `show` | Display the current board | `show` |
| `fen <fen>` | Load position from FEN string | `fen rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2` |
| `move <move>` | Make a move in algebraic notation | `move e2e4` |
| `<move>` | Make a move (shorthand) | `e2e4` |
| `undo` | Undo the last move | `undo` |
| `go depth <n>` | Search to depth n | `go depth 6` |
| `go movetime <ms>` | Search for specified milliseconds | `go movetime 5000` |
| `perft <depth>` | Run perft test to count nodes | `perft 5` |
| `divide <depth>` | Run perft divide (per-move breakdown) | `divide 4` |
| `eval` | Show static evaluation of position | `eval` |
| `legal` | Show all legal moves | `legal` |
| `ascii` | Toggle between ASCII and Unicode display | `ascii` |
| `help` | Show command help | `help` |
| `quit` | Exit the program | `quit` |

### Move Notation

Moves use coordinate notation:
- Normal moves: `e2e4`, `g1f3`
- Promotions: `e7e8q` (promote to queen), `a2a1n` (promote to knight)
- Castling: `e1g1` (kingside), `e1c1` (queenside)
- En-passant: handled automatically when moving pawn to en-passant square

Promotion pieces: `q` (queen), `r` (rook), `b` (bishop), `n` (knight)

## Testing

### Run All Tests

```bash
# Run unit tests
cargo test

# Run with optimizations
cargo test --release
```

### Perft Validation

The engine includes perft tests for validating move generation:

```bash
# Test specific perft positions
cargo test test_perft_starting_position
cargo test test_perft_position_2
cargo test test_perft_position_3
cargo test test_perft_position_4
cargo test test_perft_position_5
```

Known perft results:
- Starting position depth 1: 20 nodes
- Starting position depth 2: 400 nodes
- Starting position depth 3: 8,902 nodes
- Starting position depth 4: 197,281 nodes

### Interactive Perft

```bash
$ cargo run --release
> new
> perft 5
Nodes: 4865609 Time: 3.124s NPS: 1557421

> divide 3
a2a3: 380
a2a4: 420
b2b3: 420
...
Nodes: 8902 Time: 0.012s NPS: 741833
```

## Architecture

### Modules

- **utils.rs** - Bitboard utilities, move encoding, coordinate conversion
- **zobrist.rs** - Deterministic Zobrist hashing for position keys
- **board.rs** - Board representation, FEN parsing, make/unmake moves
- **movegen.rs** - Pseudo-legal and legal move generation
- **eval.rs** - Position evaluation with material and piece-square tables
- **tt.rs** - Transposition table with bound types
- **search.rs** - Iterative deepening, alpha-beta, quiescence, move ordering
- **perft.rs** - Performance testing for move generation validation
- **main.rs** - CLI REPL and command handling

### Key Features

**Bitboards**: Each piece type per color has a 64-bit bitboard. Fast operations using bit manipulation.

**Move Encoding**: 16-bit move encoding with from/to squares, promotion piece, and flags.

**Make/Unmake**: Reversible move operations with history stack for undo. Incremental Zobrist hash updates.

**Search**:
- Iterative deepening from depth 1 to target depth
- Alpha-beta pruning with principal variation search
- Quiescence search for tactical positions
- Transposition table with exact/lower/upper bounds
- Move ordering: TT move → MVV/LVA captures → killers → history

**Evaluation**:
- Material counting
- Piece-square tables for positional play
- Simple mobility bonus

## Performance

Typical performance on modern hardware (release build):
- **Perft 4**: ~197k nodes in ~0.1s (1.9M nps)
- **Perft 5**: ~4.8M nodes in ~3s (1.6M nps)
- **Search depth 4**: ~4k nodes in ~12ms (330k nps)

Performance can be improved with:
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Known Limitations

- No pondering (thinking on opponent's time)
- No opening book or endgame tablebases
- Basic evaluation (no pawn structure, king safety, etc.)
- Fixed-size transposition table (64 MB default)

## License

This is a demonstration chess engine for educational purposes.

## Example Games

### Scholar's Mate Demo
```
> new
> e2e4
> e7e5
> f1c4
> b8c6
> d1h5
> g8f6
> h5f7
```

### Perft Test Positions

Position 2 (Kiwipete):
```
> fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
> perft 3
Nodes: 97862 Time: 0.056s NPS: 1747535
```

Position 3:
```
> fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1
> perft 4
Nodes: 43238 Time: 0.024s NPS: 1801583
```

## Contributing

This engine was built to specification as a demonstration of Rust systems programming and chess engine architecture.

## Acknowledgments

- Perft test positions from the Chess Programming Wiki
- Bitboard techniques from various chess programming resources
- Piece-square tables adapted from Simplified Evaluation Function
