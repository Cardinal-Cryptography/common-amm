use crate::*;
use utils::*;

use farm::Farm as _;

use drink::{runtime::MinimalRuntime, session::Session};
use ink_wrapper_types::Connection;

#[test]
fn farm_start() {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");

    let ice = setup_psp22(&mut session, ICE.to_string(), ICE.to_string(), BOB);

    let wood = setup_psp22(&mut session, WOOD.to_string(), WOOD.to_string(), BOB);

    let sand = setup_psp22(&mut session, SAND.to_string(), SAND.to_string(), BOB);

    let farm = setup_farm(
        &mut session,
        ice.into(),
        vec![wood.into(), sand.into()],
        BOB,
    );

    let farm_details: farm::FarmDetails = get_farm_details(&mut session, &farm);

    let expected_details = farm::FarmDetails {
        pool_id: ice.into(),
        reward_tokens: vec![wood.into(), sand.into()],
        reward_rates: vec![0, 0],
        start: 0,
        end: 0,
    };

    assert!(farm_details == expected_details);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let now_plus_100 = now + 100;
    let farm_start = now_plus_100;
    let farm_end = farm_start + 100;

    let call_result = session
        .query(farm.owner_start_new_farm(farm_start, farm_end, vec![100, 100]))
        .unwrap()
        .result
        .unwrap();

    let insufficient_allowance =
        farm::FarmError::PSP22Error(farm::PSP22Error::InsufficientAllowance());

    assert!(
        call_result == Err(insufficient_allowance),
        "Caller hasn't increased allowance to spend reward tokens for the farm"
    );

    let rewards_amount = 100;

    increase_allowance(&mut session, wood.into(), farm.into(), rewards_amount, BOB).unwrap();
    increase_allowance(&mut session, sand.into(), farm.into(), rewards_amount, BOB).unwrap();

    let bob_wood_balance_before = balance_of(&mut session, wood.into(), bob());
    let bob_sand_balance_before = balance_of(&mut session, sand.into(), bob());
    let farm_wood_balance_before = balance_of(&mut session, wood.into(), farm.into());
    let farm_sand_balance_before = balance_of(&mut session, sand.into(), farm.into());

    let call_result = setup_farm_start(
        &mut session,
        &farm,
        farm_start,
        farm_end,
        vec![rewards_amount, rewards_amount],
        BOB,
    );

    assert!(call_result.is_ok());

    let expected_details = farm::FarmDetails {
        pool_id: ice.into(),
        reward_tokens: vec![wood.into(), sand.into()],
        reward_rates: vec![1, 1],
        start: farm_start,
        end: farm_end,
    };

    let farm_details: farm::FarmDetails = get_farm_details(&mut session, &farm);

    assert!(farm_details == expected_details);

    let bob_wood_balance_after = balance_of(&mut session, wood.into(), bob());
    let bob_sand_balance_after = balance_of(&mut session, sand.into(), bob());
    assert!(
        bob_wood_balance_after == bob_wood_balance_before - rewards_amount,
        "Farm start must deduct rewards from the caller"
    );
    assert!(
        bob_sand_balance_after == bob_sand_balance_before - rewards_amount,
        "Farm start must deduct rewards from the caller"
    );
    let farm_wood_balance_after = balance_of(&mut session, wood.into(), farm.into());
    let farm_sand_balance_after = balance_of(&mut session, sand.into(), farm.into());
    assert!(
        farm_wood_balance_after == farm_wood_balance_before + rewards_amount,
        "Farm start must transfer rewards to the farm"
    );
    assert!(
        farm_sand_balance_after == farm_sand_balance_before + rewards_amount,
        "Farm start must transfer rewards to the farm"
    );
}
