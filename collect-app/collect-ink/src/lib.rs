#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use ink_core::{
    env::DefaultSrmlTypes,
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
    #![env = DefaultSrmlTypes]

    event Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance,
    }

    struct Erc20 {
        /// The total supply.
        total_supply: storage::Value<Balance>,
        /// The balance of each user.
        balances: storage::HashMap<AccountId, Balance>,
    }

    impl Deploy for Erc20 {
        fn deploy(&mut self, init_value: Balance) {
            // ACTION: `set` the total supply to `init_value`
            // ACTION: `insert` the `init_value` as the `env.caller()` balance
            self.total_supply.set(init_value);
            self.balances.insert(env.caller(), init_value);
            // ACTION: Call `env.emit` with the `Transfer` event
            //   HINT: According to the ERC20 specification, we should set from to `None`
            //   HINT: Since we use `Option<AccountId>`, you need to wrap accounts in `Some()`
            env.emit(Transfer {
                from: None,
                to: Some(env.caller()),
                value: init_value
            });
        }
    }

    impl Erc20 {
        /// Returns the total number of tokens in existence.
        pub(external) fn total_supply(&self) -> Balance {
            // ACTION: Print the total supply to the node's output
            // ACTION: Return the total supply
            let total_supply = *self.total_supply;
            env.println(&format!("Erc20::total_supply = {:?}", total_supply));
            total_supply
        }

        /// Returns the balance of the given AccountId.
        pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
            // ACTION: Print the balance of `owner`
            // ACTION: Return the balance of `owner`
            //   HINT: Use `balance_of_or_zero` to get the `owner` balance
            let balance = self.balance_of_or_zero(&owner);
            env.println(&format!("Erc20::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// Transfers token from the sender to the `to` AccountId.
        pub(external) fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            // ACTION: Call the `transfer_impl` with `from` as `env.caller()`
            self.transfer_impl(env, env.caller(), to, value)
        }
    }

    impl Erc20 {
        /// Returns the balance of the AccountId or 0 if there is no balance.
        fn balance_of_or_zero(&self, of: &AccountId) -> Balance {
            // ACTION: `get` the balance of `of`, then `unwrap_or` fallback to 0
            // ACTION: Return the balance
            *self.balances.get(of).unwrap_or(&0)
        }

        /// Transfers token from a specified AccountId to another AccountId.
        fn transfer_impl(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, from: AccountId, to: AccountId, value: Balance) -> bool {
            // ACTION: Get the balance for `from` and `to`
            //   HINT: Use the `balance_of_or_zero` function to do this
            // ACTION: If `balance` from is less than `value`, return `false`
            // ACTION: Insert new values for `from` and `to`
            //         * balance_from - value
            //         * balance_to + value
            let balance_from = self.balance_of_or_zero(&from);
            let balance_to = self.balance_of_or_zero(&to);
            if balance_from < value {
                return false
            }
            self.balances.insert(from, balance_from - value);
            self.balances.insert(to, balance_to + value);
            // ACTION: Call `env.emit` with the `Transfer` event
            //   HINT: Since we use `Option<AccountId>`, you need to wrap accounts in `Some()`
            env.emit( Transfer {
                from: Some(from),
                to: Some(to),
                value
            });
            true
        }
    }
}


#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn deployment_works() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);

        // Deploy the contract with some `init_value`
        let erc20 = Erc20::deploy_mock(1234);
        // Check that the `total_supply` is `init_value`
        assert_eq!(erc20.total_supply(), 1234);
        // Check that `balance_of` Alice is `init_value`
        assert_eq!(erc20.balance_of(alice), 1234);
    }

    #[test]
    fn transfer_works() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);

        env::test::set_caller::<Types>(alice);
        // Deploy the contract with some `init_value`
        let mut erc20 = Erc20::deploy_mock(1234);
        // Alice does not have enough funds for this
        assert_eq!(erc20.transfer(bob, 4321), false);
        // Alice can do this though
        assert_eq!(erc20.transfer(bob, 234), true);
        // Check Alice and Bob have the expected balance
        assert_eq!(erc20.balance_of(alice), 1000);
        assert_eq!(erc20.balance_of(bob), 234);
    }

    #[test]
    fn events_work() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);

        // No events to start
        env::test::set_caller::<Types>(alice);
        assert_eq!(env::test::emitted_events::<Types>().count(), 0);
        // Event should be emitted for initial minting
        let mut erc20 = Erc20::deploy_mock(1234);
        assert_eq!(env::test::emitted_events::<Types>().count(), 1);
        // Event should be emitted for transfers
        assert_eq!(erc20.transfer(bob, 10), true);
        assert_eq!(env::test::emitted_events::<Types>().count(), 2);
    }
}
