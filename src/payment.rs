use odra::{
    execution_error,
    types::{event::Event, Address},
    ContractEnv, Event, Mapping, Variable,
};

type PaymentCode = [u8; 32];
/// PaymentSignal is an encrypted PaymentCode
type PaymentSignal = [u8; 32];

execution_error! {
    pub enum Error {
        NameAlreadyExists => 1,
        PaymentCodeAlreadyExists => 2,
    }
}

#[odra::module]
pub struct MasterPaymentCode {
    by_name: Mapping<String, Address>,
    by_code: Mapping<PaymentCode, Address>,
}

#[odra::module]
impl MasterPaymentCode {
    pub fn set(&self, key: String, contract_address: Address, code: PaymentCode) {
        if self.by_name.get(&key).is_some() {
            ContractEnv::revert(Error::NameAlreadyExists);
        }
        if self.by_code.get(&code).is_some() {
            ContractEnv::revert(Error::PaymentCodeAlreadyExists);
        }
        self.by_name.set(&key, contract_address);
        self.by_code.set(&code, contract_address);

        MasterPaymentCodeSet {
            key,
            contract_address,
            code,
        }
        .emit();
    }

    pub fn get_address_from_name(&self, key: String) -> Option<Address> {
        self.by_name.get(&key)
    }

    pub fn get_address_from_payment_code(&self, code: PaymentCode) -> Option<Address> {
        self.by_code.get(&code)
    }
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct MasterPaymentCodeSet {
    pub key: String,
    pub contract_address: Address,
    pub code: PaymentCode,
}

#[odra::module]
pub struct PersonalPaymentCodeSignalling {
    payment_signals: Mapping<u32, PaymentSignal>,
    payment_signals_index: Variable<u32>,
}

#[odra::module]
impl PersonalPaymentCodeSignalling {
    pub fn post(&self, signal: PaymentSignal) {
        let index = self.payment_signals_index.get_or_default();
        self.payment_signals.set(&index, signal);
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

#[cfg(test)]
mod tests {
    use odra::{assert_events, TestEnv};

    use super::{
        Error, MasterPaymentCode, MasterPaymentCodeSet, PersonalPaymentCodeSignalling,
        PersonalPaymentCodeSignallingPost, PersonalPaymentCodeSignallingRef,
    };

    #[test]
    fn test_simple_scenario() {
        let (ali, bob) = (TestEnv::get_account(0), TestEnv::get_account(1));

        let master_contract = MasterPaymentCode::deploy();
        let ali_contract = PersonalPaymentCodeSignalling::deploy();
        let bob_contract = PersonalPaymentCodeSignalling::deploy();

        // Register Ali contract.
        let ali_key = String::from("ali");
        let ali_code = [22u8; 32];
        let ali_signal = [24u8; 32];
        master_contract.set(ali_key.clone(), ali_contract.address(), ali_code);

        assert_events!(
            master_contract,
            MasterPaymentCodeSet {
                key: ali_key.clone(),
                contract_address: ali_contract.address(),
                code: ali_code
            }
        );

        let chuck_code = [24u8; 32];
        // Register Ali again and that fails.
        // Fail on duplicated unique name
        TestEnv::assert_exception(Error::NameAlreadyExists, || {
            master_contract.set(ali_key.clone(), ali_contract.address(), chuck_code)
        });

        // Fail on duplicated payment code
        TestEnv::assert_exception(Error::PaymentCodeAlreadyExists, || {
            master_contract.set(String::from("chuck"), ali_contract.address(), ali_code)
        });

        // Register Bob.
        let bob_key = String::from("bob");
        let bob_code = [23u8; 32];
        let bob_signal = [25u8; 32];
        master_contract.set(bob_key.clone(), bob_contract.address(), bob_code);

        assert_events!(
            master_contract,
            MasterPaymentCodeSet {
                key: bob_key.clone(),
                contract_address: bob_contract.address(),
                code: bob_code
            }
        );

        // Ali queries for Bob's address.
        let bob_address = master_contract.get_address_from_name(bob_key).unwrap();

        // Bob do the same.
        let ali_address = master_contract.get_address_from_name(ali_key).unwrap();

        // Ali sends signal to Bob.
        TestEnv::set_caller(&ali);
        PersonalPaymentCodeSignallingRef::at(bob_address).post(ali_signal);
        assert_events!(
            bob_contract,
            PersonalPaymentCodeSignallingPost { signal: ali_signal }
        );

        // Bob do the same.
        TestEnv::set_caller(&bob);
        PersonalPaymentCodeSignallingRef::at(ali_address).post(bob_signal);
        assert_events!(
            ali_contract,
            PersonalPaymentCodeSignallingPost { signal: bob_signal }
        );

        // Checks
        assert_eq!(ali_contract.get_payment_signal(0), Some(bob_signal));
        assert_eq!(ali_contract.get_payment_signal(1), None);
        assert_eq!(bob_contract.get_payment_signal(0), Some(ali_signal));
        assert_eq!(bob_contract.get_payment_signal(1), None);
    }
}
