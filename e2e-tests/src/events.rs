use crate::pair_contract;

pub fn get_events<I, T>(contract_events: I) -> impl Iterator<Item = T>
where
    I: IntoIterator<Item = Result<T, scale::Error>>,
{
    contract_events.into_iter().filter_map(|res| res.ok())
}

pub fn get_mint_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    get_events(contract_events)
        .filter(|event| matches!(event, pair_contract::event::Event::Mint { .. }))
        .collect()
}

pub fn get_swap_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    get_events(contract_events)
        .filter(|event| matches!(event, pair_contract::event::Event::Swap { .. }))
        .collect()
}

pub fn get_burn_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    get_events(contract_events)
        .filter(|event| matches!(event, pair_contract::event::Event::Burn { .. }))
        .collect()
}
