/// A minimal Move module demonstrating struct, public fun, entry fun, and use declarations.
module 0x1::Coin {
    use std::signer;
    use 0x1::Event::EventHandle;

    /// Represents a coin with a value.
    struct Coin has key, store {
        value: u64,
    }

    /// A treasury capability for minting.
    struct MintCapability has key {}

    const MAX_SUPPLY: u64 = 1000000000;

    /// Initialize the Coin module.
    public fun initialize(account: &signer): MintCapability {
        move_to(account, MintCapability {});
        MintCapability {}
    }

    /// Mint a new coin with the given value.
    public fun mint(_cap: &MintCapability, value: u64): Coin {
        assert!(value <= MAX_SUPPLY, 1);
        Coin { value }
    }

    /// Get the value of a coin.
    public fun value(coin: &Coin): u64 {
        coin.value
    }

    /// Transfer a coin to a recipient.
    entry fun transfer(sender: &signer, recipient: address, value: u64)
    acquires Coin {
        let coin = borrow_global_mut<Coin>(signer::address_of(sender));
        let transferred = Coin { value };
        coin.value = coin.value - value;
        move_to(&recipient, transferred);
    }

    /// Destroy a coin and return its value.
    public fun destroy(coin: Coin): u64 {
        let Coin { value } = coin;
        value
    }
}
