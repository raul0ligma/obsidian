pub mod uni_v2 {
    use alloy_primitives::U256;
    #[derive(Debug)]
    pub struct UniV2ReservesState {
        pub reserve0: U256,
        pub reserve1: U256,
        pub block_timestamp_last: U256,
    }

    impl TryFrom<Vec<u8>> for UniV2ReservesState {
        type Error = String;

        fn try_from(serialized: Vec<u8>) -> Result<Self, Self::Error> {
            if serialized.len() != 32 {
                return Err(format!(
                    "expected encoded length to be 32 found {}",
                    serialized.len()
                ));
            }

            // the reservers are encoded as [4][14][14] = 32
            Ok(UniV2ReservesState {
                block_timestamp_last: U256::from_be_slice(&serialized[..4]),
                reserve1: U256::from_be_slice(&serialized[5..18]),
                reserve0: U256::from_be_slice(&serialized[19..]),
            })
        }
    }
}
