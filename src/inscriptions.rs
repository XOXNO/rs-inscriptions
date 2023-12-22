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
    fn inscription(&self, payload: ManagedBuffer, name: &ManagedBuffer, royalties: &BigUint) {
        self.token_manager().require_issued_or_set();
        require!(
            royalties <= &BigUint::from(ROYALTIES_MAX),
            "Royalties must be between 0 and 10000"
        );
        let hash_bytes =      self.crypto().sha256(payload);
        let hash_hex = hash_bytes.as_managed_buffer();

        let tx_hash_bytes = self.blockchain().get_tx_hash();
        let tx_hash_hex = tx_hash_bytes.as_managed_buffer();
        let map_used_hashes = self.used_hashes(hash_hex);
        require!(
            !map_used_hashes.is_empty(),
            "Hash already used in a previous inscription"
        );
        map_used_hashes.set(true);
        let ticker_map = self.token_manager();
        let amount = &BigUint::from(NFT_AMOUNT);
        let mut uris = ManagedVec::new();
        uris.push(tx_hash_hex.clone());
        let caller = self.blockchain().get_caller();
        let nonce = self.send().esdt_nft_create(
            ticker_map.get_token_id_ref(),
            amount,
            name,
            royalties,
            hash_hex,
            caller.as_managed_buffer(),
            &uris,
        );
        self.send().direct_esdt(&self.blockchain().get_caller(), ticker_map.get_token_id_ref(), nonce, amount);
        self.emit_mint_inscription(nonce,ticker_map.get_token_id_ref(), tx_hash_hex, hash_hex)
    }

    #[payable("EGLD")]
    #[endpoint(issue)]
    fn issue_token(&self, name: ManagedBuffer, ticker: ManagedBuffer) {
        let manager = self.token_manager();
        require!(manager.is_empty(), "Token already issued");
        let amount = self.call_value().egld_value();
        require!(
            amount.clone_value() == BigUint::from(NFT_ISSUE_COST),
            "Insufficient funds");
        self.token_manager().issue(EsdtTokenType::NonFungible, amount.clone_value(), name, ticker, 0, None);
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
        #[indexed] tx_hash: &ManagedBuffer<Self::Api>, // used for off chain indexing of the inscription payload
        #[indexed] hash: &ManagedBuffer<Self::Api>,
    );
}
