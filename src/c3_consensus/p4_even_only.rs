//! In the previous chapter, we considered a hypothetical scenario where blocks must contain an even
//! state root in order to be valid. Now we will express that logic here as a higher-order consensus
//! engine. It is higher- order because it will wrap an inner consensus engine, such as PoW or PoA
//! and work in either case.

use std::marker::PhantomData;

use super::{Consensus, Header};

/// A Consensus engine that wraps another consensus engine. This engine enforces the requirement
/// that a block must have an even state root in order to be valid

/// A Consensus engine that requires the state root to be even for the header to be valid.
/// Wraps an inner consensus engine whose rules will also be enforced.
struct EvenOnly<Inner: Consensus>(PhantomData<Inner>);

impl<Inner: Consensus> Consensus for EvenOnly<Inner> {
	type Digest = Inner::Digest;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		
		// Delegate validation to the inner consensus engine
        let inner_consensus = Inner::new(); // Instantiate inner consensus engine
        let is_inner_valid = inner_consensus.validate(parent_digest, header);

        // Check if the state root is even
        let is_state_root_even = header.state_root % 2 == 0;

        // The header is valid if the inner consensus engine deems it valid and the state root is even
        is_inner_valid && is_state_root_even
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
		
       // Delegate sealing to the inner consensus engine
	   let inner_consensus = Inner::new(); // Instantiate inner consensus engine
	   let sealed_header = inner_consensus.seal(parent_digest, partial_header)?;

	   // Check if the state root is even
	   if sealed_header.state_root % 2 == 0 {
		   Some(sealed_header) // Return the sealed header if the state root is even
	   } else {
		   None // Return None if the state root is not even
	   }
	}
}

/// Using the moderate difficulty PoW algorithm you created in section 1 of this chapter as the
/// inner engine, create a PoW chain that is valid according to the inner consensus engine, but is
/// not valid according to this engine because the state roots are not all even.
fn almost_valid_but_not_all_even() -> Vec<Header<u64>> {
	// Generate headers with alternating even and odd state roots
    let mut headers = Vec::new();
    let mut is_even = true;
    for i in 0..10 {
        let state_root = if is_even { i * 2 } else { i * 2 + 1 };
        let header = Header {
            parent: 0,
            height: i as u64,
            extrinsics_root: 0,
            state_root,
            consensus_digest: 0,
        };
        headers.push(header);
        is_even = !is_even; // Toggle between even and odd state roots
    }
    headers
}
