# uri-template-ex

[![Crates.io](https://img.shields.io/crates/v/uri-template-ex.svg)](https://crates.io/crates/uri-template-ex)
[![Docs.rs](https://docs.rs/uri-template-ex/badge.svg)](https://docs.rs/uri-template-ex/)
[![Actions Status](https://github.com/frozenlib/uri-template-ex/workflows/CI/badge.svg)](https://github.com/frozenlib/uri-template-ex/actions)

Implementation of RFC6570 URI Template Level 2

## Overview

`uri-template-ex` is a crate that implements URL expansion and variable extraction (capture) using URI Template Level 2 as defined in [RFC6570].

Since RFC6570 only defines variable expansion and not extraction, most existing URI Template implementations only support variable expansion and do not support extraction. This crate supports variable extraction using the same syntax as URI Template.

## Features

- Variable expansion using RFC6570 URI Template Level 2
- Variable value extraction from URI templates

URI Template Level 2 supports the following 3 types of variables:

- `{var}`
- `{+var}`
- `{#var}`

## Installation

Add the following to your Cargo.toml:

```toml
[dependencies]
uri-template-ex = "0.0.2"
```

## Usage

### Basic Usage Example

Example of generating a URI using a URI template:

```rust
use std::collections::BTreeMap;
use uri_template_ex::UriTemplate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = UriTemplate::new("/users/{a}/files/{b}")?;
    let mut vars = BTreeMap::new();
    vars.insert("a", "xxx");
    vars.insert("b", "hello-world");

    let uri = template.expand(&vars);
    assert_eq!(uri, "/users/xxx/files/hello-world");
    Ok(())
}
```

### Value Extraction from URI

Example of extracting values that match a template from a URI:

```rust
use uri_template_ex::UriTemplate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = UriTemplate::new("/users/{a}/files/{b}")?;
    let uri = "/users/xxx/files/hello-world";

    if let Some(captures) = template.captures(uri) {
        if let Some(a) = captures.name("a") {
            println!("a: {}", a.value()?);  // "xxx"
        }
        if let Some(b) = captures.name("b") {
            println!("b: {}", b.value()?);  // "hello-world"
        }
    }
    Ok(())
}
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[RFC6570]: https://datatracker.ietf.org/doc/html/rfc6570
