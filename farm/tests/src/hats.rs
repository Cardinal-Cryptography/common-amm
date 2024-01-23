use crate::*;

use farm;
use psp22;
use utils::*;

use drink::session::Session;

#[drink::test]
fn owner_withdraws_reward_token_before_farm_start(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        BOB,
    );

    // Fix timestamp, otherwise it uses underlying one.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    // Start the first farm
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);
    psp22::increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB);
    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount, rewards_amount],
            BOB,
        ),
        Ok(())
    );

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);
    // Deposit LP tokens as Alice, not Bob.
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();
    inc_timestamp(&mut session);
    // Withdraw tokens rewards from the contract, before the farm starts.
    farm::owner_withdraw(&mut session, &farm, wood.into(), BOB).unwrap();
    // Finish farm.
    set_timestamp(&mut session, farm_end);
    // Owner withdrew before the farm started, so the farm has no WOOD tokens.
    let expected_rewards = Ok(vec![0, rewards_amount]);
    let call_result = farm::claim_rewards(&mut session, &farm, vec![0, 1], BOB);
    assert_eq!(expected_rewards, call_result);
    // Without the fix, the above fails with TransferFailed(PSP22Error::InsufficientBalance) because the contract
    // has no WOOD tokens (withdrawn in the exploit above) but farm thought it had.
}

#[drink::test]
fn owner_withdraw_updates_reward_rates(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        BOB,
    );

    // Fix timestamp, otherwise it uses underlying one.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    // Start the first farm
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);
    psp22::increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB);
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        BOB,
    )
    .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);
    // Deposit LP tokens as Alice, not Bob.
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();
    set_timestamp(&mut session, farm_end);
    assert_eq!(
        Ok(vec![rewards_amount, rewards_amount]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], BOB)
    );
    inc_timestamp(&mut session);

    // Start second farm
    let farm_duration = 100;
    let farm_start = get_timestamp(&mut session) + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);
    psp22::increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB);
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        BOB,
    )
    .unwrap();

    // Withdraw tokens rewards from the contract, before the second farm starts.
    farm::owner_withdraw(&mut session, &farm, wood.into(), BOB).unwrap();
    // Finish farm.
    set_timestamp(&mut session, farm_end);
    let expected_rewards = Ok(vec![rewards_amount, rewards_amount * 2]);
    let call_result = farm::claim_rewards(&mut session, &farm, vec![0, 1], BOB);
    assert_eq!(expected_rewards, call_result);
    // Without the fix, the above fails with TransferFailed(PSP22Error::InsufficientBalance) because the contract
    // has half of the WOOD tokens (withdrawn in the exploit above) but farm thought it had.
}
