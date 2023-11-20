use candid::Principal;

const OWNERS: [&str; 3] = [
    "syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae",
    "syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae",
    "syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae",
];

pub fn is_owner() -> Result<(), String> {
    let caller = ic_cdk::caller();
    match OWNERS
        .iter()
        .any(|p| Principal::from_text(p).unwrap() == caller)
    {
        true => Ok(()),
        false => Err("Unauthorized".to_string()),
    }
}
