use cosmrs::proto::{cosmos::base::abci::v1beta1::TxResponse, tendermint::Error};
use aetherium_core::{ChainResult, ModuleType, TxOutcome, H256, U256};
use url::Url;

pub struct IsmType(pub aetherium_cosmwasm_interface::ism::IsmType);

impl From<aetherium_cosmwasm_interface::ism::IsmType> for IsmType {
    fn from(value: aetherium_cosmwasm_interface::ism::IsmType) -> Self {
        IsmType(value)
    }
}

impl From<IsmType> for ModuleType {
    fn from(value: IsmType) -> Self {
        match value.0 {
            aetherium_cosmwasm_interface::ism::IsmType::Unused => ModuleType::Unused,
            aetherium_cosmwasm_interface::ism::IsmType::Routing => ModuleType::Routing,
            aetherium_cosmwasm_interface::ism::IsmType::Aggregation => ModuleType::Aggregation,
            aetherium_cosmwasm_interface::ism::IsmType::LegacyMultisig => {
                ModuleType::MessageIdMultisig
            }
            aetherium_cosmwasm_interface::ism::IsmType::MerkleRootMultisig => {
                ModuleType::MerkleRootMultisig
            }
            aetherium_cosmwasm_interface::ism::IsmType::MessageIdMultisig => {
                ModuleType::MessageIdMultisig
            }
            aetherium_cosmwasm_interface::ism::IsmType::Null => ModuleType::Null,
            aetherium_cosmwasm_interface::ism::IsmType::CcipRead => ModuleType::CcipRead,
        }
    }
}

pub fn tx_response_to_outcome(response: TxResponse) -> ChainResult<TxOutcome> {
    Ok(TxOutcome {
        transaction_id: H256::from_slice(hex::decode(response.txhash)?.as_slice()).into(),
        executed: response.code == 0,
        gas_used: U256::from(response.gas_used),
        gas_price: U256::one().try_into()?,
    })
}
