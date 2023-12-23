#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const NFT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
const ROYALTIES_MAX: u32 = 10_000;
const NFT_AMOUNT: u32 = 1;

#[multiversx_sc::contract]
pub trait Inscriptions {
    #[upgrade]
    fn upgrade(&self) {}

    #[init]
    fn init(&self) {}

    #[endpoint(inscription)]
    fn inscription(&self, payload: ManagedBuffer, name: &ManagedBuffer, royalties: &BigUint, create_for: &OptionalValue<ManagedAddress>) -> EsdtTokenPayment {
        self.token_manager().require_issued_or_set();

        // Check the royalties are between 0 and 10000 (0% - 100%)
        require!(
            royalties <= &BigUint::from(ROYALTIES_MAX),
            "Royalties must be between 0 and 10000!"
        );

        // Hash the payload to create a unique identifier for the inscription
        let hash_bytes = self.crypto().sha256(payload); // max payload size around 320kb
        let hash_hex = hash_bytes.as_managed_buffer();

        // Fetch the tx hash of the inscription mint transaction
        let tx_hash_bytes = self.blockchain().get_tx_hash();
        let tx_hash_hex = tx_hash_bytes.as_managed_buffer();
    
        // Check if hash already used in another inscription
        let map_used_hashes = self.used_hashes(hash_hex);
        require!(map_used_hashes.is_empty(),
            "Hash already used in a previous inscription!"
        );

        // Mark hash as used
        map_used_hashes.set(true);

        let ticker_map = self.token_manager();
        let amount = &BigUint::from(NFT_AMOUNT);
        let mut uris = ManagedVec::new();
        uris.push(tx_hash_hex.clone());
        let caller = self.blockchain().get_caller();

        // Create NFT and point the NFT to the tx hash via the URI
        // The URI is used to fetch the inscription payload off chain
        // The attributes are used to store the creator of the NFT will require a implementation in marketplaces
        let nonce = self.send().esdt_nft_create::<Attributes<Self::Api>>(
            ticker_map.get_token_id_ref(),
            amount,
            name,
            royalties,
            hash_hex,
            &Attributes {
                creator: caller.clone(),
            },
            &uris,
        );

        let to = match create_for {
            OptionalValue::Some(address) => address,
            OptionalValue::None => &caller,
        };

        // Send the inscription to the caller
        self.send().direct_esdt(to, ticker_map.get_token_id_ref(), nonce, amount);

        // Emit the inscription event to use it off chain
        self.emit_mint_inscription(nonce, ticker_map.get_token_id_ref(), &caller, &to, tx_hash_hex, hash_hex);

        // Return the inscription NFT as a response, it can be used inside other contracts
        EsdtTokenPayment::new(ticker_map.get_token_id(), nonce, amount.clone())
    }

    #[payable("EGLD")]
    #[endpoint(issue)]
    fn issue_token(&self, name: ManagedBuffer, ticker: ManagedBuffer) {
        self.blockchain().check_caller_is_owner();
        let manager = self.token_manager();
        require!(manager.is_empty(), "Token already issued!");
        let amount = self.call_value().egld_value();
        require!(
            amount.clone_value() == BigUint::from(NFT_ISSUE_COST),
            "Insufficient funds!");
        self.token_manager().issue_and_set_all_roles(EsdtTokenType::NonFungible, amount.clone_value(), name, ticker, 0, Some(self.callbacks().issue_callback()));
    }


    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                    self.token_manager().set_token_id(token_id);
            }
            ManagedAsyncCallResult::Err(_) => { }
        }
    }

    #[storage_mapper("ticker")]
    fn token_manager(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[storage_mapper("uniqueHashes")]
    fn used_hashes(&self, hash: &ManagedBuffer) -> SingleValueMapper<bool>;

    #[event("emit_mint_inscription")]
    fn emit_mint_inscription(
        &self,
        #[indexed] nonce: u64,
        #[indexed] token: &TokenIdentifier<Self::Api>,
        #[indexed] creator: &ManagedAddress<Self::Api>,
        #[indexed] receiver: &ManagedAddress<Self::Api>,
        #[indexed] tx_hash: &ManagedBuffer<Self::Api>,
        #[indexed] hash: &ManagedBuffer<Self::Api>,
    );
}


#[derive(ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Attributes<M: ManagedTypeApi> {
    pub creator: ManagedAddress<M>,
}