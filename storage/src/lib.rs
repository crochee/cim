pub mod convert;
pub mod groups;
pub mod keys;
mod model;
pub mod policies;
mod pool;
pub mod roles;
pub mod users;

pub use model::{List, Pagination, ID};
pub use pool::connection_manager;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
