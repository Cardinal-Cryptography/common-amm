use crate::{
    factory_contract,
    pair_contract,
};

pub fn get_pair_created_events(
    contract_events: Vec<Result<factory_contract::event::Event, scale::Error>>,
) -> Vec<factory_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| res.ok())
        .collect()
}

pub fn get_mint_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| {
            let event = res.ok();
            match event {
                Some(pair_contract::event::Event::Mint { .. }) => event,
                _ => None,
            }
        })
        .collect()
}

pub fn get_swap_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| {
            let event = res.ok();
            match event {
                Some(pair_contract::event::Event::Swap { .. }) => event,
                _ => None,
            }
        })
        .collect()
}

pub fn get_burn_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| {
            let event = res.ok();
            match event {
                Some(pair_contract::event::Event::Burn { .. }) => event,
                _ => None,
            }
        })
        .collect()
}
