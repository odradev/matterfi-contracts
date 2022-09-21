use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;

#[derive(WrapperApi)]
pub struct SharedParser {
    hello: fn() -> u32,
    bytes_to_address: fn(bytes: &[u8]) -> Vec<u8>
}

fn main() {
    let parser: Container<SharedParser> = unsafe {
        let filename = format!(
            r"../shared-parser/target/release/{}shared_parser.{}",
            dlopen::utils::PLATFORM_FILE_PREFIX,
            dlopen::utils::PLATFORM_FILE_EXTENSION
        );
        Container::load(filename).expect("Could not open library or load symbols")
    };

    let bytes = "4500000040000000003b4ffcfb21411ced5fc1560c3f6ffed86f4885e5ea05cde49d90962a48a14d950000000000000000000000000000000000000000000000000000000000000015";
    let bytes = hex::decode(bytes).unwrap();
    let result = parser.bytes_to_address(&bytes);
    let result = String::from_utf8(result);
    println!("{:?}, world!", result);
}
