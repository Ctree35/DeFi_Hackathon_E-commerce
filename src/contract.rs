use std::iter::Map;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Uint128, from_binary, AllBalanceResponse, Addr, CosmosMsg, BankMsg};
use cosmwasm_std::OverflowOperation::Add;
use cosmwasm_std::coin;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GoodsResponse, InstantiateMsg, OrdersResponse, QueryMsg, ShippingFeesResponse};
use crate::state::{State, STATE, Goods, GoodsStatus, GOODS_LIST, ORDER_LIST, SHIPPING_FEE_MATRIX, Order, OrderStatus};
use crate::helper::assert_sent_sufficient_coin;
// use serde::de::Unexpected::Map;
use crate::state::GoodsStatus::{Available, Ordered, Sold};
use cosmwasm_std::Order::Ascending;
use crate::ContractError::Unauthorized;
use crate::state::OrderStatus::{Confirmed, Setup, Shipping, WaitingAddressUpload};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:defi_ecommerce";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        balance: deps.querier.query_all_balances(env.contract.address).unwrap(),
        order_cnt: 0,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    // SHIPPING_FEE
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
        ExecuteMsg::Post {name, price, denom, area} => try_post(deps, info, &name, price, &denom, &area),
        ExecuteMsg::Buy {name, area} => try_buy(deps, info, &name, &area),
        ExecuteMsg::Reset {name, price} => try_reset(deps, info, &name, price),
        ExecuteMsg::TakeOrder { id, pub_key} => try_take_order(deps, info, id, pub_key),
        ExecuteMsg::UploadAddress { id, address_enc } => try_upload_address(deps, info, id, address_enc),
        ExecuteMsg::Confirm { id } => try_confirm(deps, info, id),
//        ExecuteMsg::DisputeBroken { id } => try_dispute_broken(deps, info, id),
//        ExecuteMsg::DisputeUnsatisfied { id } => try_dispute_unsatisfied(deps, info, id),
//        ExecuteMsg::DisputeConfirm { id} => try_dispute_confirm(deps, info, id)
        _ => unimplemented!()

    }
}

pub fn try_post(deps: DepsMut, info: MessageInfo, name: &str, price: u32, denom: &str, area: &str) -> Result<Response, ContractError> {
    let good = Goods {
        name: String::from(name),
        seller: info.sender,
        price: Coin {denom: String::from(denom), amount: Uint128::from(price)},
        area: String::from(area),
        status: GoodsStatus::Available
    };
    GOODS_LIST.save(deps.storage, name, &good)?;
    Ok(Response::new().add_attribute("method", "try_post"))
}

pub fn try_buy(deps: DepsMut, info: MessageInfo, name: &str, area: &str) -> Result<Response, ContractError> {
    let mut good = GOODS_LIST.load(deps.storage, name)?;
    if good.status != Available {
        return Err(ContractError::GoodsNotAvailable {});
    }
    assert_sent_sufficient_coin(&info.funds, Some(good.clone().price))?;
    good.status = Ordered;
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(one) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    GOODS_LIST.update(deps.storage, name, update_good)?;
    let order = Order {
        id: STATE.load(deps.storage)?.order_cnt.clone(),
        buyer: info.sender.clone(),
        seller: good.clone().seller,
        goods: good.clone(),
        price: good.clone().price,
        buyer_address: String::from(area),
        shipping_fee: Default::default(),
        shipper: Addr::unchecked("Dummy_Shipper"),
        shipper_key: Default::default(),
        buyer_addr_enc: Default::default(),
        seller_addr_enc: Default::default(),
        status: OrderStatus::Setup
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.order_cnt += 1;
        Ok(state)
    })?;
    ORDER_LIST.save(deps.storage, &order.id.to_string(), &order)?;

    Ok(Response::new().add_attribute("method", "try_buy"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, name: &str, price: u32) -> Result<Response, ContractError> {
    let mut good = GOODS_LIST.load(deps.storage, name)?;
    if good.status != Available {
        return Err(ContractError::GoodsNotAvailable {});
    }
    if good.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    good.price.amount = Uint128::from(price);
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    GOODS_LIST.update(deps.storage, name, update_good)?;
    Ok(Response::new().add_attribute("method", "try_reset"))
}

pub fn try_take_order(deps: DepsMut, info: MessageInfo, id: u32, pub_key: String) -> Result<Response, ContractError> {
    assert_sent_sufficient_coin(&info.funds, Some(coin(10, "LUNA")))?;
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Setup {
        return Err(ContractError::OrderNotAvailable {});
    }
    order.status = WaitingAddressUpload;
    order.shipper = info.sender;
    order.shipper_key = pub_key;
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;

    Ok(Response::new().add_attribute("method", "try_take_order"))
}

pub fn try_upload_address(deps: DepsMut, info: MessageInfo, id: u32, address_enc: String) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != WaitingAddressUpload {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer == info.sender {
        order.buyer_addr_enc = address_enc;
    }
    else if order.seller == info.sender {
        order.seller_addr_enc = address_enc;
    }
    else {
        return Err(ContractError::Unauthorized {});
    }
    if order.buyer_addr_enc != String::default() && order.seller_addr_enc != String::default() {
        order.status = Shipping;
    }
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    Ok(Response::new().add_attribute("method", "try_upload_address"))
}

pub fn try_confirm(deps: DepsMut, info: MessageInfo, id: u32) -> Result<Response, ContractError> {
    let mut order = ORDER_LIST.load(deps.storage, &id.to_string())?;
    if order.status != Shipping {
        return Err(ContractError::OrderNotAvailable {});
    }
    if order.buyer != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    order.status = Confirmed;
    let mut good = GOODS_LIST.load(deps.storage, &order.goods.name)?;
    good.status = Sold;
    order.goods = good.clone();
    let update_good = |d: Option<Goods>| -> StdResult<Goods> {
        match d {
            Some(_) => Ok(good.clone()),
            None => unimplemented!(),
        }
    };
    let update_order = |d: Option<Order>| -> StdResult<Order> {
        match d {
            Some(_) => Ok(order.clone()),
            None => unimplemented!(),
        }
    };
    ORDER_LIST.update(deps.storage, &id.to_string(), update_order)?;
    GOODS_LIST.update(deps.storage, &order.goods.name, update_good)?;
    Ok(Response::new()
        .add_attribute("method", "try_confirm")
        .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.seller.into_string(), amount: vec![order.price] }))
        .add_message(CosmosMsg::Bank(BankMsg::Send { to_address: order.shipper.into_string(), amount: vec![order.shipping_fee] })))
}
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
        QueryMsg::GetOrders {} => to_binary(&query_orders(deps)?),
        QueryMsg::GetShippingFees {} => to_binary(&query_shipping_fees(deps)?),
    }
}

//pub fn query_count(deps: Deps) -> StdResult<CountResponse> {
//    let state = STATE.load(deps.storage)?;
//    Ok(CountResponse{count: state.count})
//}

pub fn query_goods(deps: Deps) -> StdResult<GoodsResponse>{
    // let state = STATE.load(deps.storage)?;
    let good_list: StdResult<Vec<_>> = GOODS_LIST.range(deps.storage, None, None, Ascending).collect();
    let good_list = good_list.unwrap();
    let goods = good_list.iter().map(|x| x.1.clone()).collect();

    Ok(GoodsResponse{goods: {goods}})
}

pub fn query_orders(deps: Deps) -> StdResult<OrdersResponse> {
    let order_list: StdResult<Vec<_>> = ORDER_LIST.range(deps.storage, None, None, Ascending).collect();
    let order_list = order_list.unwrap();
    let orders = order_list.iter().map(|x| x.1.clone()).collect();

    Ok(OrdersResponse{orders: {orders}})
}

pub fn query_shipping_fees(deps: Deps) -> StdResult<ShippingFeesResponse> {
    let shipping_fee_matrix: StdResult<Vec<_>> = SHIPPING_FEE_MATRIX.range(deps.storage, None, None, Ascending).collect();
    let shipping_fee_matrix = shipping_fee_matrix.unwrap();
    let shipping_fees = shipping_fee_matrix.iter().map(|x| x.1.clone()).collect();

    Ok(ShippingFeesResponse{shipping_fees: {shipping_fees}})
}

#[cfg(test)]
mod tests {
    // use core::panicking::panic;
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
            area: String::from("Montreal")
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//        // it worked, let's query the state
//        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//        let value: CountResponse = from_binary(&res).unwrap();
//        assert_eq!(17, value.count);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_buy() {
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
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrders {}).unwrap();
        let value: OrdersResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_reset() {
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
            area: String::from("Montreal")
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);

        let msg2 = ExecuteMsg::Reset {
            name: String::from("TV"),
            price: 20
        };
        let info2 = mock_info("creator_fake", &coins(1000, "earth"));

        let res = execute(deps.as_mut(), mock_env(), info2, msg2.clone());
        match res {
            Err(ContractError::Unauthorized {}) => {},
            _ => panic!("Not buyer, not authorized!")
        }
        let _res = execute(deps.as_mut(), mock_env(), info, msg2);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGoods {}).unwrap();
        let value: GoodsResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_take_order() {
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
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1")
        };
        let info3 = mock_info("shipper1", &coins(20, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg4 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa2")
        };
        let info4 = mock_info("shipper2", &coins(50, "LUNA"));
        let err = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap_err();
        assert_eq!(err, ContractError::OrderNotAvailable {});

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrders {}).unwrap();
        let value: OrdersResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
    }

    #[test]
    fn test_upload_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("seller", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("fuck")
        };
        let info3 = mock_info("shipper", &coins(20, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg4 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info4 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg5 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info5 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info5, msg5).unwrap();
    }

    #[test]
    fn test_confirm() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("seller", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Post {
            name: String::from("TV"),
            price: 200,
            denom: String::from("LUNA"),
            area: String::from("Montreal")
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg2 = ExecuteMsg::Buy {
            name: String::from("TV"),
            area: String::from("Montreal")
        };

        let info2 = mock_info("buyer", &coins(2000, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info2, msg2).unwrap();

        let msg3 = ExecuteMsg::TakeOrder {
            id: 0,
            pub_key: String::from("rsa1")
        };
        let info3 = mock_info("shipper", &coins(20, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info3, msg3).unwrap();

        let msg4 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info4 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info4, msg4).unwrap();

        let msg5 = ExecuteMsg::UploadAddress {
            id: 0,
            address_enc: String::from("my address")
        };
        let info5 = mock_info("seller", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info5, msg5).unwrap();

        let msg6 = ExecuteMsg::Confirm {
            id: 0,
        };
        let info6 = mock_info("buyer", &coins(0, "LUNA"));
        let _res = execute(deps.as_mut(), mock_env(), info6, msg6).unwrap();
    }
}