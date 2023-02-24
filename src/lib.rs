pub mod contract;
pub mod error;
pub mod responses;

#[cfg(test)]
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_point {
    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
    
    use crate::error::ContractError;
    use crate::contract::{ContractExecMsg, ContractQueryMsg, QGContract, InstantiateMsg};
    
    const CONTRACT: QGContract = QGContract::new();
    
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }
    
    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: ContractQueryMsg) -> Result<Binary, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env))
    }
    
    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ContractExecMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

}
