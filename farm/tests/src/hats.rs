use crate::*;

use farm;
use psp22;
use utils::*;

use drink::session::Session;
use drink::AccountId32;

const FARM_OWNER: AccountId32 = BOB;
const FARMER: AccountId32 = ALICE;

#[drink::test]
fn owner_withdraws_reward_token_before_farm_start(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), FARM_OWNER);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
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
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);
    // Deposit LP tokens as Alice, not Bob.
    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();
    // Deposit LP tokens as Alice, not Bob.
    psp22::transfer(
        &mut session,
        ice.into(),
        alice(),
        deposit_amount,
        FARM_OWNER,
    )
    .unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER).unwrap();
    inc_timestamp(&mut session);
    // Withdraw tokens rewards from the contract, before the farm starts.
    farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER).unwrap();
    // Finish farm.
    set_timestamp(&mut session, farm_end);
    // Owner withdrew before the farm started, so the farm has no WOOD tokens.
    let expected_rewards = Ok(vec![0, rewards_amount]);
    let call_result = farm::claim_rewards(&mut session, &farm, vec![0, 1], FARMER);
    assert_eq!(expected_rewards, call_result);
    // Without the fix, the above fails with TransferFailed(PSP22Error::InsufficientBalance) because the contract
    // has no WOOD tokens (withdrawn in the exploit above) but farm thought it had.
}

#[drink::test]
fn owner_withdraw_updates_reward_rates(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), FARM_OWNER);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
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
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);
    // Deposit LP tokens as Alice, not Bob.
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARM_OWNER,
    );
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARM_OWNER).unwrap();
    set_timestamp(&mut session, farm_end);
    assert_eq!(
        Ok(vec![rewards_amount, rewards_amount]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARM_OWNER)
    );
    inc_timestamp(&mut session);

    // Start second farm
    let farm_duration = 100;
    let farm_start = get_timestamp(&mut session) + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // Withdraw tokens rewards from the contract, before the second farm starts.
    farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER).unwrap();
    // Finish farm.
    set_timestamp(&mut session, farm_end);
    let expected_rewards = Ok(vec![rewards_amount, rewards_amount * 2]);
    let call_result = farm::claim_rewards(&mut session, &farm, vec![0, 1], FARM_OWNER);
    assert_eq!(expected_rewards, call_result);
    // Without the fix, the above fails with TransferFailed(PSP22Error::InsufficientBalance) because the contract
    // has half of the WOOD tokens (withdrawn in the exploit above) but farm thought it had.
}

#[drink::test]
fn claim_rewards_long_after_farm_ends(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    const FARM_OWNER: AccountId32 = BOB;
    const FARMER: AccountId32 = ALICE;

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
    );

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);

    // Deposit LP tokens as Alice, not Bob.
    psp22::transfer(&mut session, ice.into(), alice(), deposit_amount, BOB).unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    assert_eq!(
        farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER),
        Ok(())
    );

    // Start the first farm
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount, rewards_amount],
            FARM_OWNER,
        ),
        Ok(())
    );

    set_timestamp(&mut session, farm_end);

    let expected_wood_rewards = rewards_amount;
    let expected_sand_rewards = rewards_amount;
    let expected_rewards = Ok(vec![expected_wood_rewards, expected_sand_rewards]);
    assert_eq!(
        expected_rewards,
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARMER)
    );
    set_timestamp(&mut session, farm_end + farm_duration);
    assert_eq!(
        expected_rewards,
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARMER),
        "Expected rewards don't change once the farm ends",
    );
}

#[drink::test]
fn deposit_after_finish_doesnt_earn_rewards(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    const FARM_OWNER: AccountId32 = BOB;
    const FARMER: AccountId32 = ALICE;

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
    );

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(ALICE, 1_000_000_000u128)
        .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);

    // Deposit LP tokens as Alice, not Bob.
    psp22::transfer(&mut session, ice.into(), alice(), deposit_amount, BOB).unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    assert_eq!(
        farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER),
        Ok(())
    );

    // Start the first farm
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // set timestamp to farm end so users earn some reward in this farm but AND contract has balance
    set_timestamp(&mut session, farm_end);

    // There's one farmer that owns all LPs and participates in both farms for the whole duration.
    // Owner withdrew WOOD tokens from the contract before the second farm started.
    // So there are no WOOD rewards for the second farm.
    let expected_rewards = Ok(vec![rewards_amount, rewards_amount]);
    assert_eq!(
        expected_rewards,
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARMER)
    );
    set_timestamp(&mut session, farm_end + farm_duration);
    // Deposit to farm after it's finished.
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARM_OWNER,
    );
    assert_eq!(
        farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARM_OWNER),
        Ok(())
    );
    inc_timestamp(&mut session);
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARM_OWNER,
    );
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARM_OWNER).unwrap();
    set_timestamp(&mut session, farm_end + farm_duration + farm_duration);
    assert_eq!(
        Ok(vec![0, 0]),
        farm::claim_rewards(&mut session, &farm, vec![0, 1], FARM_OWNER),
        "Rewards should not accrue after the farm ends"
    );
    // Move the timestmamp.
    inc_timestamp(&mut session);
    assert_eq!(
        Ok(vec![0, 0]),
        farm::claim_rewards(&mut session, &farm, vec![0, 1], FARM_OWNER),
        "Rewards should not accrue after the farm ends"
    );
    inc_timestamp(&mut session);
    farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER).unwrap();
    inc_timestamp(&mut session);
    assert_eq!(
        expected_rewards,
        farm::claim_rewards(&mut session, &farm, vec![0, 1], FARMER)
    );
}

#[drink::test]
fn claim_caller_not_farmer(mut session: Session) {
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
    );
    inc_timestamp(&mut session);

    // Start the first farm
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount, rewards_amount],
            FARM_OWNER,
        ),
        Ok(())
    );

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);

    // Deposit LP tokens as Alice, not Bob.
    psp22::transfer(&mut session, ice.into(), alice(), deposit_amount, BOB).unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER).unwrap();
    assert_eq!(
        farm::claim_rewards(&mut session, &farm, vec![0, 1], BOB),
        Err(farm::FarmError::CallerNotFarmer())
    );
}

#[drink::test]
fn claim_returns_zeros_when_no_rewards(mut session: Session) {
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
    );
    inc_timestamp(&mut session);

    // Start the first farm
    let farm_duration = 100;
    let farm_start = now + 10;
    let farm_end = farm_start + farm_duration;
    let rewards_amount = 100000000000000;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );
    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount, rewards_amount],
            FARM_OWNER,
        ),
        Ok(())
    );

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;
    inc_timestamp(&mut session);

    // Deposit LP tokens as Alice, not Bob.
    psp22::transfer(&mut session, ice.into(), alice(), deposit_amount, BOB).unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER).unwrap();
    set_timestamp(&mut session, farm_end + 10);
    assert_eq!(
        Ok(vec![rewards_amount, rewards_amount]),
        farm::claim_rewards(&mut session, &farm, vec![0, 1], FARMER)
    );
    // Bob joins as farmer after farm ends.
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();
    // But since farm is inactive, he has no rewards.
    assert_eq!(
        Ok(vec![0, 0]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], BOB)
    );
}
