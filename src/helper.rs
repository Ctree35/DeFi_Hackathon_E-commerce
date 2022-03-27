use cosmwasm_std::{Coin, coin, Uint128};
use crate::ContractError;


// Acknowledgement: cw-nameservice = 0.10.0
pub fn assert_sent_sufficient_coin(
    sent: &[Coin],
    required: Vec<Coin>,
) -> Result<(), ContractError> {
    if required.iter().any(|x| x.amount.u128() > 0) {
        let mut sent_sufficient_funds = true;
        for c in required.iter() {
            sent_sufficient_funds &= sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == c.denom && coin.amount.u128() >= c.amount.u128()
            });
        }
        if sent_sufficient_funds {
            return Ok(());
        } else {
            return Err(ContractError::InsufficientFundsSend {});
        }
    }
    Ok(())
}

pub fn merge_coin(coin1: Vec<Coin>, coin2: Vec<Coin>) -> Vec<Coin> {
    let mut merged_coin = vec![];
    for cc in coin1.iter() {
        let another_coin = coin2.iter().find(|&x| x.denom == cc.denom);
        let num = cc.amount + match another_coin {
            Some(c) => c.amount,
            None => Uint128::from(0u32)
        };
        merged_coin.push(coin(num.u128(), cc.clone().denom));
    }
    merged_coin
}