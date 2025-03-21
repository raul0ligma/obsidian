#![no_main]
sp1_zkvm::entrypoint!(main);

use obsidian_lib::{
    pack_order,
    states::uni_v2,
    swapper::uni_v2_swapper,
    verifier::{MPTVerifier, VerifierOutput},
    ObsidianInput,
};

pub fn main() {
    let input: ObsidianInput = sp1_zkvm::io::read::<ObsidianInput>();
    let block_number = input.block_verifier_inputs.header.number;

    let out: VerifierOutput = MPTVerifier::verify_slot(input.block_verifier_inputs).unwrap();
    let reserves_state: uni_v2::UniV2ReservesState =
        uni_v2::UniV2ReservesState::try_from(out.slot_data.clone()).unwrap();

    let swap_out = uni_v2_swapper::swap(reserves_state, input.swap_payload);

    let order = pack_order(swap_out, block_number, out.block_hash);

    sp1_zkvm::io::commit_slice(&order);
}
