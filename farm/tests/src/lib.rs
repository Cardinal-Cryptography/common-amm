#[cfg(test)]
mod farm;

#[cfg(test)]
mod psp22;

#[cfg(test)]
mod tests {
    use crate::farm::{self, Farm as _, Instance as Farm};
    use crate::psp22::{self, Instance as PSP22, PSP22 as _};

    use anyhow::Result;
    use assert2::assert;
    use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
    use ink_primitives::AccountId;
    use ink_wrapper_types::{util::ToAccountId, Connection};

    const ALICE: drink::AccountId32 = AccountId32::new([2u8; 32]);
    const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);

    fn alice() -> ink_primitives::AccountId {
        AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
    }

    fn bob() -> ink_primitives::AccountId {
        AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
    }

    /// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
    /// Returns its AccountId casted to PSP22 interface.
    fn setup_psp22(
        session: &mut Session<MinimalRuntime>,
        name: String,
        symbol: String,
        caller: drink::AccountId32,
    ) -> PSP22 {
        let _code_hash = session.upload_code(psp22::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = PSP22::new(
            1_000_000_000u128 * 10u128.pow(18),
            Some(name),
            Some(symbol),
            18,
        );

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    /// Uploads and creates a Farm instance with given pool_id and rewards.
    /// Returns its AccountId casted to Farm interface.
    fn setup_farm(
        session: &mut Session<MinimalRuntime>,
        pool_id: AccountId,
        rewards: Vec<AccountId>,
        caller: drink::AccountId32,
    ) -> Farm {
        let _code_hash = session.upload_code(farm::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = Farm::new(pool_id, rewards);

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    /// Increases allowance of given token to given spender by given amount.
    fn increase_allowance(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        spender: AccountId,
        amount: u128,
        caller: drink::AccountId32,
    ) -> Result<()> {
        let _ = session.set_actor(caller);

        session
            .execute(PSP22::increase_allowance(&token.into(), spender, amount))
            .unwrap()
            .result
            .unwrap()
            .unwrap();

        Ok(())
    }

    fn balance_of(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        account: AccountId,
    ) -> u128 {
        session
            .query(PSP22::balance_of(&token.into(), account))
            .unwrap()
            .result
            .unwrap()
    }

    fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
        session.sandbox().get_timestamp()
    }

    fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
        session.sandbox().set_timestamp(timestamp);
    }

    #[test]
    fn farm_start() {
        let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");

        let ice = {
            let ice = "ICE".to_string();
            setup_psp22(&mut session, ice.clone(), ice.clone(), BOB)
        };

        let wood = {
            let wood = "WOOD".to_string();
            setup_psp22(&mut session, wood.clone(), wood.clone(), BOB)
        };

        let sand = {
            let sand = "SAND".to_string();
            setup_psp22(&mut session, sand.clone(), sand.clone(), BOB)
        };

        let _farm_code_hash = session.upload_code(farm::upload()).unwrap();

        let farm = setup_farm(
            &mut session,
            ice.into(),
            vec![wood.into(), sand.into()],
            BOB,
        );

        let farm_details: farm::FarmDetails = session
            .query(farm.view_farm_details())
            .unwrap()
            .result
            .unwrap();

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

        let call_result = session
            .execute(farm.owner_start_new_farm(
                farm_start,
                farm_end,
                vec![rewards_amount, rewards_amount],
            ))
            .unwrap()
            .result
            .unwrap();

        assert!(call_result.is_ok());

        let expected_details = farm::FarmDetails {
            pool_id: ice.into(),
            reward_tokens: vec![wood.into(), sand.into()],
            reward_rates: vec![1, 1],
            start: farm_start,
            end: farm_end,
        };

        let farm_details: farm::FarmDetails = session
            .query(farm.view_farm_details())
            .unwrap()
            .result
            .unwrap();

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
}
