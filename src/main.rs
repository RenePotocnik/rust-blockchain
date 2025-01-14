extern crate chrono;
extern crate serde;
extern crate sha2;
mod models;

fn main() {
    let difficulty = 2;
    let mut blockchain = models::blockchain::Blockchain::new(difficulty);
    models::blockchain::Blockchain::add_block(&mut blockchain);
}
