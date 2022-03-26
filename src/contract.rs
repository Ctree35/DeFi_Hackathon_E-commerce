use std::iter::Map;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Uint128, from_binary, AllBalanceResponse, Order};
use cosmwasm_std::Order::Ascending;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GoodsResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, Goods, GoodsStatus, GOODS_LIST};
// use serde::de::Unexpected::Map;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:defi_ecommerce";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        balance: deps.querier.query_all_balances(env.contract.address).unwrap(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Post {name, price, denom, location} => try_post(deps, info, &name, price, &denom, &location),
//        ExecuteMsg::Buy {name, location} => try_buy(deps, info, name, location),
//        ExecuteMsg::Reset { price} => try_reset(deps, info, price),
//        ExecuteMsg::TakeOrder { id, pub_key} => try_take_order(deps, info, id, pub_key),
//        ExecuteMsg::UploadAddress { id, address_enc } => try_upload_address(deps, info, id, address_enc),
//        ExecuteMsg::Confirm { id } => try_confirm(deps, info, id),
//        ExecuteMsg::DisputeBroken { id } => try_dispute_broken(deps, info, id),
//        ExecuteMsg::DisputeUnsatisfied { id } => try_dispute_unsatisfied(deps, info, id),
//        ExecuteMsg::DisputeConfirm { id} => try_dispute_confirm(deps, info, id)
        _ => unimplemented!()

    }
}

pub fn try_post(deps: DepsMut, info: MessageInfo, name: &str, price: u32, denom: &str, location: &str) -> Result<Response, ContractError> {
    let good = Goods {
        name: String::from(name),
        seller: info.sender,
        price: Coin {denom: String::from(denom), amount: Uint128::from(price)},
        location: String::from(location),
        status: GoodsStatus::Available
    };
    GOODS_LIST.save(deps.storage, name, &good)?;
    Ok(Response::new().add_attribute("method", "try_post"))
}

//pub fn try_buy(deps: DepsMut, info: MessageInfo, name: &str, location: &str) -> Result<Response, ContractError> {
//    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//        let good = Goods {
//            name: String::from(name),
//            seller: info.sender,
//            price: Coin {denom: String::from(denom), amount: Uint128::from(price)},
//            location: String::from(location),
//            status: GoodsStatus::Available
//        };
//        state.goods_list.push(Box::new(good));
//        Ok(state)
//    })?;
//    Ok(Response::new().add_attribute("method", "try_post"))
//}
//pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
//    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//        state.count += 1;
//        Ok(state)
//    })?;
//    Ok(Response::new().add_attribute("method", "try_increment"))
//}
//
//pub fn try_decrement(deps: DepsMut) -> Result<Response, ContractError> {
//    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//        state.count -= 1;
//        Ok(state)
//    })?;
//    Ok(Response::new().add_attribute("method", "try_decrement"))
//}
//
//pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
//    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//        if info.sender != state.owner {
//            return Err(ContractError::Unauthorized{});
//        }
//        state.count = count;
//        Ok(state)
//    })?;
//    Ok(Response::new().add_attribute("method", "reset"))
//}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//    match msg {
////        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//        QueryMsg::GetGoods { } => to_binary(&query_goods(deps)?),
//        QueryMsg::GetOrders { } => to_binary(&query_orders(deps)?),
//        QueryMsg::GetDistance { } => to_binary(&query_distance(deps)?),
//        QueryMsg::GetOrderDetail {id} => to_binary(&query_order_detail(deps, id)?),
//        QueryMsg::GetAddresses {id} => to_binary(&query_addresses(deps, id)?),
//    }
    match msg {
        QueryMsg::GetGoods {} => to_binary(&query_goods(deps)?),
    }
}

//pub fn query_count(deps: Deps) -> StdResult<CountResponse> {
//    let state = STATE.load(deps.storage)?;
//    Ok(CountResponse{count: state.count})
//}

pub fn query_goods(deps: Deps) -> StdResult<GoodsResponse>{
    // let state = STATE.load(deps.storage)?;
    let good_list: StdResult<Vec<_>> = GOODS_LIST.range(deps.storage, None, None, Order::Ascending).collect();
    let good_list = good_list.unwrap();
    let goods = good_list.iter().map(|x| x.1.clone()).collect();

    Ok(GoodsResponse{goods: {goods}})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_post() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            location: String::from("Montreal")
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);

//        // it worked, let's query the state
//        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//        let value: CountResponse = from_binary(&res).unwrap();
//        assert_eq!(17, value.count);
    }
}