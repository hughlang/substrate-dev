#![cfg_attr(not(any(test, feature = "std")), no_std)]

// Import the `contract!` macro
use ink_lang::contract;
use ink_core::storage;
use ink_core::env::DefaultSrmlTypes;
use ink_core::memory::format;

contract! {
    #![env = DefaultSrmlTypes]

    struct Incrementer {
        // Storage Declaration
        owner: storage::Value<AccountId>,
        value: storage::Value<u64>,
        my_value: storage::HashMap<AccountId, u64>,
    }

    impl Deploy for Incrementer {
        fn deploy(&mut self, init_value: u64) {
            // Contract Constructor
            self.owner.set(env.caller());
            self.value.set(init_value);
        }
    }

    // Public: Implementation of Contract Functions
    impl Incrementer {
        pub(external) fn get(&self) -> u64 {
            // ACTION: Use `env.println` to print the value of `self.value`
            // ACTION: Return `self.value`
            env.println(&format!("Incrementer::get = {:?}", *self.value));
            *self.value
        }

        pub(external) fn inc(&mut self, by: u64) {
            self.value += by;
        }

        pub(external) fn get_mine(&self) -> u64 {
            // ACTION: Get `my_value` using `my_value_or_zero` on `env.caller()`
            // ACTION: Print `my_value` to simplify on-chain testing
            // ACTION: Return `my_value` at the end
            let caller = env.caller();
            let value = self.my_value_or_zero(&caller);
            env.println(&format!("Your value is {:?}", value));
            value
        }

        pub(external) fn inc_mine(&mut self, by: u64) {
            // ACTION: Get `my_value` using `my_value_or_zero` to get the current value of `env.caller()`
            // ACTION: Insert the new value `(my_value + by)` back into the mapping

            let caller = env.caller();
            let my_number = self.my_value_or_zero(&caller);
            self.my_value.insert(caller, my_number + by);
        }
    }

    // Private methods
    impl Incrementer {
        fn my_value_or_zero(&self, of: &AccountId) -> u64 {
            // ACTION: `get` the value of `of` and `unwrap_or` return 0
            // ACTION: Return the value at the end
            let balance = self.my_value.get(of).unwrap_or(&0);
            *balance
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    #[test]
    fn incrementer_works() {
        // Test Your Contract
    }
}

