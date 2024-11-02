use askama::Template;

/// The contract template,
#[derive(Template)]
#[template(path = "contract.sol", escape = "none")]
pub struct Contract {
    pub licence: String,
    pub solc: String,
    pub imports: String,
    pub name: String,
    pub parents: String,
}

/// The type of contract to generate
pub enum ContractType {
    Handler,
    Property,
    EntryPoint,
    Setup,
}

/// Hold the contract type specific information
impl ContractType {
    pub fn directory_name(&self) -> String {
        match self {
            ContractType::Handler => "handlers".to_string(),
            ContractType::Property => "properties".to_string(),
            _ => "".to_string(),
        }
    }

    pub fn parents_name(&self) -> String {
        match self {
            ContractType::Handler => "Handler".to_string(),
            ContractType::Property => "Property".to_string(),
        }
    }

    pub fn import_name(&self) -> String {
        match self {
            ContractType::Handler => "Setup".to_string(),
            ContractType::Property => "HandlersParent".to_string(),
        }
    }

    pub fn import_path(&self) -> String {
        match self {
            ContractType::Handler => "./".to_string(),
            ContractType::Property => "../".to_string(),
        }
    }
}
