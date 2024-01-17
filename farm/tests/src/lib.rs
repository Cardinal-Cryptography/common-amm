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
    #[test]
    fn it_works() {
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
    }
}
