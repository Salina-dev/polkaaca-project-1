//! We saw in the previous chapter that blockchain communities sometimes opt to modify the
//! consensus rules from time to time. This process is knows as a fork. Here we implement
//! a higher-order consensus engine that allows such forks to be made.
//!
//! The consensus engine we implement here does not contain the specific consensus rules to
//! be enforced before or after the fork, but rather delegates to existing consensus engines
//! for that. Here we simply write the logic for detecting whether we are before or after the fork.

use std::marker::PhantomData;

use super::{Consensus, ConsensusAuthority, Header};


/// A Higher-order consensus engine that represents a change from one set of consensus rules
/// (Before) to another set (After) at a specific block height
struct Forked<D, Before, After> {
	/// The first block height at which the new consensus rules apply
	fork_height: u64,
	phdata: PhantomData<(D, Before, After)>,
}

impl<D, B, A> Consensus for Forked<D, B, A>
where
	D: Clone + core::fmt::Debug + Eq + PartialEq + std::hash::Hash,
	B: Consensus,
	A: Consensus,
	B::Digest: Into<D>,
	A::Digest: Into<D>,
{
	type Digest = D;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		if header.height < self.fork_height {
            B::validate(&self, parent_digest.into(), header)
        } else {
            A::validate(&self,parent_digest.into(), header)
        }
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
		if partial_header.height < self.fork_height {
            B::seal(parent_digest.into(), partial_header)
        } else {
            A::seal(parent_digest.into(), partial_header)
        }
	}
}

/// Create a PoA consensus engine that changes authorities part way through the chain's history.
/// Given the initial authorities, the authorities after the fork, and the height at which the fork
/// occurs.
fn change_authorities(
	fork_height: u64,
	initial_authorities: Vec<ConsensusAuthority>,
	final_authorities: Vec<ConsensusAuthority>,
) -> impl Consensus {
	/// A PoA consensus engine that changes authorities part way through the chain's history.
    struct PoAChangeAuthorities {
        fork_height: u64,
        initial_authorities: Vec<ConsensusAuthority>,
        final_authorities: Vec<ConsensusAuthority>,
    }

    impl Consensus for PoAChangeAuthorities {
        type Digest = ConsensusAuthority;

        fn validate(&self, _: &Self::Digest, _: &Header<Self::Digest>) -> bool {
            // Add validation logic here
            true // Placeholder, change as needed
        }

        fn seal(&self, _: &Self::Digest, _: Header<()>) -> Option<Header<Self::Digest>> {
            // Add sealing logic here
            None // Placeholder, change as needed
        }
    }

    PoAChangeAuthorities {
        fork_height,
        initial_authorities,
        final_authorities,
    }
}

/// Create a PoW consensus engine that changes the difficulty part way through the chain's history.
fn change_difficulty(
	fork_height: u64,
	initial_difficulty: u64,
	final_difficulty: u64,
) -> impl Consensus {
	// Define and return the ChangingDifficultyPoW instance directly within the function
    struct ChangingDifficultyPoW<C: Consensus> {
        inner: C,
        fork_height: u64,
        initial_difficulty: u64,
        final_difficulty: u64,
    }

    impl<C: Consensus> Consensus for ChangingDifficultyPoW<C> {
        type Digest = C::Digest;

        fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
            self.inner.validate(parent_digest, header)
        }

        fn seal(
            &self,
            parent_digest: &Self::Digest,
            partial_header: Header<()>,
        ) -> Option<Header<Self::Digest>> {
            let sealed_header = self.inner.seal(parent_digest, partial_header)?;
            // Check if the fork height has been reached
            if sealed_header.height >= self.fork_height {
                // Adjust the difficulty based on the fork height
                // In this example, we simply use the final difficulty
                // You can implement more sophisticated logic here
                // For simplicity, we assume that the difficulty change is immediate
                let new_difficulty = if sealed_header.height == self.fork_height {
                    self.final_difficulty
                } else {
                    sealed_header.difficulty // Retain the current difficulty until the fork height
                };
                // Update the difficulty in the sealed header
                let modified_header = Header {
                    difficulty: new_difficulty,
                    ..sealed_header
                };
                Some(modified_header)
            } else {
                Some(sealed_header)
            }
        }
    }

    // Return an instance of ChangingDifficultyPoW with the specified parameters
    ChangingDifficultyPoW {
        inner: (),
        fork_height,
        initial_difficulty,
        final_difficulty,
    }
}

/// Earlier in this chapter we implemented a consensus rule in which blocks are only considered
/// valid if they contain an even state root. Sometimes a chain will be launched with a more
/// traditional consensus like PoW or PoA and only introduce an additional requirement like even
/// state root after a particular height.
///
/// Create a consensus engine that introduces the even-only logic only after the given fork height.
/// Other than the evenness requirement, the consensus rules should not change at the fork. This
/// function should work with either PoW, PoA, or anything else as the underlying consensus engine.
fn even_after_given_height<Original: Consensus>(fork_height: u64) -> impl Consensus {
	// Define a new consensus engine that wraps the original consensus engine
    struct EvenAfterGivenHeight<Inner: Consensus> {
        inner: Inner,
        fork_height: u64,
    }

    impl<Inner: Consensus> Consensus for EvenAfterGivenHeight<Inner> {
        type Digest = Inner::Digest;

        fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
            // Check if the fork height has been reached
            if header.height >= self.fork_height {
                // If after the fork height, also validate the evenness of the state root
                header.state_root % 2 == 0 && self.inner.validate(parent_digest, header)
            } else {
                // If before the fork height, only validate using the original consensus rules
                self.inner.validate(parent_digest, header)
            }
        }

        fn seal(
            &self,
            parent_digest: &Self::Digest,
            partial_header: Header<()>,
        ) -> Option<Header<Self::Digest>> {
            // Delegate sealing to the inner consensus engine
            self.inner.seal(parent_digest, partial_header)
        }
    }

    // Return an instance of EvenAfterGivenHeight with the specified parameters
    EvenAfterGivenHeight {
        inner: Original::default(), // Instantiate the original consensus engine
        fork_height,
    }
}

/// So far we have considered the simpler case where the consensus engines before and after the fork
/// use the same Digest type. Let us now turn our attention to the more general case where even the
/// digest type changes.
///
/// In order to implement a consensus change where even the Digest type changes, we will need an
/// enum that wraps the two individual digest types
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum PowOrPoaDigest {
	Pow(u64),
	Poa(ConsensusAuthority),
}

impl From<u64> for PowOrPoaDigest {
	fn from(d: u64) -> Self {
		PowOrPoaDigest::Pow(d)
	}
}

impl From<ConsensusAuthority> for PowOrPoaDigest {
	fn from(d: ConsensusAuthority) -> Self {
		PowOrPoaDigest::Poa(d)
	}
}

/// In the spirit of Ethereum's recent switch from PoW to PoA, let us model a similar
/// switch in our consensus framework. It should go without saying that the real-world ethereum
/// handoff was considerably more complex than it may appear in our simplified example, although
/// the fundamentals are the same.
fn pow_to_poa(
	fork_height: u64,
	difficulty: u64,
	authorities: Vec<ConsensusAuthority>,
) -> impl Consensus {
	todo!("Exercise 6")
}
