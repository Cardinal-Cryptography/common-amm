use crate::*;

use farm::{self, FarmDetails, FarmError, PSP22Error};
use psp22;
use utils::*;

use drink::{runtime::MinimalRuntime, session::Session};

#[test]
fn farm_start() {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");

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

    psp22::increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB).unwrap();
    psp22::increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB).unwrap();

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
