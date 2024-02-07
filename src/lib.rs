use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use cw_storage_plus::Item;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Cosmwasm {
        #[from]
        source: cosmwasm_std::StdError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppState {
    pub current_value: u32,
}

#[allow(dead_code)]
const APP_STATE: Item<AppState> = Item::new("app-state");

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub initial_value: u32,
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    InstantiateMsg { initial_value: _ }: InstantiateMsg,
) -> Result<Response> {
    Ok(Response::new())
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    Decrement {},
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response> {
    Ok(Response::new())
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Current {},
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CurrentResp {
    value: u32,
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary> {
    to_json_binary(&CurrentResp { value: 0 }).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use super::*;

    #[test]
    fn sanity_check() {
        let mut app = App::default();
        let code_id = app.store_code(Box::new(ContractWrapper::new(execute, instantiate, query)));
        let user = Addr::unchecked("user");
        let contract = app
            .instantiate_contract(
                code_id,
                user.clone(),
                &InstantiateMsg { initial_value: 5 },
                &[],
                "My contract",
                None,
            )
            .unwrap();

        let get_value = {
            let contract = contract.clone();
            move |app: &App| {
                let CurrentResp { value } = app
                    .wrap()
                    .query_wasm_smart(&contract, &QueryMsg::Current {})
                    .unwrap();
                value
            }
        };
        let mk_exec = |msg: ExecuteMsg| {
            let user = user.clone();
            let contract = contract.clone();
            move |app: &mut App| {
                app.execute(
                    user.clone(),
                    cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                        contract_addr: contract.as_str().to_owned(),
                        msg: to_json_binary(&msg).unwrap(),
                        funds: vec![],
                    }),
                )
                .unwrap();
            }
        };
        let increment = mk_exec(ExecuteMsg::Increment {});
        let decrement = mk_exec(ExecuteMsg::Decrement {});

        assert_eq!(get_value(&app), 5);
        increment(&mut app);
        assert_eq!(get_value(&app), 6);
        decrement(&mut app);
        assert_eq!(get_value(&app), 5);
    }
}
