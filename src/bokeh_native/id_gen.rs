//! Monotonic model ID generator for Bokeh objects.

/// Generates Bokeh-style model IDs like `"p1001"`, `"p1002"`, etc.
pub struct IdGen(u32);

impl IdGen {
    /// Create a generator that will produce `"p1001"` on the first call to
    /// [`next`](Self::next).
    pub fn new() -> Self {
        IdGen(1000)
    }

    /// Increment the counter and return the next ID string (`"p1001"`,
    /// `"p1002"`, …). Each call yields a new unique value.
    pub fn next(&mut self) -> String {
        self.0 += 1;
        format!("p{}", self.0)
    }
}

impl Default for IdGen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_sequential() {
        let mut gen = IdGen::new();
        assert_eq!(gen.next(), "p1001");
        assert_eq!(gen.next(), "p1002");
        assert_eq!(gen.next(), "p1003");
    }

    #[test]
    fn ids_have_p_prefix() {
        let mut gen = IdGen::new();
        let id = gen.next();
        assert!(id.starts_with('p'));
    }
}
