#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cat-mint";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        cat_token_contract: deps.api.addr_validate(msg.cat_token_contract.as_str())?,
        genesis_timestamp: _env.block.time,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintCat {} => mint_cat(deps, info, _env),
        ExecuteMsg::UpdateConfig { cat_token_contract } => {
            update_config(deps, info, _env, cat_token_contract)
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    cat_token_contract: Option<String>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    if info.sender.ne(&state.owner) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(ctc) = cat_token_contract {
        state.cat_token_contract = deps.api.addr_validate(ctc.as_str())?;
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn mint_cat(deps: DepsMut, info: MessageInfo, env: Env) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    if info.funds.len() > 1 {
        return Err(ContractError::MultipleCoinsSent {});
    }

    let sent_coin = info.funds.first().unwrap();
    if sent_coin.denom.ne(&String::from("uluna")) {
        return Err(ContractError::WrongDenom {});
    }

    let seconds_since_inception = env
        .block
        .time
        .minus_seconds(state.genesis_timestamp.seconds())
        .seconds();
    let weeks_since_inception = seconds_since_inception / 604800;
    let luna_price = u128::pow(10, weeks_since_inception as u32);
    let expected_sent_coin_amount = luna_price.checked_mul(u128::pow(10, 6)).unwrap();
    if sent_coin.amount.u128().lt(&expected_sent_coin_amount) {
        return Err(ContractError::InsufficientFunds {});
    }

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: state.cat_token_contract.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                // mint one coin only
                amount: Uint128::new(1_u128),
            })?,
            funds: vec![],
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse { state: Some(state) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Api, Coin, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        let msg = InstantiateMsg {
            cat_token_contract: "cat_token_contract".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let state = STATE.load(deps.as_mut().storage).unwrap();
        assert_eq!(
            state,
            State {
                owner: Addr::unchecked("creator"),
                cat_token_contract: Addr::unchecked("cat_token_contract"),
                genesis_timestamp: env.block.time,
            }
        )
    }

    #[test]
    fn test_mint_cat() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            cat_token_contract: "cat_token_contract".to_string(),
        };
        let info = mock_info("creator", &[Coin::new(1000_u128, "uluna")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(100000000_u128, "uluna"));
        let env = mock_env();
        let msg = ExecuteMsg::MintCat {};
        STATE
            .update(
                deps.as_mut().storage,
                |mut state| -> Result<_, ContractError> {
                    state.genesis_timestamp = env.block.time.minus_seconds(1814200);
                    Ok(state)
                },
            )
            .unwrap();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.messages[0], SubMsg::new(WasmMsg::Execute {
            contract_addr: "cat_token_contract".to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Mint {
                recipient: "anyone".to_string(),
                amount: Uint128::new(1_u128)
            }).unwrap(),
            funds: vec![]
        }))
    }
}
