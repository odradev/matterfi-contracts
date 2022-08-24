use odra::{
    execution_error,
    types::{event::Event, Address, NamedArg},
    ContractEnv, Event, Mapping, Variable,
};

type PaymentCode = Vec<u8>; //Bytes
/// PaymentSignal is an encrypted PaymentCode
type PaymentSignal = Vec<u8>;

type Name = String;
type PersonalContractAddress = Address;

execution_error! {
    pub enum Error {
        NameAlreadyExists => 1,
        PaymentCodeAlreadyExists => 2,
        IncorrectPaymentCodeLength => 3,
    }
}

#[odra::module]
pub struct MasterPaymentCode {
    name_to_contract_address: Mapping<Name, PersonalContractAddress>,
    name_to_plain_payment_code: Mapping<Name, PaymentCode>,
}

#[odra::module]
impl MasterPaymentCode {
    pub fn set(&self, key: Name, plain_payment_code: String) {
        
        if let Ok(converted_value) =  hex::decode(&plain_payment_code) {
            self.set_impl(key, ContractEnv::caller(), converted_value);
        } else {
            ContractEnv::revert(Error::IncorrectPaymentCodeLength);
        }
    }
    // pub fn set(&self, key: String, value: String) {
        
    //     if let Ok(converted_value) = hex::decode(&value) {
    //         self.set_impl(key, ContractEnv::caller(), converted_value);
    //     } else {
    //         ContractEnv::revert(Error::IncorrectPaymentCodeLength);
    //     }
    // }

    pub fn set_impl(&self, key: Name, contract_address: Address, plain_payment_code: PaymentCode) {
        if self.name_to_contract_address.get(&key).is_some() {
            ContractEnv::revert(Error::NameAlreadyExists);
        }
        if self.name_to_plain_payment_code.get(&key).is_some() {
            ContractEnv::revert(Error::PaymentCodeAlreadyExists);
        }
        self.name_to_contract_address.set(&key, contract_address);
        self.name_to_plain_payment_code.set(&key, plain_payment_code.clone());

        // MasterPaymentCodeSet {
        //     key,
        //     contract_address,
        //     plain_payment_code,
        // }
        // .emit();
    }

    pub fn get_contract_from_name(&self, key: Name) -> Option<Address> {
        self.name_to_contract_address.get(&key)
    }

    pub fn get_payment_code_from_name(&self, key: Name) -> Option<String> {
        let encoded_payment_code = self.name_to_plain_payment_code.get(&key);
        let plain_payment_code = Some(hex::encode(encoded_payment_code.unwrap()));
        return plain_payment_code;

    }
}

// #[derive(Event, PartialEq, Eq, Debug)]
// pub struct MasterPaymentCodeSet {
//     pub key: Name,
//     pub contract_address: PersonalContractAddress,
//     pub plain_payment_code: PaymentCode,
// }

#[odra::module]
pub struct PersonalPaymentCodeSignalling {
    payment_signals: Mapping<u32, PaymentSignal>,
    payment_signals_index: Variable<u32>,
}

#[odra::module]
impl PersonalPaymentCodeSignalling {
    pub fn post(&self, signal: PaymentSignal) {
        let index = self.payment_signals_index.get_or_default();
        self.payment_signals.set(&index, signal.clone());
        self.payment_signals_index.set(index + 1);

        PersonalPaymentCodeSignallingPost { signal }.emit();
    }

    pub fn get_payment_signal(&self, index: u32) -> Option<PaymentSignal> {
        self.payment_signals.get(&index)
    }
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct PersonalPaymentCodeSignallingPost {
    pub signal: PaymentSignal,
}

// #[cfg(test)]
// mod tests {
//     use odra::{assert_events, TestEnv};

//     use super::{
//         Error, MasterPaymentCode, MasterPaymentCodeSet, PersonalPaymentCodeSignalling,
//         PersonalPaymentCodeSignallingPost, PersonalPaymentCodeSignallingRef,
//     };

//     #[test]
//     fn test_simple_scenario() {
//         let (ali, bob) = (TestEnv::get_account(0), TestEnv::get_account(1));

//         let master_contract = MasterPaymentCode::deploy();
//         let ali_contract = PersonalPaymentCodeSignalling::deploy();
//         let bob_contract = PersonalPaymentCodeSignalling::deploy();

//         // Register Ali contract.
//         let ali_key = String::from("ali");
//         let ali_code = vec![22u8; 39];
//         let ali_signal = vec![24u8; 32];
//         master_contract.set_impl(ali_key.clone(), ali_contract.address(), ali_code.clone());

//         assert_events!(
//             master_contract,
//             MasterPaymentCodeSet {
//                 key: ali_key.clone(),
//                 contract_address: ali_contract.address(),
//                 code: ali_code.clone()
//             }
//         );

//         let chuck_code = vec![24u8; 39];
//         // Register Ali again and that fails.
//         // Fail on duplicated unique name
//         TestEnv::assert_exception(Error::NameAlreadyExists, || {
//             master_contract.set_impl(ali_key.clone(), ali_contract.address(), chuck_code.clone())
//         });

//         // Fail on duplicated payment code
//         TestEnv::assert_exception(Error::PaymentCodeAlreadyExists, || {
//             master_contract.set_impl(String::from("chuck"), ali_contract.address(), ali_code.clone())
//         });

//         // Register Bob.
//         let bob_key = String::from("bob");
//         let bob_code = vec![23u8; 39];
//         let bob_signal = vec![25u8; 32];
//         master_contract.set_impl(bob_key.clone(), bob_contract.address(), bob_code.clone());

//         assert_events!(
//             master_contract,
//             MasterPaymentCodeSet {
//                 key: bob_key.clone(),
//                 contract_address: bob_contract.address(),
//                 code: bob_code.clone()
//             }
//         );

//         // Ali queries for Bob's address.
//         let bob_address = master_contract.get_address_from_name(bob_key).unwrap();

//         // Bob do the same.
//         let ali_address = master_contract.get_address_from_name(ali_key).unwrap();

//         // Ali sends signal to Bob.
//         TestEnv::set_caller(&ali);
//         PersonalPaymentCodeSignallingRef::at(bob_address).post(ali_signal.clone());
//         assert_events!(
//             bob_contract,
//             PersonalPaymentCodeSignallingPost { signal: ali_signal.clone() }
//         );

//         // Bob do the same.
//         TestEnv::set_caller(&bob);
//         PersonalPaymentCodeSignallingRef::at(ali_address).post(bob_signal.clone());
//         assert_events!(
//             ali_contract,
//             PersonalPaymentCodeSignallingPost { signal: bob_signal.clone() }
//         );

//         // Checks
//         assert_eq!(ali_contract.get_payment_signal(0), Some(bob_signal));
//         assert_eq!(ali_contract.get_payment_signal(1), None);
//         assert_eq!(bob_contract.get_payment_signal(0), Some(ali_signal));
//         assert_eq!(bob_contract.get_payment_signal(1), None);
//     }
// }
