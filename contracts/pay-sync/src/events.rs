multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::payments::Payment;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Debug, PartialEq)]
pub struct ProcessedPaymentData<T: ManagedTypeApi> {
    pub amount: BigUint<T>,
    pub scheduled_time: u64,
    pub is_monthly: bool
}

#[multiversx_sc::module]
pub trait MoneyEvents {

    fn add_processed_payment_event(&self, recipient_id: u64, data: &Payment<Self::Api>) {
        self.payment_processed_event(
            recipient_id,
            self.blockchain().get_caller(),
            ProcessedPaymentData {
                amount: data.amount.clone(),
                scheduled_time: data.scheduled_time,
                is_monthly: data.is_monthly
            }
        );
    }

    #[event("top_up")]
    fn topup_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] amount: BigUint,
        #[indexed] token: EgldOrEsdtTokenIdentifier<Self::Api>
    );

    #[event("payment_added")]
    fn payment_added_event(
        &self,
        #[indexed] recipient_id: u64,
        #[indexed] handler: ManagedAddress,
        data: Payment<Self::Api>
    );

    #[event("payment_processed")]
    fn payment_processed_event(
        &self,
        #[indexed] recipient_id: u64,
        #[indexed] processor: ManagedAddress,
        data: ProcessedPaymentData<Self::Api>
    );

}
