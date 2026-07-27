//! Fixed-capacity ring buffer for streaming token IDs.
//!
//! Designed for the hot path of token-level verification: after [`TokenRingBuffer::new`],
//! [`TokenRingBuffer::push`] never allocates. [`TokenRingBuffer::last_n`] returns an
//! iterator over the most recent tokens without building a `Vec`.

/// Default capacity used by the guardrails streaming path (tokens).
pub const DEFAULT_CAPACITY: usize = 4096;

/// Zero-growth ring buffer for streaming tokens.
///
/// Capacity is fixed at construction. On overflow the oldest entries are overwritten.
/// The type is `Send + Sync` so it can live behind a mutex or per-session slot across threads.
#[derive(Debug, Clone)]
pub struct TokenRingBuffer {
    tokens: Box<[u32]>,
    positions: Box<[usize]>,
    /// Next write index.
    head: usize,
    /// Number of valid entries (`<= capacity`).
    len: usize,
    capacity: usize,
}

impl TokenRingBuffer {
    /// Create a buffer with the given capacity.
    ///
    /// Allocates once. Subsequent [`Self::push`] calls do not allocate.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "TokenRingBuffer capacity must be > 0");
        Self {
            tokens: vec![0; capacity].into_boxed_slice(),
            positions: vec![0; capacity].into_boxed_slice(),
            head: 0,
            len: 0,
            capacity,
        }
    }

    /// Create a buffer with [`DEFAULT_CAPACITY`] (4096).
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    /// Append a token. `O(1)`. Overwrites the oldest entry when full.
    pub fn push(&mut self, token_id: u32, position: usize) {
        let idx = self.head;
        self.tokens[idx] = token_id;
        self.positions[idx] = position;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Iterate the last `n` tokens in chronological order (oldest of the window first).
    ///
    /// If `n > len`, returns all stored tokens. Does not allocate.
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = (u32, usize)> + '_ {
        let n = n.min(self.len);
        (0..n).map(move |i| {
            let idx = self.index_of_last_n(n, i);
            (self.tokens[idx], self.positions[idx])
        })
    }

    /// Number of stored tokens.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Fixed capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Drop all entries without releasing the backing storage.
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    /// Physical index of the `i`-th element within the last-`n` window (`i` in `0..n`).
    fn index_of_last_n(&self, n: usize, i: usize) -> usize {
        // Oldest of the last n sits at: head - n + i (mod capacity).
        (self.head + self.capacity - n + i) % self.capacity
    }
}

// Explicit trait assertions for interview / API docs: safe to share across threads.
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn check() {
        assert_send_sync::<TokenRingBuffer>();
    }
    let _ = check;
};

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn push_and_last_n_returns_most_recent() {
        let mut buf = TokenRingBuffer::new(16);
        for i in 0..10 {
            buf.push(i as u32, i * 10);
        }
        let got: Vec<_> = buf.last_n(3).collect();
        assert_eq!(got, vec![(7, 70), (8, 80), (9, 90)]);
        assert_eq!(buf.len(), 10);
    }

    #[test]
    fn overflow_overwrites_oldest() {
        let mut buf = TokenRingBuffer::new(4);
        for i in 0..14 {
            buf.push(i as u32, i);
        }
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.capacity(), 4);
        let got: Vec<_> = buf.last_n(4).collect();
        assert_eq!(got, vec![(10, 10), (11, 11), (12, 12), (13, 13)]);
    }

    #[test]
    fn last_n_larger_than_len_returns_all() {
        let mut buf = TokenRingBuffer::new(8);
        buf.push(1, 0);
        buf.push(2, 1);
        let got: Vec<_> = buf.last_n(100).collect();
        assert_eq!(got, vec![(1, 0), (2, 1)]);
    }

    #[test]
    fn clear_resets_len_keeps_capacity() {
        let mut buf = TokenRingBuffer::new(8);
        buf.push(1, 0);
        buf.push(2, 1);
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 8);
        assert!(buf.last_n(3).next().is_none());
        buf.push(9, 99);
        assert_eq!(buf.last_n(1).collect::<Vec<_>>(), vec![(9, 99)]);
    }

    #[test]
    fn push_does_not_grow_backing_storage() {
        let mut buf = TokenRingBuffer::new(64);
        let token_ptr = buf.tokens.as_ptr();
        let pos_ptr = buf.positions.as_ptr();
        let cap = buf.capacity();
        for i in 0..(cap * 3) {
            buf.push(i as u32, i);
        }
        assert_eq!(buf.tokens.as_ptr(), token_ptr);
        assert_eq!(buf.positions.as_ptr(), pos_ptr);
        assert_eq!(buf.tokens.len(), cap);
        assert_eq!(buf.positions.len(), cap);
        assert_eq!(buf.len(), cap);
    }

    #[test]
    #[should_panic(expected = "capacity must be > 0")]
    fn zero_capacity_panics() {
        let _ = TokenRingBuffer::new(0);
    }

    proptest! {
        #[test]
        fn prop_len_never_exceeds_capacity(
            capacity in 1usize..128,
            ops in proptest::collection::vec(any::<(u32, usize)>(), 0..300),
        ) {
            let mut buf = TokenRingBuffer::new(capacity);
            for (tok, pos) in ops {
                buf.push(tok, pos);
                prop_assert!(buf.len() <= capacity);
            }
        }

        #[test]
        fn prop_last_n_matches_tail_of_logical_log(
            capacity in 1usize..64,
            values in proptest::collection::vec(any::<u32>(), 1..200),
            n in 0usize..80,
        ) {
            let mut buf = TokenRingBuffer::new(capacity);
            let mut logical = Vec::new();
            for (i, tok) in values.into_iter().enumerate() {
                buf.push(tok, i);
                logical.push((tok, i));
                if logical.len() > capacity {
                    logical.remove(0);
                }
            }
            let expected: Vec<_> = if n >= logical.len() {
                logical.clone()
            } else {
                logical[logical.len() - n..].to_vec()
            };
            let got: Vec<_> = buf.last_n(n).collect();
            prop_assert_eq!(got, expected);
        }
    }
}
