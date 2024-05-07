use cim_uid::Uid;

#[derive(Debug, Uid)]
pub struct User {
    pub id: i32,
    pub name: String,
}

fn main() {
    let user = User {
        id: 0,
        name: "gs".to_string(),
    };
    println!("{:#?},{}", user, user.uid());
}
