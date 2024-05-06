import BN from 'bn.js';

export type PSP22Metadata = {
    address: string,
    name: string;
    symbol: string;
    decimals: number;
    total_supply: string;
    my_balance: string;
};

export type Reward = {
    token: string;
    amount: BN;
}

export type FarmSpec = {
    poolAddress: string;
    rewards: Reward[];
    startTimestamp: number;
    endTimestamp: number;
}

export type FarmDetails = {
    address: string;
    spec: FarmSpec;
}
