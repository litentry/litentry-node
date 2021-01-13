use codec::{Encode, Decode};
/// Asset type
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq)]
pub enum BlockChainType {
    /// invalid
    INVALID,
    /// eth token
    ETH,
    /// bitcoin
    BTC,
}

impl Default for BlockChainType {
    fn default() -> Self {BlockChainType::INVALID}
}

/// Eth source enum
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq)]
pub enum DataSource {
    /// invalid
    INVALID,
    /// etherscan
    ETHERSCAN,
    /// infura
    INFURA,
    /// blockchain
    BLOCKCHAIN,
}

impl Default for DataSource {
    fn default() -> Self {DataSource::INVALID}
}

/// Task list, each validator choose one task 
// const TASKLIST: [(BlockChainType, DataSource); 3] = [
//     (BlockChainType::ETH, DataSource::ETHERSCAN),
//     (BlockChainType::ETH, DataSource::INFURA),
//     (BlockChainType::BTC, DataSource::BLOCKCHAIN),
// ];

/// Data source to blockchain type
pub fn data_source_to_index(data_source: DataSource) -> u32 {
    match data_source {
        INVALID => u32::MAX, 
        ETHERSCAN => 0,
        INFURA => 1,
        BLOCKCHAIN => 2,
    }
}

/// Data source to blockchain type
pub fn data_source_to_block_chain_type(data_source: DataSource) -> BlockChainType {
    match data_source {
        INVALID => BlockChainType::INVALID, 
        ETHERSCAN => BlockChainType::ETH,
        INFURA => BlockChainType::ETH,
        BLOCKCHAIN => BlockChainType::BTC,
    }
}

/// Http Get URL structure
pub struct HttpGet<'a> {
    pub blockchain: BlockChainType,
    // URL affix
    pub prefix: &'a str,
    pub delimiter: &'a str,
    pub postfix: &'a str,
    pub api_token: &'a str,
}

/// Http Post URL structure
pub struct HttpPost<'a> {
    pub blockchain: BlockChainType,
    // URL affix
    pub url_main: &'a str,
    pub api_token: &'a str,
    // Body affix
    pub prefix: &'a str,
    pub delimiter: &'a str,
    pub postfix: &'a str,
}

/// Request enum to wrap up both get and post method
pub enum HttpRequest<'a> {
    GET(HttpGet<'a>),
    POST(HttpPost<'a>),
}
