mod cli;
mod gen;
mod types;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::Args;
use crate::gen::{generate_contract, generate_family};
use crate::types::ContractType;

fn main() -> Result<()> {
    let args = Args::parse();

    generate_family(&args, ContractType::Handler)
        .context("Failed to generate handlers contracts")?;

    generate_family(&args, ContractType::Property)
        .context("Failed to generated properties contracts")?;

    generate_contract(&args, ContractType::EntryPoint)
        .context("Failed to generate entry point contract")?;

    generate_contract(&args, ContractType::Setup).context("Failed to generate setup contract")?;

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
