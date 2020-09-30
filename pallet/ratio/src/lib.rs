#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, Input, Output, Error as CodecError};

use sp_runtime::{RuntimeDebug, DispatchResult, DispatchError};
use sp_runtime::traits::{
	CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Saturating, AtLeast32Bit,
	Zero, Bounded, AtLeast32BitUnsigned
};

use sp_std::prelude::*;
use sp_std::{cmp, result, fmt::Debug};
use sp_std::collections::btree_map::BTreeMap;
use frame_support::{
	decl_event, decl_module, decl_storage, ensure, decl_error,
	traits::{
		Currency, ExistenceRequirement, Imbalance, LockIdentifier, LockableCurrency,
		ReservableCurrency, WithdrawReason, WithdrawReasons, TryDrop,
		BalanceStatus, ExistenceRequirement::KeepAlive, ExistenceRequirement::AllowDeath
	},
	Parameter, StorageMap,
};
use frame_system::{ensure_signed, ensure_root};


#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum TrackingType {
    AddValue,
    UseValue,
}

pub trait Trait: frame_system::Trait {
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + Debug +
		MaybeSerializeDeserialize;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Ratio {
        /// The number of units of assets held by any given account.
        /// WARNING: logic assumes AddValue > UseValue
        Balances: 
            double_map 
            hasher(blake2_128_concat) T::AccountId,
            hasher(blake2_128_concat) TrackingType 
            => T::Balance;
		/// The total unit supply, 
        TotalSupply: map hasher(twox_64_concat) TrackingType => T::Balance;
	}
}

decl_module!{
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
        
        #[weight(0)]
        fn transfer(origin,
			target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::Balance
        ){
            let sender = ensure_signed(origin)?;
            Self::transfer(sender, target, amount, AllowDeath);
        }
    }
}

impl<T: Trait> Currency<T::AccountId> for Module<T> {

	/// The balance of an account.
	type Balance: T::Balance;

    /// The opaque token type for an imbalance. This is returned by unbalanced operations
    /// and must be dealt with. It may be dropped but cannot be cloned.
    type PositiveImbalance: Imbalance<
        BTreeMap<TrackingType, Self::Balance>,
        Opposite=Self::NegativeImbalance
    >;

    /// The opaque token type for an imbalance. This is returned by unbalanced operations
    /// and must be dealt with. It may be dropped but cannot be cloned.
    type NegativeImbalance: Imbalance<
        BTreeMap<TrackingType, Self::Balance>, 
        Opposite=Self::PositiveImbalance
    >;

    impl Drop for PositiveImbalance {
        fn drop(&mut self){
            for (value_type, value) in self.0.iter() {
                <TotalSupply<T>>::mutate(value_type,
                    |v| *v = v.saturating_add(value)
                )
            }
        }
    }

    impl Drop for NegativeImbalance {
        fn drop(&mut self){
            for (value_type, value) in self.0.iter() {
                <TotalSupply<T>>::mutate(value_type,
                    |v| *v = v.saturating_sub(value)
                )
            }
        }
    }

    /// The combined balance of `who`.
    fn total_balance(who: &T::AccountId) -> Self::Balance{
        <Balances<T>>::iter_prefix(who).fold(0,|state, &element|{
            match element.0 {
                TrackingType::AddValue => state + element.1,
                TrackingType::UseValue => state - element.1,
            }
        })
    }

    /// Same result as `slash(who, value)` (but without the side-effects) assuming there are no
    /// balance changes in the meantime and only the reserved balance is not taken into account.
    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool{
        Self::total_balance(who) > value 
    }
    
    /// The total amount of issuance in the system.
    fn total_issuance() -> Self::Balance{
        <TotalSupply<T>>::iter().fold(0,|state, &element|{
            match element.0 {
                TrackingType::AddValue => state + element.1,
                TrackingType::UseValue => state - element.1,
            }
        })
    }
    
    /// The minimum balance any single account may have. This is equivalent to the `Balances` module's
    /// `ExistentialDeposit`.
    fn minimum_balance() -> Self::Balance{
        //TODO
        0.into()
    }
    
    /// Reduce the total issuance by `amount` and return the according imbalance. The imbalance will
    /// typically be used to reduce an account by the same amount with e.g. `settle`.
    ///
    /// This is infallible, but doesn't guarantee that the entire `amount` is burnt, for example
    /// in the case of underflow.
    fn burn(amount: Self::Balance) -> Self::PositiveImbalance{
        if amount.is_zero() { return PositiveImbalance::zero() }
    }
    
    /// Increase the total issuance by `amount` and return the according imbalance. The imbalance
    /// will typically be used to increase an account by the same amount with e.g.
    /// `resolve_into_existing` or `resolve_creating`.
    ///
    /// This is infallible, but doesn't guarantee that the entire `amount` is issued, for example
    /// in the case of overflow.
    fn issue(amount: Self::Balance) -> Self::NegativeImbalance;
    
    /// The 'free' balance of a given account.
    ///
    /// This is the only balance that matters in terms of most operations on tokens. It alone
    /// is used to determine the balance when in the contract execution environment. When this
    /// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
    /// deleted: specifically `FreeBalance`.
    ///
    /// `system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
        /// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
        fn free_balance(who: &T::AccountId) -> Self::Balance;
        
        /// Returns `Ok` iff the account is able to make a withdrawal of the given amount
        /// for the given reason. Basically, it's just a dry-run of `withdraw`.
        ///
        /// `Err(...)` with the reason why not otherwise.
        fn ensure_can_withdraw(
            who: &AccountId,
            _amount: Self::Balance,
            reasons: WithdrawReasons,
            new_balance: Self::Balance,
        ) -> DispatchResult;
        
        // PUBLIC MUTABLES (DANGEROUS)
        
        /// Transfer some liquid free balance to another staker.
        ///
        /// This is a very high-level function. It will ensure all appropriate fees are paid
        /// and no imbalance in the system remains.
        fn transfer(
            source: &T::AccountId,
            dest: &T::AccountId,
            value: Self::Balance,
            existence_requirement: ExistenceRequirement,
        ) -> DispatchResult;
        
        /// Deducts up to `value` from the combined balance of `who`, preferring to deduct from the
        /// free balance. This function cannot fail.
        ///
        /// The resulting imbalance is the first item of the tuple returned.
        ///
        /// As much funds up to `value` will be deducted as possible. If this is less than `value`,
        /// then a non-zero second item will be returned.
        fn slash(
            who: &T::AccountId,
            value: Self::Balance
        ) -> (Self::NegativeImbalance, Self::Balance);
        
        /// Mints `value` to the free balance of `who`.
        ///
        /// If `who` doesn't exist, nothing is done and an Err returned.
        fn deposit_into_existing(
            who: &T::AccountId,
            value: Self::Balance
        ) -> result::Result<Self::PositiveImbalance, DispatchError>;
        
        /// Similar to deposit_creating, only accepts a `NegativeImbalance` and returns nothing on
        /// success.
        fn resolve_into_existing(
            who: &T::AccountId,
            value: Self::NegativeImbalance,
        ) -> result::Result<(), Self::NegativeImbalance> {
            let v = value.peek();
            match Self::deposit_into_existing(who, v) {
                Ok(opposite) => Ok(drop(value.offset(opposite))),
                _ => Err(value),
            }
        }
        
        /// Adds up to `value` to the free balance of `who`. If `who` doesn't exist, it is created.
        ///
        /// Infallible.
        fn deposit_creating(
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::PositiveImbalance;
        
        /// Similar to deposit_creating, only accepts a `NegativeImbalance` and returns nothing on
        /// success.
        fn resolve_creating(
            who: &T::AccountId,
            value: Self::NegativeImbalance,
        ) {
            let v = value.peek();
            drop(value.offset(Self::deposit_creating(who, v)));
        }
        
        /// Removes some free balance from `who` account for `reason` if possible. If `liveness` is
        /// `KeepAlive`, then no less than `ExistentialDeposit` must be left remaining.
        ///
        /// This checks any locks, vesting, and liquidity requirements. If the removal is not possible,
        /// then it returns `Err`.
        ///
        /// If the operation is successful, this will return `Ok` with a `NegativeImbalance` whose value
        /// is `value`.
        fn withdraw(
            who: &T::AccountId,
            value: Self::Balance,
            reasons: WithdrawReasons,
            liveness: ExistenceRequirement,
        ) -> result::Result<Self::NegativeImbalance, DispatchError>;
        
        /// Similar to withdraw, only accepts a `PositiveImbalance` and returns nothing on success.
        fn settle(
            who: &T::AccountId,
            value: Self::PositiveImbalance,
            reasons: WithdrawReasons,
            liveness: ExistenceRequirement,
        ) -> result::Result<(), Self::PositiveImbalance> {
            let v = value.peek();
            match Self::withdraw(who, v, reasons, liveness) {
                Ok(opposite) => Ok(drop(value.offset(opposite))),
                _ => Err(value),
            }
        }
        
        /// Ensure an account's free balance equals some value; this will create the account
        /// if needed.
        ///
        /// Returns a signed imbalance and status to indicate if the account was successfully updated or update
        /// has led to killing of the account.
        fn make_free_balance_be(
            who: &T::AccountId,
            balance: Self::Balance,
        ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance>;
}
    