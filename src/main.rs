use anyhow::{Context, Result};
use askama::Template;
use clap::Parser;
use std::fmt::Write;
use std::fs::{DirBuilder, File};
use std::io::Write as WriteIO;

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

    /// Overwrite existing files
    #[arg(short, long, default_value = "false")]
    overwrite: bool,
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
fn parse_child_imports(parents: &[Contract]) -> String {
    parents.iter().fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "import {{ {} }} from './{}.t.sol';", b.name, b.name);
        output
    })
}

/// Create the "HandlerA, HandlerB" in "contract HandlersParent is HandlerA, HandlerB"
/// the "is" statement is conditionnaly added in the template
fn parse_parents(parents: &[Contract]) -> String {
    parents
        .iter()
        .fold(String::new(), |mut output, b| {
            let _ = write!(output, "{}, ", b.name);
            output
        })
        .trim_end_matches(", ")
        .to_string()
}

/// Create either handler or property contracts (parents+child)
fn generate_family(args: &Args, contract_type: ContractType) -> Result<()> {
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
        .create(contract_type.directory_name())
        .context(format!(
            "Fail to create folder for {}",
            contract_type.directory_name()
        ))?;

    // create_new prevents overwriting existing files - todo: add a flag to force overwrite
    parents.iter().try_for_each(|p| -> Result<()> {
        let mut f = if args.overwrite {
            File::create(format!(
                "{}/{}.t.sol",
                contract_type.directory_name(),
                p.name
            ))
        } else {
            File::create_new(format!(
                "{}/{}.t.sol",
                contract_type.directory_name(),
                p.name
            ))
        }
        .context(format!(
            "fail to create contract {}",
            contract_type.directory_name()
        ))?;

        f.write_all(
            p.render()
                .context(format!("Fail to render {}", contract_type.directory_name()))?
                .as_bytes(),
        )
        .context(format!(
            "fail to write contract {}",
            contract_type.directory_name()
        ))?;

        Ok(())
    })?;

    let mut f = if args.overwrite {
        File::create(format!(
            "{}/{}.t.sol",
            contract_type.directory_name(),
            child.name
        ))
    } else {
        File::create_new(format!(
            "{}/{}.t.sol",
            contract_type.directory_name(),
            child.name
        ))
    }
    .context(format!(
        "Failed to create {} ",
        contract_type.directory_name()
    ))?;

    let child_rendered = child.render().context(format!(
        "Fail to render child {}",
        contract_type.directory_name()
    ))?;

    f.write_all(child_rendered.as_bytes()).context(format!(
        "Fail to write child {}",
        contract_type.directory_name()
    ))?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    generate_family(&args, ContractType::Handler)
        .context("Failed to generate handlers contracts")?;

    generate_family(&args, ContractType::Property)
        .context("Failed to generated properties contracts")?;

    let fuzz_entry_point = Contract {
        licence: "MIT".to_string(),
        solc: args.solc.clone(),
        imports: "import {PropertiesParent} from './properties/PropertiesParent.t.sol';"
            .to_string(),
        name: "FuzzTest".to_string(),
        parents: "PropertiesParent".to_string(),
    };

    let mut f = if args.overwrite {
        File::create(format!("{}{}", fuzz_entry_point.name, ".t.sol"))
    } else {
        File::create_new(format!("{}{}", fuzz_entry_point.name, ".t.sol"))
    }
    .context("Failed to create entry point contract")?;

    f.write_all(
        fuzz_entry_point
            .render()
            .context("Fail to render")?
            .as_bytes(),
    )
    .context("Failed to write entry point")?;

    let setup_contract = Contract {
        licence: "MIT".to_string(),
        solc: args.solc,
        imports: "".to_string(),
        name: "Setup".to_string(),
        parents: "".to_string(),
    };

    let mut f = if args.overwrite {
        File::create(format!("{}{}", setup_contract.name, ".t.sol"))
    } else {
        File::create_new(format!("{}{}", setup_contract.name, ".t.sol"))
    }
    .context("Fail to create setup contract")?;
    f.write_all(
        setup_contract
            .render()
            .context("Fail to redner setup contract")?
            .as_bytes(),
    )
    .context("Fail to write setup contract")?;

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
