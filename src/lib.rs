mod types;
pub mod cli;

use crate::cli::Args;
use crate::types::{Contract, ContractType, ContractBuilder};

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
fn create_contracts(contract_type: &ContractType, count: u8, path: &Path) -> Result<Vec<Contract>> {
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
            .with_type(contract_type)
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
    create_contracts(&contract_type, count, path)
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
    use serial_test::serial;

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
    fn test_create_contracts() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let contract_type = ContractType::Handler;
        let count = 2;

        let contracts = create_contracts(
            &contract_type,
            count,
            &temp_dir.path().join(contract_type.directory_name())
        )?;

        // Check that the correct number of contracts was created
        assert_eq!(contracts.len(), 2);

        // Check that the contracts have the expected names
        assert_eq!(contracts[0].name, "HandlersA");
        assert_eq!(contracts[1].name, "HandlersB");

        // Verify the files were created in the correct directory
        let handler_dir = temp_dir.path().join("handlers");
        assert!(handler_dir.exists());
        assert!(handler_dir.is_dir());

        // Check that the contract files exist
        assert!(handler_dir.join("HandlersA.t.sol").exists());
        assert!(handler_dir.join("HandlersB.t.sol").exists());
        assert!(!handler_dir.join("HandlersC.t.sol").exists());

        Ok(())
    }

    #[test]
    fn test_create_contracts_empty() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let contract_type = ContractType::Handler;
        let count = 0;

        let contracts = create_contracts(
            &contract_type,
            count,
            &temp_dir.path().join(contract_type.directory_name())
        )?;

        // Check that no contracts were created
        assert!(contracts.is_empty());

        // Verify the directory was still created
        let handler_dir = temp_dir.path().join("handlers");
        assert!(handler_dir.exists());
        assert!(handler_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_generate_parents() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 1,
        };

        // Test Handler parents
        let handler_parents = generate_parents(
            ContractType::Handler,
            &args,
            &temp_dir.path().join(ContractType::Handler.directory_name()),
        )?;

        assert_eq!(handler_parents.len(), 2);
        assert_eq!(handler_parents[0].name, "HandlersA");
        assert_eq!(handler_parents[1].name, "HandlersB");

        // Test Property parents
        let property_parents = generate_parents(
            ContractType::Property,
            &args,
            &temp_dir.path().join(ContractType::Property.directory_name()),
        )?;

        assert_eq!(property_parents.len(), 1);
        assert_eq!(property_parents[0].name, "PropertiesA");

        Ok(())
    }

    #[test]
    fn test_generate_parents_invalid_type() {
        let temp_dir = TempDir::new().unwrap();
        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 1,
        };

        let result = generate_parents(
            ContractType::Setup,
            &args,
            temp_dir.path(),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid contract type in generate_parents"
        );
    }

    // All the move_temp_contents are in serial to avoid having race conditions
    #[test]
    #[serial]
    fn test_move_temp_contents() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        
        // Create a test file in temp directory
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content")?;

        let result = move_temp_contents(&temp_dir, true);
        assert!(result.is_ok());

        let dest_file = Path::new("./test/invariants/fuzz/test.txt");
        assert!(dest_file.exists());
        assert_eq!(std::fs::read_to_string(dest_file)?, "test content");

        std::fs::remove_dir_all("./test/invariants/fuzz")?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_move_temp_contents_no_overwrite() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        
        std::fs::create_dir_all("./test/invariants/fuzz")?;
        
        let result = move_temp_contents(&temp_dir, false);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Fuzz test folder already exists, did you mean --overwrite ?"
        );

        std::fs::remove_dir_all("./test/invariants/fuzz")?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_move_temp_contents_new_directory() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        
        // source directory with test file
        let source_dir = temp_dir.path().join("source");
        std::fs::create_dir(&source_dir)?;
        let test_file = source_dir.join("test.txt");
        std::fs::write(&test_file, "test content")?;

        // TempDir for source that will be moved
        let source_temp = TempDir::new_in(&source_dir).context("Failed to create source temp dir")?;
        std::fs::write(source_temp.path().join("test.txt"), "test content")?;

        // Set current directory to temp_dir
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;

        // Test moving to non-existent directory
        let result = move_temp_contents(&source_temp, false);
        if let Err(ref e) = result {
            println!("Error: {:#}", e);
        }
        assert!(result.is_ok());

        let fuzz_dir = Path::new("./test/invariants/fuzz");
        assert!(fuzz_dir.exists());
        assert!(fuzz_dir.is_dir());
        let dest_file = fuzz_dir.join("test.txt");
        assert!(dest_file.exists());
        assert_eq!(std::fs::read_to_string(dest_file)?, "test content");

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_generate_test_suite() -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;

        let args = Args {
            overwrite: true,
            solc: "0.8.23".to_string(),
            nb_handlers: 2,
            nb_properties: 1,
        };

        let result = generate_test_suite(&args);
        assert!(result.is_ok());

        let fuzz_dir = Path::new("test/invariants/fuzz");
        assert!(fuzz_dir.join("handlers/HandlersA.t.sol").exists());
        assert!(fuzz_dir.join("handlers/HandlersB.t.sol").exists());
        assert!(fuzz_dir.join("handlers/HandlersParent.t.sol").exists());
        assert!(fuzz_dir.join("properties/PropertiesA.t.sol").exists());
        assert!(fuzz_dir.join("properties/PropertiesParent.t.sol").exists());
        assert!(fuzz_dir.join("Setup.t.sol").exists());
        assert!(fuzz_dir.join("FuzzTest.t.sol").exists());

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }
}
