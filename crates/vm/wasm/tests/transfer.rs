use {
    grug_testing::TestBuilder,
    grug_types::{
        Coins, Denom, Message, MultiplyFraction, NumberConst, ResultExt, Udec128, Uint256,
    },
    grug_vm_wasm::WasmVm,
    std::{collections::BTreeMap, str::FromStr, sync::LazyLock, vec},
};

const WASM_CACHE_CAPACITY: usize = 10;

static DENOM: LazyLock<Denom> = LazyLock::new(|| Denom::from_str("ugrug").unwrap());

static FEE_RATE: LazyLock<Udec128> = LazyLock::new(|| Udec128::from_str("0.1").unwrap());

#[test]
fn transfers() -> anyhow::Result<()> {
    let (mut suite, mut accounts) = TestBuilder::new_with_vm(WasmVm::new(WASM_CACHE_CAPACITY))
        .add_account("owner", Coins::new())?
        .add_account("sender", Coins::one(DENOM.clone(), 300_000_u128)?)?
        .add_account("receiver", Coins::new())?
        .set_owner("owner")?
        .set_fee_denom(DENOM.clone())
        .set_fee_rate(*FEE_RATE)
        .build()?;

    let to = accounts["receiver"].address;

    // Check that sender has been given 300,000 ugrug.
    // Sender needs to have sufficient tokens to cover gas fee and the transfers.
    suite
        .query_balance(&accounts["sender"], DENOM.clone())
        .should_succeed_and_equal(Uint256::from(300_000_u128));
    suite
        .query_balance(&accounts["receiver"], DENOM.clone())
        .should_succeed_and_equal(Uint256::ZERO);

    // Sender sends 70 ugrug to the receiver across multiple messages
    let outcome =
        suite.send_messages_with_gas(accounts.get_mut("sender").unwrap(), 2_500_000, vec![
            Message::Transfer {
                to,
                coins: Coins::one(DENOM.clone(), 10_u128)?,
            },
            Message::Transfer {
                to,
                coins: Coins::one(DENOM.clone(), 15_u128)?,
            },
            Message::Transfer {
                to,
                coins: Coins::one(DENOM.clone(), 20_u128)?,
            },
            Message::Transfer {
                to,
                coins: Coins::one(DENOM.clone(), 25_u128)?,
            },
        ])?;

    outcome.result.should_succeed();

    // Sender remaining balance should be 300k - 70 - withhold + (withhold - charge).
    // = 300k - 70 - charge
    let fee = Uint256::from(outcome.gas_used).checked_mul_dec_ceil(*FEE_RATE)?;
    let sender_balance_after = Uint256::from(300_000_u128 - 70) - fee;

    // Check balances again
    suite
        .query_balance(&accounts["sender"], DENOM.clone())
        .should_succeed_and_equal(sender_balance_after);
    suite
        .query_balance(&accounts["receiver"], DENOM.clone())
        .should_succeed_and_equal(Uint256::from(70_u128));

    let cfg = suite.query_config().should_succeed();

    // List all holders of the denom
    suite
        .query_wasm_smart(cfg.bank, grug_bank::QueryHoldersRequest {
            denom: DENOM.clone(),
            start_after: None,
            limit: None,
        })
        .should_succeed_and_equal(BTreeMap::from([
            (accounts["owner"].address, fee),
            (accounts["sender"].address, sender_balance_after),
            (accounts["receiver"].address, Uint256::from(70_u128)),
        ]));

    Ok(())
}

#[test]
fn transfers_with_insufficient_gas_limit() -> anyhow::Result<()> {
    let (mut suite, mut accounts) = TestBuilder::new_with_vm(WasmVm::new(WASM_CACHE_CAPACITY))
        .add_account("owner", Coins::new())?
        .add_account("sender", Coins::one(DENOM.clone(), 200_000_u128)?)?
        .add_account("receiver", Coins::new())?
        .set_owner("owner")?
        .set_fee_rate(*FEE_RATE)
        .build()?;

    let to = accounts["receiver"].address;

    // Make a bank transfer with a small gas limit; should fail.
    // Bank transfers should take around ~1M gas.
    //
    // We can't easily tell whether gas will run out during the Wasm execution
    // (in which case, the error would be a `VmError::GasDepletion`) or during
    // a host function call (in which case, a `VmError::OutOfGas`). We can only
    // say that the error has to be one of the two. Therefore, we simply ensure
    // the error message contains the word "gas".
    let outcome = suite.send_message_with_gas(
        accounts.get_mut("sender").unwrap(),
        100_000,
        Message::Transfer {
            to,
            coins: Coins::one(DENOM.clone(), 10_u128)?,
        },
    )?;

    outcome.result.should_fail();

    // The transfer should have failed, but gas fee already spent is still charged.
    let fee = Uint256::from(outcome.gas_used).checked_mul_dec_ceil(*FEE_RATE)?;
    let sender_balance_after = Uint256::from(200_000_u128) - fee;

    // Tx is went out of gas.
    // Balances should remain the same
    suite
        .query_balance(&accounts["sender"], DENOM.clone())
        .should_succeed_and_equal(sender_balance_after);
    suite
        .query_balance(&accounts["receiver"], DENOM.clone())
        .should_succeed_and_equal(Uint256::ZERO);

    Ok(())
}
