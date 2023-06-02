mod contract_interactions;

use contract_interactions::*;
use multiversx_sc_scenario::rust_biguint;

#[test]
fn test_top_up() {
    let mut setup = PaySyncSetup::new(pay_sync::contract_obj);
    setup.set_balance_owner(300_000u64);
    setup.top_up(200_000u64, None);
    setup.check_contract_balance(200_000u64);
}

#[test]
fn test_add_payment_new_recipient() {
    let mut setup = PaySyncSetup::new(pay_sync::contract_obj);
    let new_recipient = setup.b_wrapper.create_user_account(&rust_biguint!(0));

    setup.add_payment(&new_recipient, 100_000, 100, false);
    setup.check_recipients(2);
    setup.add_payment(&new_recipient, 100_000, 100, false);
    setup.check_recipients(2);
    setup.check_payments(2);
}

#[test]
fn test_process_payments() {
    let mut setup = PaySyncSetup::new(pay_sync::contract_obj);
    setup.set_balance_owner(300_000u64);
    setup.top_up(200_000u64, None);

    let new_recipient = setup.b_wrapper.create_user_account(&rust_biguint!(0));

    setup.add_payment(&new_recipient, 100_000, 100, false);
    setup.add_payment(&new_recipient, 100_000, 100, false);

    setup.b_wrapper.set_block_timestamp(11);

    let payments = vec![1, 2];
    setup.process_payments(payments, None);
}

#[test]
fn test_process_payments_with_no_funds() {
    let mut setup = PaySyncSetup::new(pay_sync::contract_obj);
    setup.set_balance_owner(300_000u64);
    setup.top_up(50_000u64, None);

    let new_recipient = setup.b_wrapper.create_user_account(&rust_biguint!(0));

    setup.add_payment(&new_recipient, 100_000, 100, false);

    setup.b_wrapper.set_block_timestamp(11);

    let payments = vec![1, 2];
    setup.process_payments(payments, Some("Not enough funds to process payments"));
}
