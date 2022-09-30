use casper_types::{bytesrepr::{FromBytes, Bytes}, CLValue};

use odra_casper_shared::casper_address::CasperAddress;
use odra_types::Address as OdraAddress;

#[no_mangle]
fn hello() -> u32 {
    12
}

#[no_mangle]
fn bytes_to_address(bytes: &[u8]) -> Vec<u8> {
    let (raw_bytes, bytes): (Bytes, _) = FromBytes::from_bytes(&bytes).unwrap();
    assert_is_empty(bytes);
    let (clvalue, bytes) = CLValue::from_bytes(&raw_bytes).unwrap();
    assert_is_empty(bytes);
    let odra_address: OdraAddress = clvalue.into_t().unwrap();
    let casper_address = CasperAddress::try_from(odra_address).unwrap();
    
    let formatted_string = if casper_address.is_contract() {
        casper_address.as_contract_package_hash().unwrap().to_formatted_string()
    } else {
        casper_address.as_account_hash().unwrap().to_formatted_string()
    };
    formatted_string.as_bytes().to_vec()
}

fn assert_is_empty(bytes: &[u8]) {
    if !bytes.is_empty() {
        panic!("bytes not empty: {:?}", bytes);
    }
}