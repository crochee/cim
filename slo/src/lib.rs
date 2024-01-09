pub mod crypto;
pub mod errors;
mod id;
pub mod regexp;

pub type Result<T, E = errors::WithBacktrace> = core::result::Result<T, E>;

pub use id::next_id;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
