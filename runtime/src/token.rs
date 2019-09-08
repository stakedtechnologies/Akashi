use system;
use parity_codec::{Encode, Decode};
use balances;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Token<T:system::Trait+balances::Trait>{
	pub id:T::Hash,
    pub nonce:T::Index,
	pub deposit:T::Balance,
	pub issued:T::Balance
}

impl<T:system::Trait+balances::Trait> Token<T> {
	pub fn new(token_id:T::Hash,nonce:T::Index,deposit:T::Balance,issued:T::Balance)->Self {
		let new_token = Token {
			id:token_id,
			nonce:nonce,
			deposit:deposit,
			issued:issued
		};
        new_token
	}
}


