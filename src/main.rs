use askama::Template;
use clap::Parser;
use std::fs::{DirBuilder, File};
use std::io::Write;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Solidity version
    #[arg(short, long, default_value = "0.8.23")]
    solc: String,

    /// Number of handler to generate
    #[arg(short = 'n', long, default_value_t = 2)]
    nb_handlers: u8,

    /// Number of properties contract to generate
    #[arg(short = 'p', long, default_value_t = 2)]
    nb_properties: u8,
}

/// The contract template,
#[derive(Template, Debug)]
#[template(path = "contract.sol", escape = "none")]
struct Contract {
    licence: String,
    solc: String,
    imports: String,
    name: String,
    parents: String,
}

/// The type of contract to generate
enum ContractType {
    Handler,
    Property,
}

/// Hold the contract type specific information
impl ContractType {
    fn directory_name(&self) -> String {
        match self {
            ContractType::Handler => "handlers".to_string(),
            ContractType::Property => "properties".to_string(),
        }
    }

    fn parents_name(&self) -> String {
        match self {
            ContractType::Handler => "Handler".to_string(),
            ContractType::Property => "Property".to_string(),
        }
    }

    fn import_name(&self) -> String {
        match self {
            ContractType::Handler => "Setup".to_string(),
            ContractType::Property => "HandlersParent".to_string(),
        }
    }

    fn import_path(&self) -> String {
        match self {
            ContractType::Handler => "./".to_string(),
            ContractType::Property => "../".to_string(),
        }
    }
}

/// Create the "import { HandlerA, HandlerB } from './handlers/HandlersParent.t.sol';" from a vec of parent contracts
fn parse_child_imports(parents: &Vec<Contract>) -> String {
    parents
        .iter()
        .map(|p| format!("import {{ {} }} from './{}.t.sol';\n", p.name, p.name))
        .collect()
}

/// Create the "HandlerA, HandlerB" in "contract HandlersParent is HandlerA, HandlerB"
/// the "is" statement is conditionnaly added in the template
fn parse_parents(parents: &Vec<Contract>) -> String {
    parents
        .iter()
        .map(|p| format!("{}, ", p.name))
        .collect::<String>()
        .trim_end_matches(", ")
        .to_string()
}

/// Create either handler or property contracts (parents+child)
fn generate_family(args: &Args, contract_type: ContractType) -> Result<(), std::io::Error> {
    let nb_parents = match contract_type {
        ContractType::Handler => args.nb_handlers,
        ContractType::Property => args.nb_properties,
    };

    // Generate the parent contracts
    let parents: Vec<Contract> = (0..nb_parents)
        .map(|i| Contract {
            licence: "MIT".to_string(),
            solc: args.solc.clone(),
            imports: format!(
                "import {{ {} }} from '{}{}.t.sol';\n",
                contract_type.import_name(),
                contract_type.import_path(),
                contract_type.import_name()
            )
            .to_string(),
            name: format!("{}{}", contract_type.parents_name(), (b'A' + i) as char),
            parents: contract_type.import_name(),
        })
        .collect();

    // Generate the child contract, which inherit from all the parents
    let child = Contract {
        licence: "MIT".to_string(),
        solc: args.solc.clone(),
        imports: parse_child_imports(parents.as_ref()),
        name: format!("{}Parent", contract_type.parents_name()),
        parents: parse_parents(parents.as_ref()),
    };

    DirBuilder::new()
        .recursive(true)
        .create(contract_type.directory_name())?;

    // create_new prevents overwriting existing files - todo: add a flag to force overwrite
    parents.iter().try_for_each(|p| -> std::io::Result<()> {
        let mut f = File::create_new(format!(
            "{}/{}.t.sol",
            contract_type.directory_name(),
            p.name
        ))?;

        // Don't judge me, you know you would have done the same
        f.write_all(
            p.render()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                .as_bytes(),
        )?;

        Ok(())
    })?;

    let mut f = File::create_new(format!(
        "{}/{}.t.sol",
        contract_type.directory_name(),
        child.name
    ))?;

    let child_rendered = child
        .render()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    f.write_all(child_rendered.as_bytes())?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    generate_family(&args, ContractType::Handler)?;

    generate_family(&args, ContractType::Property)?;

    let fuzz_entry_point = Contract {
        licence: "MIT".to_string(),
        solc: args.solc.clone(),
        imports: "import {PropertiesParent} from './properties/PropertiesParent.t.sol';"
            .to_string(),
        name: "FuzzTest".to_string(),
        parents: "PropertiesParent".to_string(),
    };

    let mut f = File::create_new(format!("{}{}", fuzz_entry_point.name, ".t.sol"))?;
    f.write_all(fuzz_entry_point.render().unwrap().as_bytes())?;

    let setup_contract = Contract {
        licence: "MIT".to_string(),
        solc: args.solc,
        imports: "".to_string(),
        name: "Setup".to_string(),
        parents: "".to_string(),
    };

    let mut f = File::create_new(format!("{}{}", setup_contract.name, ".t.sol"))?;
    f.write_all(setup_contract.render().unwrap().as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_child_imports() {
        let parents = vec![Contract {
            licence: "MIT".to_string(),
            solc: "0.8.23".to_string(),
            imports: "".to_string(),
            name: "HandlerA".to_string(),
            parents: "HandlersParent".to_string(),
        }];

        assert_eq!(
            parse_child_imports(parents.as_ref()),
            "import { HandlerA } from './HandlerA.t.sol';\n"
        );
    }

    #[test]
    fn test_parse_child_imports_two() {
        let parents = vec![
            Contract {
                licence: "MIT".to_string(),
                solc: "0.8.23".to_string(),
                imports: "".to_string(),
                name: "HandlerA".to_string(),
                parents: "HandlersParent".to_string(),
            },
            Contract {
                licence: "MIT".to_string(),
                solc: "0.8.23".to_string(),
                imports: "".to_string(),
                name: "HandlerB".to_string(),
                parents: "HandlersParent".to_string(),
            },
        ];

        assert_eq!(
                parse_child_imports(parents.as_ref()),
                "import { HandlerA } from './HandlerA.t.sol';\nimport { HandlerB } from './HandlerB.t.sol';\n"
            );
    }

    #[test]
    fn test_parse_child_imports_empty() {
        let parents = vec![];
        assert_eq!(parse_child_imports(parents.as_ref()), "");
    }

    #[test]
    fn test_parse_parents() {
        let parents = vec![Contract {
            licence: "MIT".to_string(),
            solc: "0.8.23".to_string(),
            imports: "".to_string(),
            name: "HandlerA".to_string(),
            parents: "HandlersParent".to_string(),
        }];

        assert_eq!(parse_parents(parents.as_ref()), "HandlerA");
    }

    #[test]
    fn test_parse_parents_two() {
        let parents = vec![
            Contract {
                licence: "MIT".to_string(),
                solc: "0.8.23".to_string(),
                imports: "".to_string(),
                name: "HandlerA".to_string(),
                parents: "HandlersParent".to_string(),
            },
            Contract {
                licence: "MIT".to_string(),
                solc: "0.8.23".to_string(),
                imports: "".to_string(),
                name: "HandlerB".to_string(),
                parents: "HandlersParent".to_string(),
            },
        ];

        assert_eq!(parse_parents(parents.as_ref()), "HandlerA, HandlerB");
    }

    #[test]
    fn test_parse_parents_empty() {
        let parents = vec![];
        assert_eq!(parse_parents(parents.as_ref()), "");
    }
}
