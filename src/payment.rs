use odra::{execution_error, ContractEnv, Event, Mapping, Variable};

type PaymentCode = Vec<u8>; //Bytes
/// PaymentSignal is an encrypted PaymentCode
type PaymentSignal = Vec<u8>;

type Name = String;

execution_error! {
    pub enum Error {
        NameAlreadyExists => 1,
        PaymentCodeAlreadyExists => 2,
        PaymentCodeDoesntExits => 3,
        IncorrectPaymentCodeLength => 4,
        IncorrectPaymentSignalLength => 5,
    }
}

#[odra::module]
pub struct MasterPaymentCode {
    name_to_plain_payment_code: Mapping<Name, PaymentCode>,

    // payment_signals: Mapping<PaymentCode, (PaymentSignal, u32)>,
    // payment_signals_index: Mapping<PaymentCode, Variable<u32>>,
    payment_signals_mapping: Mapping<PaymentCode, Vec<u32>>,
    payment_signals: Mapping<u32, PaymentSignal>,
    payment_signals_index: Variable<u32>,
    // paymens_signals = [ (0, "AliceToMichael"), (1, "BobToAlice"), (2, "BobToGeorge"), (3, "BobToAlpha"), (4, "GeorgeToAlpha") ]
    // payment_signals_index = 4
    // payment_signals_mapping = [[Alice, [0, 1]],[Bob, [1, 2, 3]], [Michael, [0]]
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct PersonalPaymentCodeSignallingPost {
    pub signal: PaymentSignal,
}

pub fn as_u8_slice(v: &[u32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            v.as_ptr() as *const u8,
            v.len() * std::mem::size_of::<u32>(),
        )
    }
}

#[odra::module]
impl MasterPaymentCode {
    pub fn get_codes(&self, plain_payment_code: String) -> String {
        if let Ok(converted_value) = bs58::decode(&plain_payment_code)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
        {
            return bs58::encode(as_u8_slice(
                &self.payment_signals_mapping.get(&converted_value).unwrap(),
            ))
            .into_string();
        }
        String::new()
    }

    pub fn set_user(&self, key: Name, plain_payment_code: String) {
        if self.name_to_plain_payment_code.get(&key).is_some() {
            ContractEnv::revert(Error::NameAlreadyExists);
        }
        if let Ok(converted_value) = bs58::decode(&plain_payment_code)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
        {
            self.name_to_plain_payment_code
                .set(&key, converted_value.clone());

            if self.payment_signals_mapping.get(&converted_value).is_none() {
                self.payment_signals_mapping
                    .set(&converted_value, Vec::new());
            }
        } else {
            ContractEnv::revert(Error::IncorrectPaymentCodeLength);
        }
    }

    pub fn set_pcode(&self, plain_payment_code: String) {
        if let Ok(converted_value) = bs58::decode(&plain_payment_code)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
        {
            if self.payment_signals_mapping.get(&converted_value).is_some() {
                ContractEnv::revert(Error::PaymentCodeAlreadyExists);
            }
            self.payment_signals_mapping
                .set(&converted_value, Vec::new());
        } else {
            ContractEnv::revert(Error::IncorrectPaymentCodeLength);
        }
    }

    pub fn set_signal(&self, recipient_payment_code: String, masked_payment_code: String) {
        if let Ok(converted_recipient_payment_code) = bs58::decode(&recipient_payment_code)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
        {
            if self
                .payment_signals_mapping
                .get(&converted_recipient_payment_code)
                .is_none()
            {
                ContractEnv::revert(Error::PaymentCodeDoesntExits);
            }

            if let Some(mut payment_code_signals_vec) = self
                .payment_signals_mapping
                .get(&converted_recipient_payment_code)
            {
                let index = self.payment_signals_index.get_or_default();

                if let Ok(converted_masked_payment_code) = bs58::decode(&masked_payment_code)
                    .with_alphabet(bs58::Alphabet::BITCOIN)
                    .into_vec()
                {
                    self.payment_signals
                        .set(&index, converted_masked_payment_code);

                    payment_code_signals_vec.push(index);
                    self.payment_signals_mapping
                        .set(&converted_recipient_payment_code, payment_code_signals_vec);

                    self.payment_signals_index.set(index + 1);
                } else {
                    ContractEnv::revert(Error::IncorrectPaymentSignalLength);
                }
            }
        } else {
            ContractEnv::revert(Error::IncorrectPaymentCodeLength);
        }

        //PersonalPaymentCodeSignallingPost { signal }.emit();
    }

    // pub fn get_payment_code_from_name(&self, key: Name) -> Option<String> {
    //     let encoded_payment_code = self.name_to_plain_payment_code.get(&key);
    //     Some(hex::encode(encoded_payment_code.unwrap()))
    // }
    //
    // pub fn get_payment_signal(&self, index: u32) -> Option<PaymentSignal> {
    //     self.payment_signals.get(&index)
    // }
}

// #[derive(Event, PartialEq, Eq, Debug)]
// pub struct MasterPaymentCodeSet {
//     pub key: Name,
//     pub contract_address: PersonalContractAddress,
//     pub plain_payment_code: PaymentCode,
// }

// #[odra::module]
// pub struct PersonalPaymentCodeSignalling {
//     payment_signals: Mapping<u32, PaymentSignal>,
//     payment_signals_index: Variable<u32>,
// }

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

// #[cfg(test)]
// mod tests {
//     use odra::{assert_events, TestEnv};

//     use super::{Error, MasterPaymentCode};

//     #[test]
//     fn test_simple_scenario() {
//         let (ali, bob) = (TestEnv::get_account(0), TestEnv::get_account(1));

//         let master_contract = MasterPaymentCode::deploy();

//         // Register Ali contract.
//         let ali_key = String::from("ali");
//         let ali_code = String::from("aaaaaaaa");
//         let ali_signal = String::from("aaaaaaaaaaaaaa");

//         let bob_key = String::from("bob");
//         let bob_code = String::from("bbbbbbbbbbb");
//         let bob_signal = String::from("bbbbbbbbffffff");

//         let rado_key = String::from("bob");
//         let rado_code = String::from("rrrrrrrrrr");
//         let rado_signal = String::from("rrrrrrrrrrffffff");
//         master_contract.set_user(ali_key.clone(), ali_code.clone());
//         master_contract.set_signal(ali_code.clone(), bob_signal.clone());

//         // Ali queries for Bob's address.
//         let bob_address = master_contract.get_address_from_name(bob_key).unwrap();

//         // Bob do the same.
//         let ali_address = master_contract.get_address_from_name(ali_key).unwrap();

//         // Ali sends signal to Bob.
//         TestEnv::set_caller(&ali);
//         PersonalPaymentCodeSignallingRef::at(bob_address).post(ali_signal.clone());
//         assert_events!(
//             bob_contract,
//             PersonalPaymentCodeSignallingPost {
//                 signal: ali_signal.clone()
//             }
//         );

//         // Bob do the same.
//         TestEnv::set_caller(&bob);
//         PersonalPaymentCodeSignallingRef::at(ali_address).post(bob_signal.clone());
//         assert_events!(
//             ali_contract,
//             PersonalPaymentCodeSignallingPost {
//                 signal: bob_signal.clone()
//             }
//         );

//         // Checks
//         assert_eq!(ali_contract.get_payment_signal(0), Some(bob_signal));
//         assert_eq!(ali_contract.get_payment_signal(1), None);
//         assert_eq!(bob_contract.get_payment_signal(0), Some(ali_signal));
//         assert_eq!(bob_contract.get_payment_signal(1), None);
//     }
// }
