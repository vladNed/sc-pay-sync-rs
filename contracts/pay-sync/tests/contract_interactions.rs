use multiversx_sc::types::{Address, EgldOrEsdtTokenIdentifier, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use pay_sync::PaySyncContract;

pub const WASM_PATH: &'static str = "../out/pay-sync.wasm";
pub const TKN: &[u8] = b"TKN-s81ks0";

pub struct PaySyncSetup<Builder>
where
    Builder: 'static + Copy + Fn() -> pay_sync::ContractObj<DebugApi>,
{
    pub b_wrapper: BlockchainStateWrapper,
    pub c_wrapper: ContractObjWrapper<pay_sync::ContractObj<DebugApi>, Builder>,
    pub owner: Address,
}

impl<Builder> PaySyncSetup<Builder>
where
    Builder: 'static + Copy + Fn() -> pay_sync::ContractObj<DebugApi>,
{
    pub fn new(builder: Builder) -> Self {
        let mut blockchain_wrapper = BlockchainStateWrapper::new();
        let owner_address = blockchain_wrapper.create_user_account(&rust_biguint!(0));

        let c_wrapper = blockchain_wrapper.create_sc_account(
            &rust_biguint!(0),
            Some(&owner_address),
            builder,
            WASM_PATH,
        );

        blockchain_wrapper
            .execute_tx(&owner_address, &c_wrapper, &rust_biguint!(0), |sc| {
                let mut money_handlers = MultiValueEncoded::new();
                let mut money_processor = MultiValueEncoded::new();
                money_processor.push(managed_address!(&owner_address));
                money_handlers.push(managed_address!(&owner_address));

                sc.init(
                    EgldOrEsdtTokenIdentifier::esdt(TKN),
                    money_handlers,
                    money_processor,
                );
            })
            .assert_ok();

        Self {
            b_wrapper: blockchain_wrapper,
            c_wrapper,
            owner: owner_address,
        }
    }

    pub fn top_up(&mut self, amount: u64, expected_err: Option<&str>) {
        let tx = self.b_wrapper.execute_esdt_transfer(
            &self.owner,
            &self.c_wrapper,
            TKN,
            0,
            &rust_biguint!(amount),
            |sc| {
                sc.top_up();
            },
        );

        match expected_err {
            Some(err) => tx.assert_error(4, err),
            None => tx.assert_ok(),
        }
    }

    pub fn check_contract_balance(&self, expected_balance: u64) {
        self.b_wrapper.check_esdt_balance(
            &self.c_wrapper.address_ref(),
            TKN,
            &rust_biguint!(expected_balance),
        );
    }

    pub fn set_balance_owner(&mut self, amount: u64) {
        self.b_wrapper
            .set_esdt_balance(&self.owner, TKN, &rust_biguint!(amount));
    }

    pub fn add_payment(
        &mut self,
        recipient: &Address,
        amount: u64,
        scheduled_time: u64,
        is_monthly: bool,
    ) {
        self.b_wrapper
            .execute_tx(&self.owner, &self.c_wrapper, &rust_biguint!(0), |sc| {
                sc.add_payment(
                    managed_address!(&recipient),
                    managed_biguint!(amount),
                    scheduled_time,
                    is_monthly,
                );
            })
            .assert_ok();
    }

    pub fn check_recipients(&mut self, expected: u64) {
        self.b_wrapper.execute_query(&self.c_wrapper, |sc| {
            let recipients = sc.recipient_id_index().get();
            assert_eq!(recipients, expected);
        }).assert_ok();
    }

    pub fn check_payments(&mut self, expected: u64) {
        self.b_wrapper.execute_query(&self.c_wrapper, |sc| {
            let payments = sc.all_payments().len();
            assert_eq!(payments, expected as usize);
        }).assert_ok();
    }

    pub fn process_payments(&mut self, payments_ids: Vec<u64>, expected_err: Option<&str>) {
        let tx = self.b_wrapper
            .execute_tx(&self.owner, &self.c_wrapper, &rust_biguint!(0), |sc| {
                let mut values = MultiValueEncoded::new();
                for id in payments_ids {
                    values.push(id);
                }
                sc.process_payments(values);
            });

        match expected_err {
            Some(err) => tx.assert_error(4, err),
            None => tx.assert_ok(),
        }
    }
}
