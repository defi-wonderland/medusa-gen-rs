use crate::types::{Contract, ContractBuilder};
use crate::{Args, ContractType};
use anyhow::{Context, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fmt::Write;
use std::fs::DirBuilder;
use std::path::Path;
use tempfile::TempDir;

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

/// create a vec of contracts of a given type
fn create_contracts(contract_type: ContractType, count: u8, path: &Path) -> Result<Vec<Contract>> {
    let mut contracts = Vec::new();

    // directories
    DirBuilder::new()
        .recursive(true)
        .create(path)
        .context(format!(
            "Failed to create directory for {} contracts",
            contract_type.directory_name()
        ))?;

    for i in 0..count {
        let contract = ContractBuilder::new()
            .with_type(&contract_type)
            .with_name(format!("{}{}", contract_type.name(), (b'A' + i) as char))
            .build();

        contract.write_rendered_contract(path).context(format!(
            "Failed to write rendered {} contract",
            contract_type.name()
        ))?;

        contracts.push(contract);
    }

    Ok(contracts)
}

/// Generate parents contracts and write them to a temp folder
fn generate_parents(
    contract_type: ContractType,
    args: &Args,
    path: &Path,
) -> Result<Vec<Contract>> {
    // Determine the number of parents to generate
    let count = match contract_type {
        ContractType::Handler => args.nb_handlers,
        ContractType::Property => args.nb_properties,
        _ => {
            return Err(anyhow::anyhow!("Invalid contract type in generate_parents"));
        }
    };

    // Use the helper function to generate the contracts
    create_contracts(contract_type, count, path)
}

/// Move the content of a temp folder to the fuzz test folder
fn move_temp_contents(temp_dir: &TempDir, overwrite: bool) -> Result<()> {
    let path = Path::new("./test/invariants/fuzz");
    if path.exists() {
        if !overwrite {
            return Err(anyhow::anyhow!(
                "Fuzz test folder already exists, did you mean --overwrite ?"
            ));
        }
    } else {
        DirBuilder::new()
            .recursive(true)
            .create(path)
            .context("Failed to create fuzz test folder")?;
    }

    let options = CopyOptions {
        overwrite,
        skip_exist: !overwrite,
        content_only: true,
        ..Default::default()
    };

    copy(temp_dir.path(), "./test/invariants/fuzz", &options)
        .context("Failed to copy temp directory contents")?;

    Ok(())
}

/// Generate and write the test suite
pub fn generate_test_suite(args: &Args) -> Result<()> {
    let temp_dir = TempDir::new().context("Failed creating temp dir")?; // will be deleted once dropped

    let handler_parents = generate_parents(
        ContractType::Handler,
        args,
        &temp_dir.path().join(ContractType::Handler.directory_name()),
    )
    .context("Failed to generate handler parents")?;

    let handler_child = ContractBuilder::new()
        .with_type(&ContractType::Handler)
        .with_name(format!("{}Parent", &ContractType::Handler.name()))
        .with_imports(parse_child_imports(&handler_parents))
        .with_parents(parse_parents(&handler_parents))
        .build();

    handler_child
        .write_rendered_contract(&temp_dir.path().join(ContractType::Handler.directory_name()))
        .context("Failed to write rendered handler child")?;

    let properties_parents = generate_parents(
        ContractType::Property,
        args,
        &temp_dir
            .path()
            .join(ContractType::Property.directory_name()),
    )
    .context("Failed to generate handler property")?;

    let property_child = ContractBuilder::new()
        .with_type(&ContractType::Property)
        .with_name(format!("{}Parent", &ContractType::Property.name()))
        .with_imports(parse_child_imports(&properties_parents))
        .with_parents(parse_parents(&properties_parents))
        .build();

    property_child
        .write_rendered_contract(
            &temp_dir
                .path()
                .join(ContractType::Property.directory_name()),
        )
        .context("Failed to write rendered property child")?;

    let entry_point = ContractBuilder::new()
        .with_type(&ContractType::EntryPoint)
        .build();

    entry_point
        .write_rendered_contract(temp_dir.path())
        .context("Failed to write rendered entry point")?;

    let setup = ContractBuilder::new()
        .with_type(&ContractType::Setup)
        .build();

    setup
        .write_rendered_contract(temp_dir.path())
        .context("Failed to write rendered setup point")?;

    move_temp_contents(&temp_dir, args.overwrite).context("Failed to move temp contents")?;

    Ok(())
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

    // #[test]
    // fn test_generate_family_with_handler() -> Result<()> {
    //     let tmpdir = std::env::temp_dir();
    //     env::set_current_dir(&tmpdir)?;

    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_family(&args, ContractType::Handler);

    //     assert!(result.is_ok());

    //     assert!(Path::new("handlers").is_dir());
    //     assert!(Path::new("handlers/HandlerA.t.sol").is_file());
    //     assert!(Path::new("handlers/HandlerB.t.sol").is_file());
    //     assert!(!Path::new("handlers/HandlerC.t.sol").is_file());
    //     assert!(Path::new("handlers/HandlerParent.t.sol").is_file());

    //     Ok(())
    // }

    // #[test]
    // fn test_generate_family_with_property() -> Result<()> {
    //     let tmpdir = std::env::temp_dir();
    //     env::set_current_dir(&tmpdir)?;

    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_family(&args, ContractType::Property);

    //     assert!(result.is_ok());

    //     assert!(Path::new("properties").is_dir());
    //     assert!(Path::new("properties/PropertyA.t.sol").is_file());
    //     assert!(Path::new("properties/PropertyB.t.sol").is_file());
    //     assert!(!Path::new("properties/PropertyC.t.sol").is_file());
    //     assert!(Path::new("properties/PropertyParent.t.sol").is_file());

    //     Ok(())
    // }

    // #[test]
    // fn test_generate_family_with_setup_fail() {
    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_family(&args, ContractType::Setup);
    //     let error = result.as_ref().unwrap_err();

    //     assert_eq!(format!("{}", error), "Invalid contract type in gen family");
    // }

    // #[test]
    // fn test_generate_family_with_entry_point_fail() {
    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_family(&args, ContractType::EntryPoint);
    //     let error = result.as_ref().unwrap_err();

    //     assert_eq!(format!("{}", error), "Invalid contract type in gen family");
    // }

    // #[test]
    // fn test_generate_contract_with_setup() -> Result<()> {
    //     let tmpdir = std::env::temp_dir();
    //     env::set_current_dir(&tmpdir)?;

    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_contract(
    //         &args,
    //         ContractType::Setup,
    //         &ContractType::Setup.name(),
    //         &tmpdir,
    //     );

    //     assert!(result.is_ok());

    //     assert_eq!(
    //         result.unwrap(),
    //         Contract {
    //             licence: "MIT".to_string(),
    //             solc: "0.8.23".to_string(),
    //             imports: "".to_string(),
    //             name: "Setup".to_string(),
    //             parents: "".to_string(),
    //         }
    //     );

    //     assert!(Path::new("Setup.t.sol").is_file());
    //     Ok(())
    // }

    // #[test]
    // fn test_generate_contract_with_entry_point() -> Result<()> {
    //     let tmpdir = std::env::temp_dir();
    //     env::set_current_dir(&tmpdir)?;

    //     let args = Args {
    //         overwrite: true,
    //         solc: "0.8.23".to_string(),
    //         nb_handlers: 2,
    //         nb_properties: 2,
    //     };

    //     let result = generate_contract(
    //         &args,
    //         ContractType::EntryPoint,
    //         &ContractType::EntryPoint.name(),
    //         &tmpdir,
    //     );

    //     assert!(result.is_ok());

    //     assert_eq!(
    //         result.unwrap(),
    //         Contract {
    //             licence: "MIT".to_string(),
    //             solc: "0.8.23".to_string(),
    //             imports: "import {PropertiesParent} from './properties/PropertiesParent.t.sol';"
    //                 .to_string(),
    //             name: "FuzzTest".to_string(),
    //             parents: "PropertiesParent".to_string(),
    //         }
    //     );

    //     assert!(Path::new("FuzzTest.t.sol").is_file());

    //     Ok(())
    // }
}
