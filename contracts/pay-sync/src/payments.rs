multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const RECURRENT_PERIOD: u64 = 30 * 24 * 60 * 60;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Debug, PartialEq)]
pub struct Payment<T: ManagedTypeApi> {
    pub recipient_id: u64,
    pub amount: BigUint<T>,
    pub scheduled_time: u64,
    pub is_monthly: bool
}

impl <T: ManagedTypeApi> Payment<T> {
    pub fn update_recurrence(&mut self) {
        if !self.is_monthly {
            return;
        }
        self.scheduled_time += RECURRENT_PERIOD;
    }
}
