pub mod uni_v2_swapper {
    use crate::states::uni_v2;
    use alloy_primitives::U256;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SwapInput {
        pub sell_token0: bool,
        pub sell_token: Vec<u8>,
        pub seller: Vec<u8>,
        pub buy_token: Vec<u8>,
        pub sell_amount: Vec<u8>,
    }

    #[derive(Debug)]
    pub struct SwapOutput {
        pub bought_amount: Vec<u8>,
        pub sold_amount: Vec<u8>,
        pub seller: Vec<u8>,
        pub buy_token: Vec<u8>,
        pub sell_token: Vec<u8>,
    }

    pub fn swap(state: uni_v2::UniV2ReservesState, payload: SwapInput) -> SwapOutput {
        let sell_amount = U256::from_be_slice(&payload.sell_amount);

        let (reserve_in, reserve_out) = if payload.sell_token0 {
            (state.reserve0, state.reserve1)
        } else {
            (state.reserve1, state.reserve0)
        };

        assert!(
            reserve_in > U256::ZERO,
            "insufficient liquidity for token in"
        );
        assert!(
            reserve_out > U256::ZERO,
            "insufficient liquidity for token out"
        );
        assert!(sell_amount > U256::ZERO, "must sell a non-zero amount");

        let numerator = reserve_out * sell_amount;
        let denominator = reserve_in + sell_amount;
        let amount_out = numerator / denominator;

        assert!(amount_out > U256::ZERO, "insufficient output amount");
        assert!(amount_out < reserve_out, "output amount exceeds reserves");

        SwapOutput {
            bought_amount: amount_out.to_be_bytes_vec(),
            sold_amount: payload.sell_amount,
            seller: payload.seller,
            buy_token: payload.buy_token,
            sell_token: payload.sell_token,
        }
    }
}
