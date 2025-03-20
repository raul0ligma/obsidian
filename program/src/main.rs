#![no_main]
sp1_zkvm::entrypoint!(main);

use obsidian_lib::{
    states::uni_v2,
    verifier::{MPTVerifier, VerifierInputs, VerifierOutput},
};

pub fn main() {
    let inputs: VerifierInputs = sp1_zkvm::io::read::<VerifierInputs>();

    let out: VerifierOutput = MPTVerifier::verify_slot(inputs).unwrap();
    let reserves = uni_v2::UniV2ReservesState::try_from(out.slot_data.clone());
    println!("{:0x?}", reserves);
    sp1_zkvm::io::commit_slice(&out.slot_data);
}
