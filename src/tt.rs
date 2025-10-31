//! Transposition table for caching search results.

use crate::utils::Move;

/// Entry bound types
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

/// Transposition table entry
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub best_move: Option<Move>,
    pub bound: Bound,
}

impl TTEntry {
    pub const fn empty() -> Self {
        TTEntry {
            hash: 0,
            depth: 0,
            score: 0,
            best_move: None,
            bound: Bound::Exact,
        }
    }
}

/// Transposition table
pub struct TranspositionTable {
    table: Vec<TTEntry>,
    size: usize,
}

impl TranspositionTable {
    /// Create a new transposition table with given size in MB
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        // Round to power of 2 for efficient modulo
        let size = num_entries.next_power_of_two();
        
        TranspositionTable {
            table: vec![TTEntry::empty(); size],
            size,
        }
    }

    /// Get index for a hash
    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.size - 1)
    }

    /// Probe the transposition table
    #[inline]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.table[self.index(hash)];
        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    /// Store an entry in the transposition table
    #[inline]
    pub fn store(&mut self, hash: u64, depth: u8, score: i32, best_move: Option<Move>, bound: Bound) {
        let idx = self.index(hash);
        let entry = &mut self.table[idx];
        
        // Always replace scheme (can be improved with depth-preferred replacement)
        if entry.hash != hash || depth >= entry.depth {
            *entry = TTEntry {
                hash,
                depth,
                score,
                best_move,
                bound,
            };
        }
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        for entry in &mut self.table {
            *entry = TTEntry::empty();
        }
    }

    /// Get the number of used entries (for statistics)
    pub fn used_entries(&self) -> usize {
        self.table.iter().filter(|e| e.hash != 0).count()
    }

    /// Get fill percentage
    pub fn fill_percentage(&self) -> f64 {
        (self.used_entries() as f64 / self.size as f64) * 100.0
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(64) // 64 MB default
    }
}
