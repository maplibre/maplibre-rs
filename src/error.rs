#[derive(Debug)]
pub enum Error {
    Network(String),
    File(String),
}