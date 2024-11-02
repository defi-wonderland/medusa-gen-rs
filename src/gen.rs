use anyhow::{Context, Result};
use askama::Template;
use mockall::*;
use std::fmt::Write;
use std::fs::{DirBuilder, File};
use std::io::Write as WriteIO;
use std::path::Path;

use crate::types::Contract;
use crate::{Args, ContractType};

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

fn create_file(path: &str, should_overwrite: bool) -> Result<File> {
    let f = if should_overwrite {
        File::create(path)
    } else {
        File::create_new(path)
    }?;

    Ok(f)
}

fn write_file(f: &mut File, child_rendered: &str) -> Result<()> {
    f.write_all(child_rendered.as_bytes())?;
    Ok(())
}

/// Create and write either handler or property contracts (parents+child)
pub fn generate_family(args: &Args, contract_type: ContractType) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .create(contract_type.directory_name())
        .context(format!(
            "Fail to create folder for {}",
            contract_type.directory_name()
        ))?;

    let nb_parents = match contract_type {
        ContractType::Handler => args.nb_handlers,
        ContractType::Property => args.nb_properties,
        _ => Err(anyhow::anyhow!("Invalid contract type in gen family"))?,
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
            name: format!("{}{}", contract_type.name(), (b'A' + i) as char),
            parents: contract_type.import_name(),
        })
        .collect();

    // Generate the child contract, which inherit from all the parents
    let child = Contract {
        licence: "MIT".to_string(),
        solc: args.solc.clone(),
        imports: parse_child_imports(parents.as_ref()),
        name: format!("{}Parent", contract_type.name()),
        parents: parse_parents(parents.as_ref()),
    };

    // write all parents
    parents.iter().try_for_each(|p| -> Result<()> {
        let mut f = create_file(
            &format!("{}/{}.t.sol", contract_type.directory_name(), p.name),
            args.overwrite,
        )
        .context(format!("Failed to create {}", p.name))?;

        write_file(
            &mut f,
            &p.render()
                .context(format!("Fail to render {}", contract_type.directory_name()))?,
        )
        .context(format!(
            "fail to write contract {}",
            contract_type.directory_name()
        ))
        .context(format!("Failed to write {}", p.name))?;

        Ok(())
    })?;

    // write child
    let mut f = create_file(
        &format!("{}/{}.t.sol", contract_type.directory_name(), child.name),
        args.overwrite,
    )
    .context(format!("Failed to create {}", child.name))?;

    let child_rendered = child
        .render()
        .context(format!("Fail to render {}", child.name))?;

    write_file(&mut f, &child_rendered).context(format!("Failed to write {}", child.name))?;

    Ok(())
}

/// Create and write a single contract - TODO: reuse in generate_family, not dry...
pub fn generate_contract(args: &Args, contract_type: ContractType) -> Result<Contract> {
    let contract = Contract {
        licence: "MIT".to_string(),
        solc: args.solc.clone(),
        imports: contract_type.import_path(),
        name: contract_type.name(),
        parents: contract_type.import_name(),
    };

    let mut f = create_file(&format!("{}{}", contract.name, ".t.sol"), args.overwrite).context(
        format!("Failed to create {} contract", contract_type.name()),
    )?;

    write_file(
        &mut f,
        &contract
            .render()
            .context(format!("Fail to render {} contract", contract_type.name()))?,
    )
    .context(format!("Failed to write {}", contract_type.name()))?;

    Ok(contract)
}

// TESTS //

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

    #[test]
    fn test_generate_family_with_handler() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_family(&args, ContractType::Handler);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_family_with_property() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_family(&args, ContractType::Property);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_family_with_setup_fail() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_contract(&args, ContractType::Setup);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_family_with_entry_point_fail() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_contract(&args, ContractType::EntryPoint);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_contract_with_setup() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_contract(&args, ContractType::Setup);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_contract_with_entry_poing() {
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 2,
        };

        let result = generate_contract(&args, ContractType::EntryPoint);

        assert!(result.is_ok());
    }
}
