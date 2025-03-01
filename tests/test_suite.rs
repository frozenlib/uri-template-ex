use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uri_template_ex::UriTemplate;

fn load_test_suite(file_name: &str) -> TestSuite {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("external");
    path.push("uritemplate-test");
    path.push(file_name);

    let json =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {}: {}", file_name, e));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("Failed to parse {}: {}", file_name, e))
}

/// Guess the level from the section name
#[allow(clippy::if_same_then_else)]
fn guess_level(section_name: &str) -> u8 {
    if section_name.contains("Level 1") || section_name.contains("3.2.2") {
        1
    } else if section_name.contains("Level 2") || section_name.contains("3.2.3") {
        2
    } else if section_name.contains("Level 3")
        || section_name.contains("3.2.4")
        || section_name.contains("3.2.5")
        || section_name.contains("3.2.6")
        || section_name.contains("3.2.9")
    // Form-Style Query Continuation is Level 3
    {
        3
    } else if section_name.contains("Level 4")
        || section_name.contains("3.2.7")
        || section_name.contains("3.2.8")
    {
        4
    } else if section_name == "Failure Tests" {
        4 // Failure Tests is treated as a special case
    } else {
        1 // Default to Level 1
    }
}

/// Check if the value type is supported in Level 2
fn is_level2_supported_value(value: &VariableValue) -> bool {
    matches!(
        value,
        VariableValue::String(_) | VariableValue::Number(_) | VariableValue::Null
    )
}

/// Extract variable names used in the template
fn extract_variable_names(template: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_var = false;
    let mut start = 0;

    for (i, c) in template.chars().enumerate() {
        match c {
            '{' => {
                in_var = true;
                start = i + 1;
            }
            '}' => {
                if in_var {
                    let var_part = &template[start..i];
                    // Remove modifiers (+, #, ., /, ;, ?, &)
                    let name = var_part.trim_start_matches(|c| "#+.;/?&".contains(c));
                    names.push(name.to_string());
                    in_var = false;
                }
            }
            _ => {}
        }
    }
    names
}

#[test]
fn check_all_test_suite() {
    let test_files = [
        "spec-examples.json",
        "spec-examples-by-section.json",
        "extended-tests.json",
        "negative-tests.json",
    ];

    for file_name in test_files {
        let test_suite = load_test_suite(file_name);
        println!("Testing {}", file_name);

        for (section_name, section) in test_suite.0.iter() {
            // Determine the section level
            let level = if section.level > 0 {
                section.level
            } else {
                guess_level(section_name)
            };

            // Skip tests for level 3 and above
            if level > 2 {
                println!("Skipping {} (level {})", section_name, level);
                continue;
            }

            println!("  Testing section: {} (level {})", section_name, level);
            for test in &section.testcases {
                let template = match UriTemplate::new(&test.template) {
                    Ok(t) => t,
                    Err(e) => {
                        // For Failure Tests section, errors are expected
                        if section_name == "Failure Tests" {
                            if let ExpectedValue::Bool(false) = test.expected {
                                // If an error is expected and actually occurs, it's a success
                                continue;
                            }
                        }
                        panic!("Failed to parse template '{}': {}", test.template, e);
                    }
                };

                // Skip templates using features beyond Level 2
                if template.to_string().contains(',') || // Multiple variable expansion (Level 3)
                   template.to_string().contains('*') || // Variable expansion modifier (Level 4)
                   template.to_string().contains(':') || // Prefix modifier (Level 4)
                   template.to_string().contains('?') || // Query parameter expansion (Level 3)
                   template.to_string().contains('&') || // Query parameter continuation (Level 3)
                   template.to_string().contains(';') || // Semicolon-prefixed parameters (Level 3)
                   template.to_string().contains('#') || // Fragment identifier (Level 3)
                   template.to_string().contains('.')
                // Dot-prefixed labels (Level 3)
                {
                    println!(
                        "    Skipping template: {} (requires level > 2)",
                        test.template
                    );
                    continue;
                }

                // Check only variables used in the template
                let var_names = extract_variable_names(&test.template);
                let has_unsupported_var = var_names.iter().any(|name| {
                    if let Some(value) = section.variables.get(name) {
                        !is_level2_supported_value(value)
                    } else {
                        false // Undefined variables are allowed (treated as empty strings)
                    }
                });

                if has_unsupported_var {
                    println!(
                        "    Skipping template: {} (uses unsupported variable types)",
                        test.template
                    );
                    continue;
                }

                let mut vars = HashMap::new();
                for (k, v) in section.variables.iter() {
                    if var_names.contains(k) {
                        let value = match v {
                            VariableValue::String(s) => s.clone(),
                            VariableValue::Number(n) => n.to_string(),
                            VariableValue::Null => String::new(),
                            _ => unreachable!("Already filtered out unsupported types"),
                        };
                        vars.insert(k.clone(), value);
                    }
                }

                let expanded = template.expand(&vars);

                match &test.expected {
                    ExpectedValue::String(expected) => {
                        assert_eq!(
                            expanded, *expected,
                            "Template '{}' with variables {:#?} expanded to '{}', expected '{}'",
                            test.template, section.variables, expanded, expected
                        );
                    }
                    ExpectedValue::Array(_) => {
                        println!(
                            "    Skipping template: {} (array result is not expected in level 2)",
                            test.template
                        );
                        continue;
                    }
                    ExpectedValue::Bool(expected) => {
                        if *expected {
                            // For success test cases, the expansion result should match the expected value
                            // However, this case is not handled in the current implementation
                            println!("Warning: Unhandled success test case: {}", test.template);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuite(pub HashMap<String, TestSection>);

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSection {
    #[serde(default)]
    pub level: u8,
    pub variables: HashMap<String, VariableValue>,
    pub testcases: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    /// String value
    /// Example: `"var": "value"`, `"hello": "Hello World!"`
    /// [spec-examples.json#L5-L8](../external/uritemplate-test/spec-examples.json#L5-L8)
    String(String),

    /// Number (integer or floating point)
    /// Example: `"number": 6`, `"long": 37.76`, `"lat": -122.427`
    /// [extended-tests.json#L13-L15](../external/uritemplate-test/extended-tests.json#L13-L15)
    Number(f64),

    /// Array of strings
    /// Example: `"list": ["red", "green", "blue"]`, `"geocode": ["37.76","-122.427"]`
    /// [spec-examples.json#L82-L83](../external/uritemplate-test/spec-examples.json#L82-L83)
    Array(Vec<String>),

    /// Map with string keys and values
    /// Example: `"keys": { "semi": ";", "dot": ".", "comma": "," }`
    /// [spec-examples.json#L83](../external/uritemplate-test/spec-examples.json#L83)
    Object(HashMap<String, String>),

    /// Null value
    /// Example: `"undef": null`
    /// [spec-examples-by-section.json#L29](../external/uritemplate-test/spec-examples-by-section.json#L29)
    Null,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    #[serde(rename = "0")]
    pub template: String,
    #[serde(rename = "1")]
    pub expected: ExpectedValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpectedValue {
    /// String value
    /// Example: `["{var}", "value"]`
    /// [spec-examples.json#L10](../external/uritemplate-test/spec-examples.json#L10)
    String(String),

    /// Boolean (indicates test success/failure)
    /// Example: `["{/id*}", false]`
    /// [negative-tests.json#L7](../external/uritemplate-test/negative-tests.json#L7)
    Bool(bool),

    /// Array of strings (represents multiple possible expansion results)
    /// Example: `["{keys}", ["comma,%2C,dot,.,semi,%3B", "comma,%2C,semi,%3B,dot,.", ...]]`
    /// [spec-examples.json#L82-L89](../external/uritemplate-test/spec-examples.json#L82-L89)
    Array(Vec<String>),
}

mod json_reading_test {
    use super::*;

    fn verify_variable_value(value: &VariableValue, expected: &str) {
        if let VariableValue::String(s) = value {
            assert_eq!(s, expected);
        } else {
            panic!("Variable value should be a string");
        }
    }

    #[test]
    fn test_parse_all_json_files() {
        let test_files = [
            "spec-examples.json",
            "spec-examples-by-section.json",
            "extended-tests.json",
            "negative-tests.json",
        ];

        for file_name in test_files {
            let test_suite = load_test_suite(file_name);
            assert!(
                !test_suite.0.is_empty(),
                "Test suite {} should not be empty",
                file_name
            );

            for (section_name, section) in test_suite.0.iter() {
                // Verify level (only if level field is present)
                if section.level > 0 {
                    assert!(
                        section.level <= 4,
                        "Level should be between 1 and 4 in {} section {}",
                        file_name,
                        section_name
                    );
                }

                // Verify variables
                assert!(
                    !section.variables.is_empty(),
                    "Variables should not be empty in {} section {}",
                    file_name,
                    section_name
                );

                // Verify test cases
                assert!(
                    !section.testcases.is_empty(),
                    "Test cases should not be empty in {} section {}",
                    file_name,
                    section_name
                );

                // Verify each test case
                for test in &section.testcases {
                    assert!(
                        !test.template.is_empty(),
                        "Template should not be empty in {} section {}",
                        file_name,
                        section_name
                    );

                    match &test.expected {
                        ExpectedValue::String(s) => assert!(
                            !s.is_empty(),
                            "Expected string should not be empty in {} section {}",
                            file_name,
                            section_name
                        ),
                        ExpectedValue::Bool(_) => {
                            assert_eq!(
                                section_name, "Failure Tests",
                                "Boolean expected value should only appear in Failure Tests section"
                            );
                        }
                        ExpectedValue::Array(arr) => assert!(
                            !arr.is_empty(),
                            "Expected array should not be empty in {} section {}",
                            file_name,
                            section_name
                        ),
                    }
                }
            }
        }
    }

    #[test]
    fn test_specific_examples() {
        let test_suite = load_test_suite("spec-examples.json");

        // Verify Level 1 Examples
        let level1 = test_suite
            .0
            .get("Level 1 Examples")
            .expect("Level 1 Examples not found");
        assert_eq!(level1.level, 1);
        verify_variable_value(&level1.variables["var"], "value");

        let first_test = &level1.testcases[0];
        assert_eq!(first_test.template, "{var}");
        assert!(matches!(first_test.expected, ExpectedValue::String(ref s) if s == "value"));

        // Verify Failure Tests
        let test_suite = load_test_suite("negative-tests.json");
        let failure_tests = test_suite
            .0
            .get("Failure Tests")
            .expect("Failure Tests not found");
        assert_eq!(failure_tests.level, 4);
        verify_variable_value(&failure_tests.variables["id"], "thing");

        let first_test = &failure_tests.testcases[0];
        assert_eq!(first_test.template, "{/id*");
        assert!(matches!(first_test.expected, ExpectedValue::Bool(false)));
    }
}
