//! Zobrist hashing for position keys.
//! Uses a deterministic PRNG (LCG) for reproducible hash keys.

/// Zobrist key table for hashing positions
pub struct Zobrist {
    /// Keys for each piece type and square [piece][square]
    /// piece index: 0-5 = white P,N,B,R,Q,K; 6-11 = black P,N,B,R,Q,K
    pub pieces: [[u64; 64]; 12],
    /// Key for side to move (black)
    pub side: u64,
    /// Keys for castling rights [4]
    pub castling: [u64; 4],
    /// Keys for en-passant file [8]
    pub en_passant: [u64; 8],
}

/// Simple LCG for deterministic random number generation
struct Lcg {
    state: u64,
}

impl Lcg {
    const fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }
}

impl Zobrist {
    /// Initialize Zobrist keys with a fixed seed for reproducibility
    pub fn new() -> Self {
        let mut rng = Lcg::new(0x123456789ABCDEF0);
        
        let mut pieces = [[0u64; 64]; 12];
        for piece in 0..12 {
            for sq in 0..64 {
                pieces[piece][sq] = rng.next();
            }
        }

        let side = rng.next();

        let mut castling = [0u64; 4];
        for i in 0..4 {
            castling[i] = rng.next();
        }

        let mut en_passant = [0u64; 8];
        for i in 0..8 {
            en_passant[i] = rng.next();
        }

        Zobrist {
            pieces,
            side,
            castling,
            en_passant,
        }
    }

    /// Get piece index for Zobrist table
    /// piece: 0=pawn, 1=knight, 2=bishop, 3=rook, 4=queen, 5=king
    /// color: 0=white, 1=black
    #[inline]
    pub const fn piece_index(piece: u8, color: u8) -> usize {
        (color as usize * 6) + piece as usize
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

// Global Zobrist instance (lazy_static alternative using const/static)
static ZOBRIST_INIT: std::sync::Once = std::sync::Once::new();
static mut ZOBRIST_KEYS: Option<Zobrist> = None;

/// Get the global Zobrist instance
pub fn zobrist() -> &'static Zobrist {
    unsafe {
        ZOBRIST_INIT.call_once(|| {
            ZOBRIST_KEYS = Some(Zobrist::new());
        });
        ZOBRIST_KEYS.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zobrist_deterministic() {
        let z1 = Zobrist::new();
        let z2 = Zobrist::new();
        assert_eq!(z1.side, z2.side);
        assert_eq!(z1.pieces[0][0], z2.pieces[0][0]);
        assert_eq!(z1.castling[0], z2.castling[0]);
    }

    #[test]
    fn test_zobrist_unique() {
        let z = Zobrist::new();
        // Check that keys are different (high probability)
        assert_ne!(z.pieces[0][0], z.pieces[0][1]);
        assert_ne!(z.side, z.castling[0]);
    }
}
