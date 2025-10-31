//! Utility functions for bitboard manipulation, move encoding, and coordinate conversion.

use std::time::Instant;

/// A chess move encoded in 16 bits:
/// - bits 0-5: from square (0-63)
/// - bits 6-11: to square (0-63)
/// - bits 12-13: promotion piece (0=none, 1=knight, 2=bishop, 3=rook, 4=queen)
/// - bits 14-15: flags (0=normal, 1=castle, 2=en-passant, 3=promotion)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move(pub u16);

impl Move {
    #[inline]
    pub const fn new(from: u8, to: u8) -> Self {
        Move(((to as u16) << 6) | (from as u16))
    }

    #[inline]
    pub const fn with_flags(from: u8, to: u8, flags: u16) -> Self {
        Move(((flags & 3) << 14) | ((to as u16) << 6) | (from as u16))
    }

    #[inline]
    pub const fn with_promotion(from: u8, to: u8, promotion: u8) -> Self {
        Move(
            (3u16 << 14) | // promotion flag
            (((promotion - 1) as u16) << 12) |
            ((to as u16) << 6) |
            (from as u16),
        )
    }

    #[inline]
    pub const fn from(self) -> u8 {
        (self.0 & 0x3F) as u8
    }

    #[inline]
    pub const fn to(self) -> u8 {
        ((self.0 >> 6) & 0x3F) as u8
    }

    #[inline]
    pub const fn flags(self) -> u8 {
        ((self.0 >> 14) & 3) as u8
    }

    #[inline]
    pub const fn promotion(self) -> u8 {
        (((self.0 >> 12) & 3) + 1) as u8
    }

    #[inline]
    pub const fn is_castle(self) -> bool {
        self.flags() == 1
    }

    #[inline]
    pub const fn is_en_passant(self) -> bool {
        self.flags() == 2
    }

    #[inline]
    pub const fn is_promotion(self) -> bool {
        self.flags() == 3
    }

    /// Convert move to algebraic notation (e.g., "e2e4", "e7e8q")
    pub fn to_string(self) -> String {
        let from_sq = square_name(self.from());
        let to_sq = square_name(self.to());
        if self.is_promotion() {
            let promo = match self.promotion() {
                1 => 'n',
                2 => 'b',
                3 => 'r',
                4 => 'q',
                _ => '?',
            };
            format!("{}{}{}", from_sq, to_sq, promo)
        } else {
            format!("{}{}", from_sq, to_sq)
        }
    }

    /// Parse algebraic notation (e.g., "e2e4", "e7e8q")
    pub fn from_string(s: &str) -> Option<Self> {
        if s.len() < 4 {
            return None;
        }
        let from = parse_square(&s[0..2])?;
        let to = parse_square(&s[2..4])?;
        
        if s.len() == 5 {
            let promo = match s.chars().nth(4)? {
                'n' | 'N' => 1,
                'b' | 'B' => 2,
                'r' | 'R' => 3,
                'q' | 'Q' => 4,
                _ => return None,
            };
            Some(Move::with_promotion(from, to, promo))
        } else {
            Some(Move::new(from, to))
        }
    }
}

/// Convert square index (0-63) to algebraic notation (e.g., 0 -> "a1", 63 -> "h8")
#[inline]
pub fn square_name(sq: u8) -> String {
    let file = (sq & 7) as u8;
    let rank = (sq >> 3) as u8;
    format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
}

/// Parse algebraic notation to square index (e.g., "e4" -> 28)
#[inline]
pub fn parse_square(s: &str) -> Option<u8> {
    if s.len() != 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let file = bytes[0].to_ascii_lowercase().wrapping_sub(b'a');
    let rank = bytes[1].wrapping_sub(b'1');
    if file < 8 && rank < 8 {
        Some(rank * 8 + file)
    } else {
        None
    }
}

/// Convert file index (0-7) to algebraic notation (e.g., 0 -> 'a', 7 -> 'h')
#[inline]
pub const fn file_char(file: u8) -> char {
    (b'a' + file) as char
}

/// Convert rank index (0-7) to algebraic notation (e.g., 0 -> '1', 7 -> '8')
#[inline]
pub const fn rank_char(rank: u8) -> char {
    (b'1' + rank) as char
}

/// Count the number of set bits in a u64 (population count)
#[inline]
pub const fn popcount(bb: u64) -> u32 {
    bb.count_ones()
}

/// Get the index of the least significant bit (0-63)
/// Returns 64 if bb is 0
#[inline]
pub const fn lsb(bb: u64) -> u8 {
    if bb == 0 {
        64
    } else {
        bb.trailing_zeros() as u8
    }
}

/// Get the index of the most significant bit (0-63)
/// Returns 64 if bb is 0
#[inline]
pub const fn msb(bb: u64) -> u8 {
    if bb == 0 {
        64
    } else {
        63 - bb.leading_zeros() as u8
    }
}

/// Pop the least significant bit and return its index
#[inline]
pub fn pop_lsb(bb: &mut u64) -> u8 {
    let sq = lsb(*bb);
    *bb &= *bb - 1;
    sq
}

/// Get the bit at a specific square
#[inline]
pub const fn bit_at(sq: u8) -> u64 {
    1u64 << sq
}

/// Check if a bit is set at a specific square
#[inline]
pub const fn is_set(bb: u64, sq: u8) -> bool {
    (bb & bit_at(sq)) != 0
}

/// Set a bit at a specific square
#[inline]
pub const fn set_bit(bb: u64, sq: u8) -> u64 {
    bb | bit_at(sq)
}

/// Clear a bit at a specific square
#[inline]
pub const fn clear_bit(bb: u64, sq: u8) -> u64 {
    bb & !bit_at(sq)
}

/// Get file (0-7) from square index
#[inline]
pub const fn file_of(sq: u8) -> u8 {
    sq & 7
}

/// Get rank (0-7) from square index
#[inline]
pub const fn rank_of(sq: u8) -> u8 {
    sq >> 3
}

/// Create a square index from file and rank
#[inline]
pub const fn square(rank: u8, file: u8) -> u8 {
    rank * 8 + file
}

/// Distance between two squares (Chebyshev distance)
#[inline]
pub fn distance(sq1: u8, sq2: u8) -> u8 {
    let file_dist = (file_of(sq1) as i8 - file_of(sq2) as i8).abs();
    let rank_dist = (rank_of(sq1) as i8 - rank_of(sq2) as i8).abs();
    file_dist.max(rank_dist) as u8
}

/// Timer for measuring search time
pub struct Timer {
    start: Instant,
}

impl Timer {
    #[inline]
    pub fn new() -> Self {
        Timer { start: Instant::now() }
    }

    #[inline]
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    #[inline]
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_conversion() {
        assert_eq!(parse_square("a1"), Some(0));
        assert_eq!(parse_square("h8"), Some(63));
        assert_eq!(parse_square("e4"), Some(28));
        assert_eq!(square_name(0), "a1");
        assert_eq!(square_name(63), "h8");
        assert_eq!(square_name(28), "e4");
    }

    #[test]
    fn test_move_encoding() {
        let m = Move::new(12, 28);
        assert_eq!(m.from(), 12);
        assert_eq!(m.to(), 28);
        
        let m2 = Move::with_promotion(52, 60, 4);
        assert!(m2.is_promotion());
        assert_eq!(m2.promotion(), 4);
    }

    #[test]
    fn test_bitboard_ops() {
        assert_eq!(popcount(0xFF), 8);
        assert_eq!(lsb(0x8), 3);
        assert_eq!(msb(0x8), 3);
        assert_eq!(msb(0x80), 7);
    }
}
