#[derive(Debug, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl From<usize> for Network {
    fn from(network: usize) -> Self {
        match network {
            0 => Network::Mainnet,
            1 => Network::Testnet,
            _ => panic!("Wrong network"),
        }
    }
}

impl Network {
    pub fn to_string(&self) -> String {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        }
        .to_string()
    }
}
