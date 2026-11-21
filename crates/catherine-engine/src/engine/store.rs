//! Purpose: the fixed-capacity table type behind the lane flavor's
//! symbolic-path structures (openspec change `add-kani-verification-lane`,
//! beads `catherine-6g2`) — dense `Vec`-shaped tables whose backing store
//! is a `[Option<T>; CAP]` array under `cfg(kani)` and a `Vec<Option<T>>`
//! everywhere else, so the verification formula never charges allocation
//! modeling, symbolic-length heap writes, or growth branches for the
//! tables the build walk and the families traverse.
//!
//! Responsibilities: one call surface (`new`/`with_len`/`len`/`push`/
//! `get`/`iter`/`IndexMut`/`from_option_vec`) with identical semantics on
//! both flavors — `iter` yields `&Option<T>` (cells within the length may
//! be `None`), `get` returns `Option<&Option<T>>` (`None` past the
//! length), `push` appends. Under `cfg(kani)` the capacity asserts are
//! proof obligations (`with_len(n)` requires `n <= CAP`, `push` requires
//! `len < CAP`, `IndexMut` requires `i < CAP` via array bounds), never a
//! reachable failure for the declared shapes; the non-kani flavor ignores
//! `CAP` entirely — production documents are parsed-data-sized and the
//! parsed flavor compiles the same walkers.
//!
//! Rationale: measured on this lane (ledger `0004-c7563980`), the build
//! harness's propositional reduction is OOM-killed above 24 GB — the
//! formula mass is the bit-blasted allocation model over heap `Vec`s
//! (symbolic-size `vec![None; n_src]` fills, `with_capacity` on symbolic
//! bounds, raw-pointer bounded pushes). Concrete-index arrays with
//! asserted caps encode to a CNF that fits the lane standard, on the
//! probe-matrix precedent (dense tables over symbolic scalars at concrete
//! bounds prove in seconds; lane construction verifies ~26 s).

/// Row-space cap for one arrow's partial morphism table (parallel to its
/// source rows). The declared shapes top out at 2 rows per object
/// (`shapes::OBJECT_ROWS`); the cap carries headroom and is a proof
/// obligation under `cfg(kani)`.
pub const ROWS_CAP: usize = 8;

/// Cap for one build's report vectors (typing violations, dangling
/// cells): every taken cell lands in at most one report, and the
/// declared shapes hold at most 7 arrows with 2 source rows each.
/// Proof obligation under `cfg(kani)`.
pub const REPORT_CAP: usize = 32;

#[cfg(kani)]
mod flavor {
    /// The array-backed lane flavor: a fixed-capacity partial table.
    #[derive(Clone, Debug)]
    pub struct Bounded<T, const CAP: usize> {
        slots: [Option<T>; CAP],
        len: usize,
    }

    impl<T, const CAP: usize> Bounded<T, CAP> {
        pub fn new() -> Self {
            Self {
                slots: [const { None }; CAP],
                len: 0,
            }
        }

        /// A table of `n` unmapped rows (the `vec![None; n]` shape of
        /// the build walk). `n <= CAP` is a proof obligation.
        pub fn with_len(n: usize) -> Self {
            assert!(n <= CAP);
            Self {
                slots: [const { None }; CAP],
                len: n,
            }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        /// Asserted append — the proof obligation replaces the
        /// allocation model.
        pub fn push(&mut self, value: T) {
            assert!(self.len < CAP);
            self.slots[self.len] = Some(value);
            self.len += 1;
        }

        /// Asserted append of a partial cell (an unmapped row is a
        /// `None` entry inside the table, distinct from a short
        /// length). Same obligation as `push`.
        pub fn push_cell(&mut self, cell: Option<T>) {
            assert!(self.len < CAP);
            self.slots[self.len] = cell;
            self.len += 1;
        }

        /// The cell at `i`: `None` past the length (cells beyond a
        /// source object's row space are ignored, never a failure).
        pub fn get(&self, i: usize) -> Option<&Option<T>> {
            if i < self.len {
                Some(&self.slots[i])
            } else {
                None
            }
        }

        /// The owned cell vector, for the parsed surface's materialized
        /// tables (`TypedInstance::build`).
        pub fn to_option_vec(&self) -> Vec<Option<T>>
        where
            T: Clone,
        {
            self.slots.iter().take(self.len).cloned().collect()
        }

        pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
            self.slots.iter().take(self.len)
        }

        /// Adopt a parsed cell vector. Dead on the lane flavor (the
        /// lane never builds through the parsed surface) but compiled
        /// so both flavors serve the same walkers; `v.len() <= CAP` is
        /// a proof obligation.
        pub fn from_option_vec(v: Vec<Option<T>>) -> Self {
            let len = v.len();
            assert!(len <= CAP);
            let mut this = Self::with_len(len);
            for (i, cell) in v.into_iter().enumerate() {
                this.slots[i] = cell;
            }
            this
        }
    }

    impl<T, const CAP: usize> Default for Bounded<T, CAP> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T, const CAP: usize> std::ops::Index<usize> for Bounded<T, CAP> {
        type Output = Option<T>;

        fn index(&self, i: usize) -> &Self::Output {
            &self.slots[i]
        }
    }

    impl<T, const CAP: usize> std::ops::IndexMut<usize> for Bounded<T, CAP> {
        fn index_mut(&mut self, i: usize) -> &mut Self::Output {
            &mut self.slots[i]
        }
    }
}

#[cfg(not(kani))]
mod flavor {
    /// The production flavor: a plain owned cell vector. `CAP` is
    /// carried in the type so both flavors share one call surface and
    /// one walker set; it is a verification-only bound and is ignored
    /// here — parsed documents are arbitrary-sized.
    #[derive(Clone, Debug)]
    pub struct Bounded<T, const CAP: usize>(Vec<Option<T>>);

    impl<T, const CAP: usize> Bounded<T, CAP> {
        pub fn new() -> Self {
            Self(Vec::new())
        }

        /// The `vec![None; n]` shape of the build walk, owned storage.
        pub fn with_len(n: usize) -> Self {
            Self((0..n).map(|_| None).collect())
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn push(&mut self, value: T) {
            self.0.push(Some(value));
        }

        /// Asserted append of a partial cell (an unmapped row is a
        /// `None` entry inside the table, distinct from a short
        /// length).
        pub fn push_cell(&mut self, cell: Option<T>) {
            self.0.push(cell);
        }

        pub fn get(&self, i: usize) -> Option<&Option<T>> {
            self.0.get(i)
        }

        /// The owned cell vector, for the parsed surface's materialized
        /// tables (`TypedInstance::build`).
        pub fn to_option_vec(&self) -> Vec<Option<T>>
        where
            T: Clone,
        {
            self.0.clone()
        }

        pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
            self.0.iter()
        }

        pub fn from_option_vec(v: Vec<Option<T>>) -> Self {
            Self(v)
        }
    }

    impl<T, const CAP: usize> Default for Bounded<T, CAP> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T, const CAP: usize> std::ops::Index<usize> for Bounded<T, CAP> {
        type Output = Option<T>;

        fn index(&self, i: usize) -> &Self::Output {
            &self.0[i]
        }
    }

    impl<T, const CAP: usize> std::ops::IndexMut<usize> for Bounded<T, CAP> {
        fn index_mut(&mut self, i: usize) -> &mut Self::Output {
            &mut self.0[i]
        }
    }
}

pub use flavor::Bounded;
