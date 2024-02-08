use crate::*;

use farm::{self, FarmDetails, FarmError, PSP22Error};
use psp22;
use utils::*;

use drink::session::Session;

#[drink::test]
fn farm_start(mut session: Session) {
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        BOB,
    );

    let farm_details: FarmDetails = farm::get_farm_details(&mut session, &farm);

    let expected_details = FarmDetails {
        pool_id: ice.into(),
        reward_tokens: vec![wood.into(), sand.into()],
        reward_rates: vec![0, 0],
        start: 0,
        end: 0,
    };

    assert!(farm_details == expected_details);

    // Fix timestamp, otherwise it changes with every invocation.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let farm_start = now + 100;
    let farm_end = farm_start + 100;
    let rewards_amount = 100;

    let start_result = farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        BOB,
    );

    let insufficient_allowance = FarmError::PSP22Error(PSP22Error::InsufficientAllowance());

    assert_eq!(
        start_result,
        Err(insufficient_allowance),
        "Caller hasn't increased allowance to spend reward tokens for the farm"
    );

    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);
    psp22::increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB);

    let bob_wood_balance_before = psp22::balance_of(&mut session, wood.into(), bob());
    let bob_sand_balance_before = psp22::balance_of(&mut session, sand.into(), bob());
    let farm_wood_balance_before = psp22::balance_of(&mut session, wood.into(), farm.into());
    let farm_sand_balance_before = psp22::balance_of(&mut session, sand.into(), farm.into());

    let call_result = farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        BOB,
    );

    assert!(call_result.is_ok());

    let expected_details = FarmDetails {
        pool_id: ice.into(),
        reward_tokens: vec![wood.into(), sand.into()],
        reward_rates: vec![1, 1],
        start: farm_start,
        end: farm_end,
    };

    let farm_details = farm::get_farm_details(&mut session, &farm);

    assert!(farm_details == expected_details);

    let bob_wood_balance_after = psp22::balance_of(&mut session, wood.into(), bob());
    let bob_sand_balance_after = psp22::balance_of(&mut session, sand.into(), bob());
    assert!(
        bob_wood_balance_after == bob_wood_balance_before - rewards_amount,
        "Farm start must deduct rewards from the caller"
    );
    assert!(
        bob_sand_balance_after == bob_sand_balance_before - rewards_amount,
        "Farm start must deduct rewards from the caller"
    );
    let farm_wood_balance_after = psp22::balance_of(&mut session, wood.into(), farm.into());
    let farm_sand_balance_after = psp22::balance_of(&mut session, sand.into(), farm.into());
    assert!(
        farm_wood_balance_after == farm_wood_balance_before + rewards_amount,
        "Farm start must transfer rewards to the farm"
    );
    assert!(
        farm_sand_balance_after == farm_sand_balance_before + rewards_amount,
        "Farm start must transfer rewards to the farm"
    );
}

const FARM_OWNER: drink::AccountId32 = BOB;
const FARMER: drink::AccountId32 = ALICE;

#[drink::test]
fn owner_withdraws_reward_token_before_farm_start(mut session: Session) {
    // Hats submission 0x1198b6c533d75e9d605e8bf0433c390a22e251d622bb963233c4313255b42759

    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD and SAND - reward tokens
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    // Fix timestamp, otherwise it uses underlying one.
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
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
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
    let withdraw_from_active = farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER);
    assert!(withdraw_from_active == Err(FarmError::FarmIsRunning()));
}

#[drink::test]
fn second_stop_is_noop(mut session: Session) {
    // Set up the necessary tokens.
    // ICE - LP token
    // WOOD reward token
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    // Fix timestamp, otherwise it uses underlying one.
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
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    set_timestamp(&mut session, farm_start + 10);
    let first_farm_end = farm::get_farm_details(&mut session, &farm).end;
    assert!(
        first_farm_end == farm_end,
        "Farm end should be set to the original end"
    );
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();
    let second_farm_end = farm::get_farm_details(&mut session, &farm).end;
    assert!(
        second_farm_end == farm_start + 10,
        "Farm end should be set to the current time in owner_stop_farm"
    );

    set_timestamp(&mut session, farm_start + 20);
    assert!(
        farm::owner_stop_farm(&mut session, &farm, FARM_OWNER)
            == Err(FarmError::FarmAlreadyStopped())
    );
}

#[drink::test]
fn non_farmer_claim_zero_rewards(mut session: Session) {
    // Fix the timestamp, otherwise it uses the underlying UNIX clock.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

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

    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount],
            FARM_OWNER,
        ),
        Ok(())
    );

    set_timestamp(&mut session, farm_end);

    assert_eq!(
        Ok(vec![0]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0], FARM_OWNER),
        "For non-farmer we return 0 rewards",
    )
}

#[drink::test]
fn claim_rewards_long_after_farm_ends(mut session: Session) {
    // Fix the timestamp, otherwise it uses the underlying UNIX clock.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();

    // deposits lp token
    let deposit_amount = 1000000;

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

    assert_eq!(
        farm::start(
            &mut session,
            &farm,
            farm_start,
            farm_end,
            vec![rewards_amount],
            FARM_OWNER,
        ),
        Ok(())
    );

    set_timestamp(&mut session, farm_end);

    let expected_wood_rewards = rewards_amount;
    assert_eq!(
        Ok(vec![expected_wood_rewards]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0], FARMER)
    );
    set_timestamp(&mut session, farm_end + farm_duration);
    assert_eq!(
        Ok(vec![expected_wood_rewards]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0], FARMER),
        "Expected rewards don't change once the farm ends",
    );
}

#[drink::test]
fn deposit_after_farm_ends_does_not_earn_rewards(mut session: Session) {
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);
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
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // Seed the farmer with some tokens to execute txns.
    session
        .sandbox()
        .mint_into(FARMER, 1_000_000_000u128)
        .unwrap();

    // Deposit LP tokens as Alice, not Bob.
    let deposit_amount = 1000000;
    psp22::transfer(&mut session, ice.into(), alice(), deposit_amount, BOB).unwrap();
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARMER,
    );
    set_timestamp(&mut session, farm_start + farm_duration / 2);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARMER).unwrap();

    set_timestamp(&mut session, farm_end + 10);
    assert_eq!(
        Ok(vec![rewards_amount / 2]),
        farm::claim_rewards(&mut session, &farm, vec![0], FARMER),
        "Farmer joined the farm in the middle of its lifetime, so he should get half of the rewards"
    );

    // Bob joins as farmer after farm ends.
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();
    set_timestamp(&mut session, farm_end + 20);
    // But since farm is inactive, he has no rewards.
    assert_eq!(
        Ok(vec![0]),
        farm::query_unclaimed_rewards(&mut session, &farm, vec![0], BOB)
    );
}

#[drink::test]
fn start_stop_emits_event(mut session: Session) {
    use farm::FarmT;
    use ink_wrapper_types::{Connection, ContractEvents};

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

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
    let start_result = session
        .execute(farm.owner_start_new_farm(farm_start, farm_end, vec![rewards_amount]))
        .unwrap();

    assert!(
        start_result.events.len() > 0,
        "expected events emitted from Farm::start"
    );
    let start_events = ContractEvents::from_iter(&start_result.events, farm);
    assert_eq!(
        start_events[0],
        Ok(farm::event::Event::FarmStarted {
            start: farm_start,
            end: farm_end,
            reward_rates: vec![rewards_amount / farm_duration as u128],
        })
    );

    // Stop the farm
    inc_timestamp(&mut session);
    let stop_timestamp = get_timestamp(&mut session);
    let stop_result = session.execute(farm.owner_stop_farm()).unwrap();

    assert!(
        stop_result.events.len() > 0,
        "expected events emitted from Farm::stop"
    );
    let stop_events = ContractEvents::from_iter(&stop_result.events, farm);
    assert_eq!(
        stop_events[0],
        Ok(farm::event::Event::FarmStopped {
            end: stop_timestamp,
        })
    );
}
