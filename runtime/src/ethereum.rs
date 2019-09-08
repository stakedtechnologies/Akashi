use support::{decl_module, decl_storage, StorageValue, StorageMap, decl_event, ensure};
use support::dispatch::{Result,Vec};
use system::ensure_signed;
use parity_codec::{Encode, Decode};
use balances;
use runtime_primitives::traits::{Hash,As};
use primitives::{H256, sr25519};
use crate::state::{State};
use crate::token::{Token};



pub trait Trait: system::Trait+balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct EthereumHeader<Hash,BlockNumber,Balance> {
	pub hash: Hash,
    pub parent_hash: Hash,
    pub uncles_hash: Hash,
    pub author: Vec<u8>,
    pub state_root: Hash,
    pub transactions_root: Hash,
    pub receipts_root: Hash,
    pub number: BlockNumber,
    pub gas_used: Balance,
    pub gas_limit: Balance,
    pub extra_data: Vec<u8>,
    pub logs_bloom: Vec<u8>,
    pub timestamp: u16,
    pub difficulty: Vec<u8>,
    pub mix_hash: Hash,
    pub nonce: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct EthereumTx<Hash,BlockNumber,Balance> {
	pub hash: Hash,
    pub nonce: u64,
    pub block_hash: Hash,
    pub block_number: BlockNumber,
    pub transactions_index: u16,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub value: Balance,
    pub gas_price: Balance,
    pub gas: Balance,
    pub input: Vec<u8>,
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct EthereumData<Hash,BlockNumber,Balance> {
	header:EthereumHeader<Hash,BlockNumber,Balance>,
	txs:Vec<EthereumTx<Hash,BlockNumber,Balance>>
}


decl_storage! {
	pub trait Store for Module<T: Trait> as EthereumStorage {
		pub Init get(already): bool = false;
		pub Nonce get(db_nonce): T::Index = <T::Index as As<u64>>::sa(0);
		pub OwnedState get(states_of_owner): map (T::AccountId,T::Index) => Option<State<T>>;
		pub StateNonce get(nonce_of_state): map (T::AccountId) => Option<T::Index>;
		pub IndexedToken get(token_of_nonce): map T::Index => Option<Token<T>>;
		pub LastTokenNonce get(last_token_nonce): T::Index = <T::Index as As<u64>>::sa(0);
		pub TokenData get(data_of_token): map T::Index => Option<EthereumData<T::Hash,T::BlockNumber,T::Balance>>;
		pub LastDataNonce get(nonce_of_data): T::Index = <T::Index as As<u64>>::sa(0);
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn init(origin,data:EthereumData<T::Hash,T::BlockNumber,T::Balance>) -> Result {
			ensure_signed(origin)?;
			ensure!(Self::already()==false,"already init");
			let ethereum_token_id = ("ethereum").using_encoded(T::Hashing::hash);
			let zero_index = <T::Index as As<u64>>::sa(0);
			let first_index = <T::Index as As<u64>>::sa(<T::BlockNumber as As<u64>>::as_(data.header.number));
			let zero_balance = <T::Balance as As<u64>>::sa(0);
			let first_token:Token<T> = Token::new(ethereum_token_id as T::Hash,first_index as T::Index,zero_balance as T::Balance,zero_balance as T::Balance);
			<Init<T>>::put(true);
			<Nonce<T>>::put(zero_index);
			<IndexedToken<T>>::insert(zero_index,first_token);
			<LastTokenNonce<T>>::put(zero_index);
			<TokenData<T>>::insert(first_index,data);
			<LastDataNonce<T>>::put(first_index);
			Self::deposit_event(RawEvent::TokenUpdate(first_index));
			Ok(())
		}

		pub fn record_header(origin,data:EthereumData<T::Hash,T::BlockNumber,T::Balance>)-> Result {
			let sender = ensure_signed(origin)?;
			ensure!(Self::already()==true,"not init");
			let pre_data_nonce = Self::nonce_of_data();
			let new_data_nonce = pre_data_nonce + <T::Index as As<u64>>::sa(1);
			ensure!(<T::BlockNumber as As<u64>>::as_(data.header.number)==<T::Index as As<u64>>::as_(new_data_nonce),"invalid block number");
			let pre_token_nonce = Self::last_token_nonce();
			let new_token_nonce = pre_token_nonce + <T::Index as As<u64>>::sa(1);
			let ethereum_token_id = ("ethereum").using_encoded(T::Hashing::hash);
			let pre_token = Self::token_of_nonce(pre_token_nonce).ok_or("previous token is not saved")?;
			let deposit = pre_token.deposit;
			let locked = <T::Balance as As<u64>>::sa(data.txs.iter().fold(0,|sum,tx| sum + <T::Balance as As<u64>>::as_(tx.value)));
			let issued = pre_token.issued + locked;
			let new_token:Token<T> = Token::new(ethereum_token_id as T::Hash,new_token_nonce as T::Index,deposit as T::Balance,issued as T::Balance);
			let pre_sender_nonce:T::Index = match Self::nonce_of_state(sender.clone()) {
				None=><T::Index as As<u64>>::sa(0),
				Some(nonce)=>nonce
			};
			let pre_state:State<T> = match Self::states_of_owner((sender.clone(),pre_sender_nonce)){
				None=>State::new(<T::Index as As<u64>>::sa(0),ethereum_token_id,sender.clone(),<T::Balance as As<u64>>::sa(0)),
				Some(state)=>state
			};
			let new_state_nonce = pre_sender_nonce + <T::Index as As<u64>>::sa(1);
			let new_balance = pre_state.amount + locked;
			let new_state:State<T> = State::new(new_state_nonce,ethereum_token_id,sender.clone(),new_balance);
			<Nonce<T>>::mutate(|n| *n += <T::Index as As<u64>>::sa(1));
			<OwnedState<T>>::insert((sender.clone(),new_state_nonce),new_state);
			<StateNonce<T>>::insert(sender.clone(),new_state_nonce);
			<IndexedToken<T>>::insert(new_token_nonce,new_token);
			<LastTokenNonce<T>>::put(new_token_nonce);
			<TokenData<T>>::insert(new_data_nonce,data);
			<LastDataNonce<T>>::put(new_data_nonce);
			Self::deposit_event(RawEvent::TokenUpdate(new_state_nonce));
			Self::deposit_event(RawEvent::Mint(sender,issued));
			Ok(())
		}

		pub fn remittance(origin,to:T::AccountId,value:T::Balance) -> Result {
			let from = ensure_signed(origin)?;
			ensure!(Self::already()==true,"not init");
			Self::basic_remittance(from,to,value)
		}

		pub fn unlock(origin,value:T::Balance) -> Result {
			let signer = ensure_signed(origin)?;
			ensure!(Self::already()==true,"not init");
			let signer_pre_nonce = Self::nonce_of_state(signer.clone()).ok_or("nonce of 'from' doesn't exist")?;
			let signer_pre_state = Self::states_of_owner((signer.clone(),signer_pre_nonce)).ok_or("state of 'from' doesn't exist")?;
			ensure!(signer_pre_state.amount>=value,"too large amount (state amount)");
			let ethereum_token_id = ("ethereum").using_encoded(T::Hashing::hash);
			let signer_new_nonce = signer_pre_nonce + <T::Index as As<u64>>::sa(1);
			let signer_new_amount = signer_pre_state.amount - value;
			let signer_new_state:State<T> = State::new(signer_new_nonce,ethereum_token_id,signer.clone(),signer_new_amount);
			let pre_token_nonce = Self::last_token_nonce();
			let new_token_nonce = pre_token_nonce + <T::Index as As<u64>>::sa(1);
			let ethereum_token_id = ("ethereum").using_encoded(T::Hashing::hash);
			let pre_token = Self::token_of_nonce(pre_token_nonce).ok_or("previous token is not saved")?;
			ensure!(pre_token.issued>=value,"too large amount (issued amount)");
			let deposit = pre_token.deposit;
			let issued = pre_token.issued -value;
			let new_token:Token<T> = Token::new(ethereum_token_id as T::Hash,new_token_nonce as T::Index,deposit as T::Balance,issued as T::Balance);
			<Nonce<T>>::mutate(|n| *n += <T::Index as As<u64>>::sa(1));
			<OwnedState<T>>::insert((signer.clone(),signer_new_nonce),signer_new_state);
			<StateNonce<T>>::insert(signer.clone(),signer_new_nonce);
			<IndexedToken<T>>::insert(new_token_nonce,new_token);
			<LastTokenNonce<T>>::put(new_token_nonce);
			Self::deposit_event(RawEvent::Burn(signer,signer_new_amount));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::Index,
		<T as system::Trait>::AccountId,
		<T as balances::Trait>::Balance,
	{
		TokenUpdate(Index),
		Mint(AccountId,Balance),
		Remit(AccountId,Balance,AccountId,Balance),
		Burn(AccountId,Balance),
	}
);

impl<T:Trait> Module<T> {
	fn basic_remittance(from:T::AccountId,to:T::AccountId,value:T::Balance)->Result {
		let from_pre_nonce = Self::nonce_of_state(from.clone()).ok_or("nonce of 'from' doesn't exist")?;
		let from_pre_state = Self::states_of_owner((from.clone(),from_pre_nonce)).ok_or("state of 'from' doesn't exist")?;
		ensure!(from_pre_state.amount>=value,"too large amount");
		if from==to { return Ok(()); }
		let to_pre_nonce = match Self::nonce_of_state(to.clone()) {
			None => <T::Index as As<u64>>::sa(0),
			Some(nonce) => nonce
		};
		let ethereum_token_id = ("ethereum").using_encoded(T::Hashing::hash);
		let to_pre_state:State<T> = match Self::states_of_owner((to.clone(),to_pre_nonce)) {
			None => State::new(to_pre_nonce,ethereum_token_id,to.clone(),<T::Balance as As<u64>>::sa(0)),
			Some(state) => state
		};
		let from_new_nonce = from_pre_nonce + <T::Index as As<u64>>::sa(1);
		let from_new_amount = from_pre_state.amount - value;
		let from_new_state:State<T> = State::new(from_new_nonce,ethereum_token_id,from.clone(),from_new_amount);
		let to_new_nonce = to_pre_nonce + <T::Index as As<u64>>::sa(1);
		let to_new_amount = to_pre_state.amount + value;
		let to_new_state:State<T> = State::new(to_new_nonce,ethereum_token_id,to.clone(),to_new_amount);
		<Nonce<T>>::mutate(|n| *n += <T::Index as As<u64>>::sa(1));
		<OwnedState<T>>::insert((from.clone(),from_new_nonce),from_new_state);
		<StateNonce<T>>::insert(from.clone(),from_new_nonce);
		<OwnedState<T>>::insert((to.clone(),to_new_nonce),to_new_state);
		<StateNonce<T>>::insert(to.clone(),to_new_nonce);
		Self::deposit_event(RawEvent::Remit(from,from_new_amount,to,to_new_amount));
		Ok(())
	}
}

