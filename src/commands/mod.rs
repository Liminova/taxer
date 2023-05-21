use rand::Rng;
pub mod ping;
pub mod satisfaction;
pub(crate) fn out_of_ten() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(-1..11)
}
