use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,

    pub cat_token_contract: Addr,
    pub genesis_timestamp: Timestamp,
    pub funds_wallet: Addr,
}

pub const STATE: Item<State> = Item::new("state");
