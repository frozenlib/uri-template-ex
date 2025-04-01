// #![include_doc("../README.ja.md", start)]
//! # uri-template-ex
//!
//! [![Crates.io](https://img.shields.io/crates/v/uri-template-ex.svg)](https://crates.io/crates/uri-template-ex)
//! [![Docs.rs](https://docs.rs/uri-template-ex/badge.svg)](https://docs.rs/uri-template-ex/)
//! [![Actions Status](https://github.com/frozenlib/uri-template-ex/workflows/CI/badge.svg)](https://github.com/frozenlib/uri-template-ex/actions)
//!
//! RFC6570 URI Template Level 2 の実装
//!
//! ## 概要
//!
//! `uri-template-ex`は、[RFC6570] で定義されている URI Template Level 2 による URL の展開と変数の抽出（キャプチャ）を実装した crate です。
//!
//! RFC6570 は変数の展開のみを定義しており抽出は定義していないこともあり、既存の URI Template の実装は変数の展開のみをサポートし、抽出はサポートしないものがほとんどです。この crate では URI Tempalte を同じ構文による変数の抽出もサポートします。
//!
//! ## 特徴
//!
//! - RFC6570 URI Template Level 2 による変数の展開
//! - URI テンプレートからの変数値の抽出
//!
//! URI Template Level 2 では下記の 3 種類の変数を使用できます。
//!
//! - `{var}`
//! - `{+var}`
//! - `{#var}`
//!
//! ## インストール
//!
//! Cargo.toml に以下を追加してください：
//!
//! ```toml
//! [dependencies]
//! uri-template-ex = "0.0.2"
//! ```
//!
//! ## 使用方法
//!
//! ### 基本的な使用例
//!
//! URI テンプレートを使用して URI を生成する例：
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use uri_template_ex::UriTemplate;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let template = UriTemplate::new("/users/{a}/files/{b}")?;
//!     let mut vars = BTreeMap::new();
//!     vars.insert("a", "xxx");
//!     vars.insert("b", "hello-world");
//!
//!     let uri = template.expand(&vars);
//!     assert_eq!(uri, "/users/xxx/files/hello-world");
//!     Ok(())
//! }
//! ```
//!
//! ### URI からの値の抽出
//!
//! URI からテンプレートにマッチする値を抽出する例：
//!
//! ```rust
//! use uri_template_ex::UriTemplate;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let template = UriTemplate::new("/users/{a}/files/{b}")?;
//!     let uri = "/users/xxx/files/hello-world";
//!
//!     if let Some(captures) = template.captures(uri) {
//!         if let Some(a) = captures.name("a") {
//!             println!("a: {}", a.value()?);  // "xxx"
//!         }
//!         if let Some(b) = captures.name("b") {
//!             println!("b: {}", b.value()?);  // "hello-world"
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## ライセンス
//!
//! This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
//!
//! [RFC6570]: https://datatracker.ietf.org/doc/html/rfc6570
// #![include_doc("../README.ja.md", end)]
