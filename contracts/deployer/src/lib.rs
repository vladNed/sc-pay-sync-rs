#![no_std]

multiversx_sc::imports!();

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait DeployerContract {

    #[init]
    fn init(&self, contract_template: ManagedAddress) {
        self.contract_template().set(contract_template);
    }

    #[endpoint(deployNewPaySync)]
    fn deploy_new_pay_sync(
        &self,
        token: EgldOrEsdtTokenIdentifier,
        money_handlers: OptionalValue<MultiValueEncoded<ManagedAddress>>,
    ) -> ManagedAddress {
        let caller = self.blockchain().get_caller();
        let mut arguments = ManagedArgBuffer::new();
        arguments.push_arg(token);
        match money_handlers {
            OptionalValue::Some(handlers) => {
                for handler in handlers {
                    arguments.push_arg(handler);
                }
            }
            OptionalValue::None => (),
        }
        arguments.push_arg(caller.clone());
        let (pay_sync_address, _) = Self::Api::send_api_impl().deploy_from_source_contract(
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &self.contract_template().get(),
            CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE,
            &arguments,
        );

        self.owner_contracts(&caller).update(|contracts| contracts.push(pay_sync_address.clone()));
        self.all_child_contracts().update(|contracts| contracts.push(pay_sync_address.clone()));

        pay_sync_address
    }

    #[view(getContractTemplate)]
    #[storage_mapper("contract_template")]
    fn contract_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getOwnerContracts)]
    #[storage_mapper("owner_contracts")]
    fn owner_contracts(
        &self,
        owner_address: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

    #[view(getAllChildContracts)]
    #[storage_mapper("all_child_contracts")]
    fn all_child_contracts(&self) -> SingleValueMapper<ManagedVec<ManagedAddress>>;
}
