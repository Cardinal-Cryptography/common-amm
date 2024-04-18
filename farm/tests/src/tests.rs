use crate::*;

use farm::{self, FarmDetails, FarmError, PSP22Error};
use psp22;
use utils::*;

use drink::runtime::MinimalRuntime;
use drink::session::Session;

#[drink::test]
fn farm_start(mut session: Session) {
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), FARM_OWNER);

    let farm = farm::setup(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        FARM_OWNER,
    );

    let farm_details: FarmDetails = farm::get_farm_details(&mut session, &farm);

    let expected_details = FarmDetails {
        pool_id: ice.into(),
        is_active: false,
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
        FARM_OWNER,
    );

    let insufficient_allowance = FarmError::PSP22Error(PSP22Error::InsufficientAllowance());

    assert_eq!(
        start_result,
        Err(insufficient_allowance),
        "Caller hasn't increased allowance to spend reward tokens for the farm"
    );

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
        FARM_OWNER,
    );

    assert!(call_result.is_ok());

    let expected_details = FarmDetails {
        pool_id: ice.into(),
        is_active: true,
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
    seed_account(&mut session, FARMER);
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
    seed_account(&mut session, FARMER);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARMER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);

    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    // Seed the farmer with some tokens to execute txns.
    seed_account(&mut session, FARMER);

    // deposits lp token
    let deposit_amount = 1000000;

    // Deposit LP tokens as Alice, not Bob.
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

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
    seed_account(&mut session, FARMER);

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
    set_timestamp(&mut session, farm_start + farm_duration / 2);
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

#[drink::test]
fn calc_round_down(mut session: Session) {
    // This test verifies that we don't round down rewards incorrectly.

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    // set up the farm with ICE as the pool token and WOOD as a reward token
    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], BOB);

    let deposit_amount = 100;
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();

    // setting up start, end and the rewards amount
    let duration = 100;
    let farm_start = now;
    let farm_end = farm_start + duration;
    // 1.5 rewards per time unit
    let rewards_amount = 150;

    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        BOB,
    )
    .unwrap();

    set_timestamp(&mut session, farm_end);

    let wood_rewards = farm::claim_rewards(&mut session, &farm, [0].to_vec(), BOB).unwrap();
    let dust = 1;
    assert!(rewards_amount - dust <= wood_rewards[0] && wood_rewards[0] <= rewards_amount);
}

#[drink::test]
fn farm_rewards_distribution(mut session: Session) {
    // Fix timestamp
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARM_OWNER);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    // deposits lp token
    let deposit_amount = 1000000;
    psp22::increase_allowance(
        &mut session,
        ice.into(),
        farm.into(),
        deposit_amount,
        FARM_OWNER,
    );
    let call_result = farm::deposit_to_farm(&mut session, &farm, deposit_amount, FARM_OWNER);
    assert!(call_result.is_ok());

    // setting up start, end and the rewards amount
    let farm_start = now;
    let farm_end = farm_start + 259200000; // 1 MONTH (30 days * 24 hours * 60 min * 60 sec * 1000 millisecond)
    let rewards_amount = 10_000_000000; // USDC 10_000

    // increasing allowance for the reward token
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );

    // starting the new farm
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    // set timestamp to farm end so users earn rewards
    set_timestamp(&mut session, farm_end);

    let bob_wood_balance_before = psp22::balance_of(&mut session, wood.into(), bob());

    // We need to stop farm before we're allowed to withdraw tokens.
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();
    let withdrawn = farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER).unwrap();
    // Farm was farmed for its whole duration so all rewards should have been distributed to farmer.
    let dust = 1;
    assert!(
        withdrawn <= dust,
        "all rewards (modulo dust) should have been distributed to farmer"
    );

    let bob_wood_balance_after = psp22::balance_of(&mut session, wood.into(), bob());
    assert_eq!(bob_wood_balance_after - bob_wood_balance_before, withdrawn)
}

#[drink::test]
fn max_rewards(mut session: Session) {
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);
    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], BOB);

    // deposits lp token
    let deposit_amount = u128::MAX;
    psp22::increase_allowance(&mut session, ice.into(), farm.into(), deposit_amount, BOB);
    farm::deposit_to_farm(&mut session, &farm, deposit_amount, BOB).unwrap();

    let farm_start = now;
    let duration = 100;
    let farm_end = farm_start + duration;
    let rewards_amount = u128::MAX;

    // increasing allowance for the reward token
    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB);

    // starting the new farm
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        BOB,
    )
    .unwrap();

    // set timestamp to farm end so users earn some reward in this farm but AND contract has balance
    set_timestamp(&mut session, farm_end + 1);

    // We're not intersted in exact numbers here, more in the fact
    // that the contract can handle such big numbers - u128::MAX for rewards, u128::MAX for deposit.

    let rewards = farm::claim_rewards(&mut session, &farm, vec![0], BOB).unwrap();
    // Stop & withdraw
    farm::owner_stop_farm(&mut session, &farm, BOB).unwrap();
    farm::owner_withdraw(&mut session, &farm, wood.into(), BOB).unwrap();
}

/// Creates a farm where ICE is the LP token and WOOD is the reward token.
///
/// Returns (Farm, ICE, WOOD)
fn setup_farm(
    session: &mut Session<MinimalRuntime>,
    farm_start: u64,
    farm_end: u64,
    rewards_amount: u128,
) -> (crate::farm::Farm, crate::psp22::PSP22, crate::psp22::PSP22) {
    let ice = psp22::setup(session, ICE.to_string(), ICE.to_string(), FARMER);
    let wood = psp22::setup(session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let farm = farm::setup(session, ice.into(), vec![wood.into()], FARM_OWNER);

    psp22::increase_allowance(
        session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );

    // starting the new farm
    farm::start(
        session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    (farm, ice, wood)
}

#[drink::test]
fn owner_stop_farm_before_start(mut session: Session<MinimalRuntime>) {
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    seed_account(&mut session, FARMER);

    // Stop before it even starts.
    let (farm, ice, _wood) = setup_farm(&mut session, now + 10, now + 100, u128::MAX);

    let deposit_amount = 1_000_000;
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();
    let details = farm::get_farm_details(&mut session, &farm);
    assert_eq!(details.is_active, false);
    assert_eq!(
        details.end,
        now + 10,
        "When stopped before start, end == start"
    );
}

#[drink::test]
fn owner_stop_farm_while_running(mut session: Session<MinimalRuntime>) {
    let now = get_timestamp(&mut session);
    let farm_start = now + 10;
    let farm_duration = 100;
    seed_account(&mut session, FARMER);

    let (farm, ice, _) = setup_farm(
        &mut session,
        farm_start,
        farm_start + farm_duration,
        u128::MAX,
    );

    let deposit_amount = 1_000_000;
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

    set_timestamp(&mut session, farm_start + 10);
    let now = get_timestamp(&mut session);
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();
    let details = farm::get_farm_details(&mut session, &farm);
    assert_eq!(details.is_active, false);
    assert_eq!(details.end, now, "When stopped while running, end == now");
}

#[drink::test]
fn owner_stop_farm_after_end(mut session: Session<MinimalRuntime>) {
    let now = get_timestamp(&mut session);
    let farm_start = now + 10;
    let farm_duration = 100;
    let farm_end = farm_start + farm_duration;
    seed_account(&mut session, FARMER);

    let (farm, ice, _) = setup_farm(
        &mut session,
        farm_start,
        farm_start + farm_duration,
        u128::MAX,
    );

    let deposit_amount = 1_000_000;
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

    set_timestamp(&mut session, farm_end + 10);
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();
    let details = farm::get_farm_details(&mut session, &farm);
    assert_eq!(details.is_active, false);
    assert_eq!(
        details.end, farm_end,
        "When stopped after finished, end == planned end"
    );
}

#[drink::test]
fn owner_withdraw_pool_token(mut session: Session<MinimalRuntime>) {
    let now = get_timestamp(&mut session);
    let farm_start = now + 10;
    let farm_duration = 100;
    seed_account(&mut session, FARMER);

    let (farm, ice, _) = setup_farm(
        &mut session,
        farm_start,
        farm_start + farm_duration,
        u128::MAX,
    );

    let deposit_amount = 1_000_000;
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

    // Transfer LP token to the farm, by mistake.
    let lp_token_amount = 1000;
    // Some LP tokens have been trasferred by farmers, when joining the farm.
    let ice_farm_balance_before = psp22::balance_of(&mut session, ice.into(), farm.into());
    psp22::transfer(
        &mut session,
        ice.into(),
        farm.into(),
        lp_token_amount,
        FARMER,
    )
    .unwrap();
    let ice_farm_balance = psp22::balance_of(&mut session, ice.into(), farm.into());
    assert_eq!(ice_farm_balance - ice_farm_balance_before, lp_token_amount);

    // Withdraw LP token from the farm.
    // We need to wait for the farm to end first.
    set_timestamp(&mut session, farm_start + farm_duration + 10);
    // Stop it explicitly first.
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();

    let owner_lp_before = psp22::balance_of(&mut session, ice.into(), bob());
    let res = farm::owner_withdraw(&mut session, &farm, ice.into(), FARM_OWNER);
    assert_eq!(res, Ok(lp_token_amount));

    let ice_farm_balance = psp22::balance_of(&mut session, ice.into(), farm.into());
    assert_eq!(ice_farm_balance, ice_farm_balance_before);

    let owner_lp_after = psp22::balance_of(&mut session, ice.into(), bob());
    assert_eq!(owner_lp_before + lp_token_amount, owner_lp_after);
}

#[drink::test]
fn owner_add_reward_token_failures(mut session: Session<MinimalRuntime>) {
    use ink_primitives::AccountId;

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    seed_account(&mut session, FARMER);

    let farm_start = now + 10;
    let farm_duration = 100;
    let farm_end = farm_start + farm_duration;

    // pool
    let ice = psp22::setup(&mut session, ICE.to_string(), ICE.to_string(), FARMER);
    // reward
    let wood = psp22::setup(&mut session, WOOD.to_string(), WOOD.to_string(), FARM_OWNER);
    let farm = farm::setup(&mut session, ice.into(), vec![wood.into()], FARM_OWNER);

    let farm_details: FarmDetails = farm::get_farm_details(&mut session, &farm);
    let expected_details = FarmDetails {
        pool_id: ice.into(),
        is_active: false,
        reward_tokens: vec![wood.into()],
        reward_rates: vec![0],
        start: 0,
        end: 0,
    };
    assert_eq!(farm_details, expected_details);

    let deposit_amount = 1000000;
    let _ = farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER);

    let rewards_amount = u128::MAX;
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        rewards_amount,
        FARM_OWNER,
    );

    // starting the new farm
    farm::start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount],
        FARM_OWNER,
    )
    .unwrap();

    let fake_token = AccountId::from([11u8; 32]);
    let add_result = farm::owner_add_reward_token(&mut session, &farm, FARM_OWNER, fake_token);
    assert_eq!(add_result, Err(FarmError::FarmIsRunning()));

    let wrong_caller_res = farm::owner_add_reward_token(&mut session, &farm, FARMER, wood.into());
    assert_eq!(wrong_caller_res, Err(FarmError::CallerNotOwner()));

    set_timestamp(&mut session, now + 50);

    let _ = farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).expect("To succeed");
    let add_result = farm::owner_add_reward_token(&mut session, &farm, FARM_OWNER, wood.into());
    assert_eq!(add_result, Err(FarmError::DuplicateRewardTokens()));
}

#[drink::test]
fn owner_add_reward_token_success(mut session: Session<MinimalRuntime>) {
    seed_account(&mut session, FARMER);

    // Start the BASE farm, without new tokens yet.
    let now = get_timestamp(&mut session);
    let farm_start = now + 10;
    let farm_duration = 100u64;

    let reward_amount = farm_duration as u128 * 1_000_000u128;

    let (farm, ice, wood) = setup_farm(
        &mut session,
        farm_start,
        farm_start + farm_duration,
        reward_amount,
    );

    let deposit_amount = 1_000_000;
    farm::join_farm(&mut session, ice.into(), &farm, deposit_amount, FARMER).unwrap();

    set_timestamp(&mut session, farm_start + farm_duration / 2);
    let rewards = farm::claim_rewards(&mut session, &farm, vec![0], FARMER).unwrap();
    let half_rewards = reward_amount / 2;
    assert_eq!(rewards[0], half_rewards);

    // Stop the farm exactly in its middle, there should be still half of the `reward_amount` left.
    farm::owner_stop_farm(&mut session, &farm, FARM_OWNER).unwrap();

    // Extend farm with new reward token.

    // Create token
    let sand = psp22::setup(&mut session, SAND.to_string(), SAND.to_string(), FARM_OWNER);
    // Add the new reward
    assert_eq!(
        Ok(()),
        farm::owner_add_reward_token(&mut session, &farm, FARM_OWNER, sand.into())
    );

    let farm_details: FarmDetails = farm::get_farm_details(&mut session, &farm);
    assert_eq!(farm_details.reward_tokens, vec![wood.into(), sand.into()]);

    // Start new farm instance.
    let now = get_timestamp(&mut session);
    inc_timestamp(&mut session);
    let new_start = now + 10;
    let new_end = new_start + farm_duration;

    let missing_reward_res = farm::start(
        &mut session,
        &farm,
        new_start,
        new_end,
        vec![half_rewards],
        FARM_OWNER,
    );
    assert_eq!(
        missing_reward_res,
        Err(FarmError::RewardsVecLengthMismatch())
    );

    psp22::increase_allowance(
        &mut session,
        sand.into(),
        farm.into(),
        half_rewards,
        FARM_OWNER,
    );

    // We need to withdraw original reward token before starting a new farm.
    assert!(farm::owner_withdraw(&mut session, &farm, wood.into(), FARM_OWNER).is_ok());
    psp22::increase_allowance(
        &mut session,
        wood.into(),
        farm.into(),
        half_rewards,
        FARM_OWNER,
    );

    let start_farm_res = farm::start(
        &mut session,
        &farm,
        new_start,
        new_end,
        vec![half_rewards, half_rewards],
        FARM_OWNER,
    );
    assert_eq!(start_farm_res, Ok(()));

    let new_farm_details = farm::get_farm_details(&mut session, &farm);
    assert_eq!(
        new_farm_details.reward_tokens,
        vec![wood.into(), sand.into()]
    );
    let expected_rate = half_rewards / farm_duration as u128;
    assert_eq!(
        new_farm_details.reward_rates,
        vec![expected_rate, expected_rate]
    );

    // No rewards yet earned, at the beginning of the farm.
    set_timestamp(&mut session, new_start);
    let rewards = farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARMER).unwrap();
    assert_eq!(rewards, vec![0, 0]);

    // Finish farm.
    set_timestamp(&mut session, new_end);
    let rewards = farm::query_unclaimed_rewards(&mut session, &farm, vec![0, 1], FARMER).unwrap();
    assert_eq!(rewards, vec![half_rewards, half_rewards]);

    let rewards = farm::claim_rewards(&mut session, &farm, vec![0, 1], FARMER).unwrap();
    assert_eq!(rewards, vec![half_rewards, half_rewards]);
}
