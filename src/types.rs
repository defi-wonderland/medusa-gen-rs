use anyhow::{Context, Result};
use askama::Template;
use std::fs::File;
use std::io::Write as WriteIO;
use std::path::Path;

/// The contract template,
#[derive(Template, Debug, Clone, PartialEq)]
#[template(path = "contract.sol", escape = "none")]
pub struct Contract {
    pub licence: String,
    pub solc: String,
    pub imports: String,
    pub name: String,
    pub parents: String,
}

impl Contract {
    pub fn write_rendered_contract(&self, path: &Path) -> Result<()> {
        let mut f = File::create_new(path.join(format!("{}{}", self.name, ".t.sol")))
            .context(format!("Failed to create contract {}", self.name))?;

        let rendered = self
            .render()
            .context(format!("Fail to render {} contract", self.name))?;

        f.write_all(rendered.as_bytes())
            .context(format!("Failed to write {}", self.name))?;

        Ok(())
    }
}

#[derive(Default)]
pub struct ContractBuilder {
    licence: String,
    solc: String,
    imports: String,
    name: String,
    parents: String,
}

impl ContractBuilder {
    pub fn new() -> ContractBuilder {
        ContractBuilder {
            licence: String::from("MIT"),
            solc: String::from("^0.8.0"),
            imports: String::from(""),
            name: String::from(""),
            parents: String::from(""),
        }
    }

    pub fn with_imports(mut self, imports: String) -> Self {
        self.imports = imports;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_parents(mut self, parents: String) -> Self {
        self.parents = parents;
        self
    }

    pub fn with_type(mut self, contract_type: &ContractType) -> Self {
        self.imports = contract_type.import().to_owned();
        self.name = contract_type.name().to_owned();
        self.parents = contract_type.import_name().to_owned();
        self
    }

    pub fn build(self) -> Contract {
        Contract {
            licence: self.licence,
            solc: self.solc,
            imports: self.imports,
            name: self.name,
            parents: self.parents,
        }
    }
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
    pub fn directory_name(&self) -> &'static str {
        match self {
            ContractType::Handler => "handlers",
            ContractType::Property => "properties",
            _ => "",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ContractType::Handler => "Handlers",
            ContractType::Property => "Properties",
            ContractType::EntryPoint => "FuzzTest",
            ContractType::Setup => "Setup",
        }
    }

    pub fn import(&self) -> &'static str {
        match self {
            ContractType::Handler => "import {Setup} from '../Setup.t.sol';\n",
            ContractType::Property => {
                "import {HandlersParent} from '../handlers/HandlersParent.t.sol';\n"
            }
            ContractType::EntryPoint => {
                "import {PropertiesParent} from './properties/PropertiesParent.t.sol';\n"
            }
            ContractType::Setup => "",
        }
    }

    pub fn import_name(&self) -> &'static str {
        match self {
            ContractType::Handler => "Setup",
            ContractType::Property => "HandlersParent",
            ContractType::EntryPoint => "PropertiesParent",
            ContractType::Setup => "",
        }
    }
}
