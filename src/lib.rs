// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::{aliases::U24, Address, U256};
use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, FixedBytes as SOLFixedBytes, *},
    SolType,
};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::FixedBytes, crypto::keccak, prelude::*};

const MIN_LIQUIDITY: u64 = 1000;

sol! {
    error PoolAlreadyExists(bytes32 pool_id);
    error PoolDoesNotExist(bytes32 pool_id);
    error InsufficientLiquidityMinted();
    error InsufficientAmount();
    error InsufficientLiquidityOwned();
    error FailedOrInsufficientTokenTransfer(address token, address from, address to, uint256 amount);
    error TooMuchSlippage();
}

sol_interface! {
    interface IERC20 {
        function transferFrom(address from, address to, uint256 value) external returns (bool);
        function transfer(address to, uint256 value) external returns (bool);
    }
}

// Define some persistent storage using the Solidity ABI.
// `StylusSwap` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct StylusSwap {
        mapping(bytes32 => Pool) pools;
    }

    pub struct Pool {
        address token0;
        address token1;
        uint24 fee;
        uint256 liquidity;
        uint256 balance0;
        uint256 balance1;
        mapping(bytes32 => Position) positions;
    }

    pub struct Position {
        address owner;
        uint256 liquidity;
    }

}

#[derive(SolidityError)]
pub enum StylusSwapError {
    PoolAlreadyExists(PoolAlreadyExists),
    PoolDoesNotExist(PoolDoesNotExist),
    InsufficientAmount(InsufficientAmount),
    InsufficientLiquidityMinted(InsufficientLiquidityMinted),
    InsufficientLiquidityOwned(InsufficientLiquidityOwned),
    FailedOrInsufficientTokenTransfer(FailedOrInsufficientTokenTransfer),
    TooMuchSlippage(TooMuchSlippage),
}

/// Internal methods for the contract
impl StylusSwap {
    fn integer_sqrt(&self, x: U256) -> U256 {
        let two = U256::from(2);

        let mut z: U256 = (x + U256::from(1)) >> 1;
        let mut y = x;

        while z < y {
            y = z;
            z = (x / z + z) / two;
        }

        y
    }

    fn min(&self, x: U256, y: U256) -> U256 {
        if x < y {
            return x;
        }

        y
    }
}

/// Declare that `StylusSwap` is a contract with the following external methods.
#[public]
impl StylusSwap {
    pub fn create_pool(
        &mut self,
        token_a: Address,
        token_b: Address,
        fee: U24,
    ) -> Result<(), StylusSwapError> {
        let (pool_id, token0, token1) = self.get_pool_id(token_a, token_b, fee);
        let existing_pool = self.pools.get(pool_id);

        if !existing_pool.token0.is_zero() || !existing_pool.token1.is_zero() {
            return Err(StylusSwapError::PoolAlreadyExists(PoolAlreadyExists {
                pool_id,
            }));
        }

        let mut pool_setter = self.pools.setter(pool_id);
        pool_setter.token0.set(token0);
        pool_setter.token1.set(token1);
        pool_setter.fee.set(fee);
        pool_setter.liquidity.set(U256::from(0));
        pool_setter.balance0.set(U256::from(0));
        pool_setter.balance1.set(U256::from(0));

        Ok(())
    }

    #[payable]
    pub fn add_liquidity(
        &mut self,
        pool_id: FixedBytes<32>,
        amount_0_desired: U256,
        amount_1_desired: U256,
        amount_0_min: U256,
        amount_1_min: U256,
    ) -> Result<(), StylusSwapError> {
        let msg_sender = self.vm().msg_sender();
        let address_this = self.vm().contract_address();

        let pool = self.pools.get(pool_id);
        let token0 = pool.token0.get();
        let token1 = pool.token1.get();
        let balance_0 = pool.balance0.get();
        let balance_1 = pool.balance1.get();
        let liquidity = pool.liquidity.get();

        if token0.is_zero() && token1.is_zero() {
            return Err(StylusSwapError::PoolDoesNotExist(PoolDoesNotExist {
                pool_id,
            }));
        }

        let position_id = self.get_position_id(pool_id, msg_sender);
        let user_position = pool.positions.get(position_id);
        let user_liquidity = user_position.liquidity.get();

        let is_initial_liquidity = liquidity == U256::from(0);

        let (amount_0, amount_1) = self.get_liquidity_amounts(
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
            balance_0,
            balance_1,
        )?;

        let new_user_liquidity = if is_initial_liquidity {
            self.integer_sqrt(amount_0 * amount_1)
        } else {
            let l_0 = (amount_0 * liquidity) / balance_0;
            let l_1 = (amount_1 * liquidity) / balance_1;
            self.min(l_0, l_1)
        };

        let new_pool_liquidity = if is_initial_liquidity {
            new_user_liquidity + U256::from(MIN_LIQUIDITY)
        } else {
            new_user_liquidity
        };

        if new_pool_liquidity <= U256::ZERO {
            return Err(StylusSwapError::InsufficientLiquidityMinted(
                InsufficientLiquidityMinted {},
            ));
        }

        let mut pool_setter = self.pools.setter(pool_id);
        pool_setter.liquidity.set(new_pool_liquidity + liquidity);
        pool_setter.balance0.set(balance_0 + amount_0);
        pool_setter.balance1.set(balance_1 + amount_1);
        let mut position_setter = pool_setter.positions.setter(position_id);
        position_setter
            .liquidity
            .set(user_liquidity + new_user_liquidity);
        position_setter.owner.set(msg_sender);

        if token0.is_zero() {
            if self.vm().msg_value() < amount_0 {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: token0,
                        from: msg_sender,
                        to: address_this,
                        amount: amount_0,
                    },
                ));
            }
        } else {
            let token_0_contract = IERC20::new(token0);
            let result =
                token_0_contract.transfer_from(&mut *self, msg_sender, address_this, amount_0);
            if result.is_err() || result.unwrap() == false {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: token0,
                        from: msg_sender,
                        to: address_this,
                        amount: amount_0,
                    },
                ));
            }
        }

        let token_1_contract = IERC20::new(token1);
        let result = token_1_contract.transfer_from(&mut *self, msg_sender, address_this, amount_1);
        if result.is_err() || result.unwrap() == false {
            return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                FailedOrInsufficientTokenTransfer {
                    token: token0,
                    from: msg_sender,
                    to: address_this,
                    amount: amount_1,
                },
            ));
        }

        Ok(())
    }

    pub fn remove_liquidity(
        &mut self,
        pool_id: FixedBytes<32>,
        liquidity_to_remove: U256,
    ) -> Result<(), StylusSwapError> {
        let msg_sender = self.vm().msg_sender();
        let address_this = self.vm().contract_address();

        let pool = self.pools.get(pool_id);
        let token0 = pool.token0.get();
        let token1 = pool.token1.get();
        let balance_0 = pool.balance0.get();
        let balance_1 = pool.balance1.get();
        let pool_liquidity = pool.liquidity.get();

        if token0.is_zero() && token1.is_zero() {
            return Err(StylusSwapError::PoolDoesNotExist(PoolDoesNotExist {
                pool_id,
            }));
        }

        let position_id = self.get_position_id(pool_id, msg_sender);
        let user_position = pool.positions.get(position_id);
        let user_liquidity = user_position.liquidity.get();

        if liquidity_to_remove > user_liquidity {
            return Err(StylusSwapError::InsufficientLiquidityOwned(
                InsufficientLiquidityOwned {},
            ));
        }

        let amount_0 = (balance_0 * liquidity_to_remove) / pool_liquidity;
        let amount_1 = (balance_1 * liquidity_to_remove) / pool_liquidity;

        if amount_0 <= U256::ZERO || amount_1 <= U256::ZERO {
            return Err(StylusSwapError::InsufficientLiquidityOwned(
                InsufficientLiquidityOwned {},
            ));
        }

        let mut pool_setter = self.pools.setter(pool_id);
        pool_setter
            .liquidity
            .set(pool_liquidity - liquidity_to_remove);
        pool_setter.balance0.set(balance_0 - amount_0);
        pool_setter.balance1.set(balance_1 - amount_1);
        let mut position_setter = pool_setter.positions.setter(position_id);
        position_setter
            .liquidity
            .set(user_liquidity - liquidity_to_remove);

        if token0.is_zero() {
            let result = self.vm().transfer_eth(msg_sender, amount_0);
            if result.is_err() {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: token0,
                        from: address_this,
                        to: msg_sender,
                        amount: amount_0,
                    },
                ));
            }
        } else {
            let token_0_contract = IERC20::new(token0);
            let result = token_0_contract.transfer(&mut *self, msg_sender, amount_0);
            if result.is_err() || result.unwrap() == false {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: token0,
                        from: address_this,
                        to: msg_sender,
                        amount: amount_0,
                    },
                ));
            }
        }

        let token_1_contract = IERC20::new(token1);
        let result = token_1_contract.transfer(&mut *self, msg_sender, amount_1);
        if result.is_err() || result.unwrap() == false {
            return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                FailedOrInsufficientTokenTransfer {
                    token: token0,
                    from: address_this,
                    to: msg_sender,
                    amount: amount_1,
                },
            ));
        }

        Ok(())
    }

    #[payable]
    pub fn swap(
        &mut self,
        pool_id: FixedBytes<32>,
        input_amount: U256,
        min_output_amount: U256,
        zero_for_one: bool,
    ) -> Result<(), StylusSwapError> {
        if input_amount == U256::ZERO {
            return Err(StylusSwapError::InsufficientAmount(InsufficientAmount {}));
        }

        let msg_sender = self.vm().msg_sender();
        let address_this = self.vm().contract_address();

        let pool = self.pools.get(pool_id);
        let token0 = pool.token0.get();
        let token1 = pool.token1.get();
        let balance0 = pool.balance0.get();
        let balance1 = pool.balance1.get();
        let fee = pool.fee.get();

        let k = balance0 * balance1;

        let input_token = if zero_for_one { token0 } else { token1 };
        let output_token = if zero_for_one { token1 } else { token0 };

        let input_balance = if zero_for_one { balance0 } else { balance1 };
        let output_balance = if zero_for_one { balance1 } else { balance0 };

        let output_amount = output_balance - (k / (input_balance + input_amount));
        let fees = (output_amount * U256::from(fee)) / U256::from(10_000);
        let output_amount_after_fees = output_amount - fees;

        if output_amount_after_fees < min_output_amount {
            return Err(StylusSwapError::TooMuchSlippage(TooMuchSlippage {}));
        }

        let mut pool_setter = self.pools.setter(pool_id);
        if zero_for_one {
            pool_setter.balance0.set(balance0 + input_amount);
            pool_setter
                .balance1
                .set(balance1 - output_amount_after_fees);
        } else {
            pool_setter
                .balance0
                .set(balance0 - output_amount_after_fees);
            pool_setter.balance1.set(balance1 + input_amount);
        }

        if input_token.is_zero() {
            if self.vm().msg_value() < input_amount {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: token0,
                        from: msg_sender,
                        to: address_this,
                        amount: input_amount,
                    },
                ));
            }
        } else {
            let input_token_contract = IERC20::new(input_token);
            let result = input_token_contract.transfer_from(
                &mut *self,
                msg_sender,
                address_this,
                input_amount,
            );
            if result.is_err() || result.unwrap() == false {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: input_token,
                        from: msg_sender,
                        to: address_this,
                        amount: input_amount,
                    },
                ));
            }
        }

        if output_token.is_zero() {
            let result = self.vm().transfer_eth(msg_sender, output_amount_after_fees);
            if result.is_err() {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: output_token,
                        from: address_this,
                        to: msg_sender,
                        amount: output_amount_after_fees,
                    },
                ));
            }
        } else {
            let output_token_contract = IERC20::new(output_token);
            let result =
                output_token_contract.transfer(&mut *self, msg_sender, output_amount_after_fees);
            if result.is_err() || result.unwrap() == false {
                return Err(StylusSwapError::FailedOrInsufficientTokenTransfer(
                    FailedOrInsufficientTokenTransfer {
                        token: output_token,
                        from: address_this,
                        to: msg_sender,
                        amount: output_amount_after_fees,
                    },
                ));
            }
        }
        Ok(())
    }

    pub fn get_liquidity_amounts(
        &self,
        amount_0_desired: U256,
        amount_1_desired: U256,
        amount_0_min: U256,
        amount_1_min: U256,
        balance_0: U256,
        balance_1: U256,
    ) -> Result<(U256, U256), StylusSwapError> {
        let amount_1_given_0 = (amount_0_desired * balance_1) / balance_0;
        let amount_0_given_1 = (amount_1_desired * balance_0) / balance_1;

        if amount_1_given_0 <= amount_1_desired {
            if amount_1_given_0 >= amount_1_min {
                return Err(StylusSwapError::InsufficientAmount(InsufficientAmount {}));
            }
            return Ok((amount_0_desired, amount_1_given_0));
        }

        if amount_0_given_1 <= amount_0_desired {
            return Err(StylusSwapError::InsufficientAmount(InsufficientAmount {}));
        }

        if amount_0_given_1 >= amount_0_min {
            return Err(StylusSwapError::InsufficientAmount(InsufficientAmount {}));
        }

        return Ok((amount_0_given_1, amount_1_desired));
    }

    pub fn get_position_id(&self, pool_id: FixedBytes<32>, owner: Address) -> FixedBytes<32> {
        type HashType = (SOLFixedBytes<32>, SOLAddress);
        let tx_hash_data = (pool_id, owner);
        let encoded_data = HashType::abi_encode_sequence(&tx_hash_data);

        keccak(encoded_data).into()
    }

    pub fn get_pool_id(
        &self,
        token_a: Address,
        token_b: Address,
        fee: U24,
    ) -> (FixedBytes<32>, Address, Address) {
        let token0: Address;
        let token1: Address;

        if token_a <= token_b {
            token0 = token_a;
            token1 = token_b;
        } else {
            token0 = token_b;
            token1 = token_a;
        }

        // // define sol types tuple
        type HashType = (SOLAddress, SOLAddress, Uint<24>);
        let tx_hash_data = (token0, token1, fee);
        // // encode the tuple
        let tx_hash_data_encode = HashType::abi_encode_sequence(&tx_hash_data);
        // // hash the encoded data
        let pool_id = keccak(tx_hash_data_encode).into();
        (pool_id, token0, token1)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_create_pool() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = StylusSwap::from(&vm);

        let token_a = address!("0x0000000000000000000000000000000000000000");
        let token_b = address!("0x0000000000000000000000000000000000000001");
        let fee = U24::from(300);

        let (pool_id, token0, token1) = contract.get_pool_id(token_a, token_b, fee);
        let create_pool_result = contract.create_pool(token_a, token_b, U24::from(300));
        assert!(create_pool_result.is_ok());

        let pool_info = contract.pools.get(pool_id);
        assert_eq!(fee, pool_info.fee.get());
        assert_eq!(token0, pool_info.token0.get());
        assert_eq!(token1, pool_info.token1.get());
        assert_eq!(U256::from(0), pool_info.liquidity.get());
        assert_eq!(U256::from(0), pool_info.balance0.get());
        assert_eq!(U256::from(0), pool_info.balance1.get());
    }
}
