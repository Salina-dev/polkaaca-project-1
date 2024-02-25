//! PoW and PoA each have their own set of strengths and weaknesses. Many chains are happy to choose
//! one of them. But other chains would like consensus properties that fall in between. To achieve
//! this we could consider interleaving PoW blocks with PoA blocks. Some very early designs of
//! Ethereum considered this approach as a way to transition away from PoW.

/// A Consensus engine that alternates back and forth between PoW and PoA sealed blocks.
use super::{Consensus, Header};

/// A Consensus engine that alternates back and forth between PoW and PoA sealed blocks.
struct AlternatingPowPoa {
    current_engine: Box<dyn Consensus<Digest = u64>>,
}

impl AlternatingPowPoa {
    /// Create a new `AlternatingPowPoa` instance, starting with the given consensus engine.
    fn new(initial_engine: Box<dyn Consensus<Digest = u64>>) -> Self {
        Self {
            current_engine: initial_engine,
        }
    }

    /// Toggle between PoW and PoA consensus engines.
    fn toggle_engine(&mut self) {
        // If the current engine is PoW, switch to PoA, and vice versa
        if self.current_engine.human_name() == "Proof of Work" {
            self.current_engine = Box::new(PoaConsensus);
        } else {
            self.current_engine = Box::new(PowConsensus);
        }
    }
}

impl Consensus for AlternatingPowPoa {
    type Digest = u64;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        self.current_engine.validate(parent_digest, header)
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        self.current_engine.seal(parent_digest, partial_header)
    }
}
