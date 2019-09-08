use system;
use parity_codec::{Encode, Decode};
use balances;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct State<T:system::Trait+balances::Trait>{
    pub nonce:T::Index,
	pub token:T::Hash,
	pub owner:T::AccountId,
	pub amount:T::Balance
}



impl<T:system::Trait+balances::Trait> State<T>  {
	pub fn new(nonce:T::Index,token_id:T::Hash,owner:T::AccountId,amount:T::Balance)->Self {
		let new_state = State {
            nonce:nonce,
			token:token_id,
			owner:owner,
			amount:amount
		};
        new_state
	}
}

