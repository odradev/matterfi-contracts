use std::{env, convert::TryInto, fmt::Debug};
use odra::types::Address as OdraAddress;
use odra_casper_shared::casper_address::CasperAddress;
use casper_types::{bytesrepr::{ToBytes, FromBytes, Bytes}, URef, Key, CLValue, AccessRights, URefAddr};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};

fn to_dictionary_item_key<T: ToBytes + Clone + Debug>(key: &T) -> String {
    let preimage = key.to_bytes().unwrap();
    let hash = blake2b(preimage);
    hex::encode(hash)
}

fn blake2b<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut result = [0; 32];
    let mut hasher = VarBlake2b::new(32).expect("should create hasher");

    hasher.update(data);
    hasher.finalize_variable(|slice| {
        result.copy_from_slice(slice);
    });
    result
}

fn parser_event(bytes: CLValue) {
    let bytes: Bytes = bytes.into_t().unwrap();
    let (event_name, bytes): (String, _) = FromBytes::from_bytes(&bytes).unwrap();
    match event_name.as_str() {
        "MasterPaymentCodeSet" => {
            let (key, bytes): (String, _) = FromBytes::from_bytes(bytes).unwrap();
            let (address, bytes): (OdraAddress, _) = FromBytes::from_bytes(bytes).unwrap();
            let (code, bytes): ([u8; 32], _) = FromBytes::from_bytes(bytes).unwrap();
            println!("Key: {}", key);
            println!("Addr: {:?}", CasperAddress::try_from(address).unwrap());
            println!("code: {:?}", code);
            assert_is_empty(bytes);
        },
        _ => panic!("Unknown event: {}", event_name)
    };
    println!("Event is parsable: {}", event_name);
}

fn assert_is_empty(bytes: &[u8]) {
    if !bytes.is_empty() {
        panic!("bytes not empty: {:?}", bytes);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    match command.as_str() {
        "to-dictionary-item-key" => {
            let index: String = args[2].parse().unwrap();
            let result = to_dictionary_item_key(&index);
            println!("{}", result);
        },

        "bytes-to-address" => {
            let bytes: Vec<u8> = hex::decode(&args[2]).unwrap();
            let (raw_bytes, bytes): (Bytes, _) = FromBytes::from_bytes(&bytes).unwrap();
            assert_is_empty(bytes);
            let (clvalue, bytes) = CLValue::from_bytes(&raw_bytes).unwrap();
            assert_is_empty(bytes);
            let odra_address: OdraAddress = clvalue.into_t().unwrap();
            let casper_address = CasperAddress::try_from(odra_address).unwrap();
            println!("{:?}", casper_address);
        }

        "to-key" => {
            let uref: URef = URef::from_formatted_str(&args[2]).unwrap();
            let index: u32 = args[3].parse().unwrap();
            let dict_item_key = to_dictionary_item_key(&index);
            let key = Key::dictionary(uref, dict_item_key.as_bytes());
            println!("{}", key.to_formatted_string());
        },

        "parse-full-event-bytes" => {
            let bytes: Vec<u8> = hex::decode(&args[2]).unwrap();
            let (cl_value, bytes): (CLValue, _) = FromBytes::from_bytes(&bytes).unwrap();
            let (uref_addr, bytes): (Bytes, _) = FromBytes::from_bytes(bytes).unwrap();
            let (_key_bytes, bytes): (Bytes, _) = FromBytes::from_bytes(bytes).unwrap();
            assert_is_empty(bytes);

            let uref_addr: URefAddr = uref_addr.inner_bytes().clone().try_into().unwrap();
            let uref = URef::new(uref_addr, AccessRights::READ_ADD_WRITE);
            println!("Events dictionary seed: {:?}", uref.to_formatted_string());
            parser_event(cl_value);
        }

        _ => panic!("Unknown command: {}", command)
    }
}
