#![no_std]

use payments::Payment;

pub mod events;
pub mod payments;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait PaySyncContract: events::MoneyEvents {
    #[init]
    fn init(
        &self,
        accepted_token: EgldOrEsdtTokenIdentifier,
        money_handlers: MultiValueEncoded<ManagedAddress>,
        money_processor: MultiValueEncoded<ManagedAddress>,
    ) {
        for handler in money_handlers {
            self.money_handlers().add(&handler);
        }
        for processor in money_processor {
            self.money_processor().add(&processor);
        }
        self.payments_index().set_if_empty(1);
        self.recipient_id_index().set_if_empty(1);
        self.accepted_token().set_if_empty(accepted_token);
    }

    /// Adds money to the contract by a money handler
    /// Triggers a topup event
    #[payable("*")]
    #[endpoint(topUp)]
    fn top_up(&self) {
        self.require_money_handler();
        let payment = self.call_value().egld_or_single_esdt();
        require!(payment.token_identifier == self.accepted_token().get(), "Invalid token");
        self.topup_event(
            self.blockchain().get_caller(),
            payment.amount,
            payment.token_identifier
        );
    }

    /// Adds a new payment to the contract to be processed by a money processor
    /// at a scheduled time.
    ///
    /// Adds a new recipient if it doesn't exist, if it does exist, it uses the existing one.
    /// This is done to avoid having to store the recipient's address in the contract.
    #[endpoint(addPayment)]
    fn add_payment(
        &self,
        recipient: ManagedAddress,
        amount: BigUint<Self::Api>,
        scheduled_time: u64,
        is_monthly: bool
    ) {
        self.require_money_handler();
        let recipient_id = self.get_or_add_recipient_id(recipient);
        let payment = Payment {
            recipient_id,
            amount,
            scheduled_time,
            is_monthly
        };

        let payments_index_handler = self.payments_index();
        let new_payment_id = payments_index_handler.get();
        self.payments(new_payment_id).set(payment.clone());
        self.all_payments().insert(new_payment_id);
        payments_index_handler.update(|x| *x += 1);
        self.payment_added_event(recipient_id, self.blockchain().get_caller(), payment);
    }

    /// Processes all payments that are scheduled to be processed at the current time.
    /// If a payment is monthly, it updates the scheduled time for the next month.
    /// If a payment is not monthly, it removes it from the contract.
    ///
    /// This endpoint is meant to be called by a money processor.
    #[endpoint(processPayments)]
    fn process_payments(&self, payments_ids: MultiValueEncoded<u64>) {
        self.require_money_processor();
        let token = self.accepted_token().get();
        let now = self.blockchain().get_block_timestamp();
        let mut all_payments_handler = self.all_payments();

        let mut current_tkn_balance = self.blockchain().get_sc_balance(&token, 0);
        require!(current_tkn_balance > 0, "No funds allocated to contract");
        for payment_id in payments_ids {
            let payment_handler = self.payments(payment_id);
            let mut payment = payment_handler.get();
            if payment.scheduled_time < now {
                continue;
            }
            require!(current_tkn_balance >= payment.amount, "Not enough funds to process payments");
            let recipient = self.recipients(payment.recipient_id).get();
            self.send().direct(&recipient, &token, 0, &payment.amount);
            if payment.is_monthly {
                payment.update_recurrence();
                payment_handler.set(payment.clone());
            } else {
                all_payments_handler.swap_remove(&payment_id);
                payment_handler.clear();
            }

            current_tkn_balance -= &payment.amount;
            self.add_processed_payment_event(payment_id, &payment);
        }
    }

    #[endpoint(addMoneyHandler)]
    fn add_money_handler(&self, handler: ManagedAddress) {
        self.require_money_handler();
        self.money_handlers().add(&handler);
    }

    #[endpoint(removeMoneyHandler)]
    fn remove_money_handler(&self, handler: ManagedAddress) {
        self.require_money_handler();
        self.money_handlers().remove(&handler);
    }

    #[endpoint(addMoneyProcessor)]
    fn add_money_processor(&self, processor: ManagedAddress) {
        self.require_money_handler();
        self.money_processor().add(&processor);
    }

    #[endpoint(removeMoneyProcessor)]
    fn remove_money_processor(&self, processor: ManagedAddress) {
        self.require_money_handler();
        self.money_processor().remove(&processor);
    }

    fn require_money_handler(&self) {
        let caller = self.blockchain().get_caller();
        let is_money_handler = self.money_handlers().contains(&caller);
        let is_contract_owner = self.blockchain().get_owner_address() == caller;

        require!(is_money_handler || is_contract_owner, "Caller is not a money handler");
    }

    fn require_money_processor(&self) {
        let caller = self.blockchain().get_caller();
        let is_money_processor = self.money_processor().contains(&caller);
        let is_contract_owner = self.blockchain().get_owner_address() == caller;

        require!(is_money_processor || is_contract_owner, "Caller is not a money processor");
    }

    fn get_or_add_recipient_id(&self, address: ManagedAddress) -> u64 {
        let recipient_to_id_handler = self.recipient_to_id(&address);
        let recipient_id = recipient_to_id_handler.get();
        if recipient_id != 0 {
            return recipient_id;
        }

        let recipient_id_index_handler = self.recipient_id_index();
        let new_recipient_id = recipient_id_index_handler.get();
        self.recipients_ids().add(&new_recipient_id);
        self.recipients(new_recipient_id).set(address.clone());
        recipient_to_id_handler.set(new_recipient_id);
        recipient_id_index_handler.update(|x| *x += 1);

        new_recipient_id
    }

    #[storage_mapper("money_handlers")]
    fn money_handlers(&self) -> WhitelistMapper<ManagedAddress>;

    #[storage_mapper("money_processor")]
    fn money_processor(&self) -> WhitelistMapper<ManagedAddress>;

    #[storage_mapper("accepted_token")]
    fn accepted_token(&self) -> SingleValueMapper<EgldOrEsdtTokenIdentifier>;

    #[storage_mapper("payments_index")]
    fn payments_index(&self) -> SingleValueMapper<u64>;

    #[view(getAllPaymentsIds)]
    #[storage_mapper("all_payments")]
    fn all_payments(&self) -> UnorderedSetMapper<u64>;

    #[view(getPaymentById)]
    #[storage_mapper("payments")]
    fn payments(&self, id: u64) -> SingleValueMapper<Payment<Self::Api>>;

    #[storage_mapper("recipient_id_index")]
    fn recipient_id_index(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("recipients")]
    fn recipients(&self, id: u64) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("recipients_ids")]
    fn recipients_ids(&self) -> WhitelistMapper<u64>;

    #[storage_mapper("recipient_to_id")]
    fn recipient_to_id(&self, recipient: &ManagedAddress) -> SingleValueMapper<u64>;
}
