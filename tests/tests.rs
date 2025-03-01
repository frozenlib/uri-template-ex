use std::collections::BTreeMap;

use uri_template_ex::{Error, UriTemplate};

type Result<T> = std::result::Result<T, Error>;

#[test]
fn no_var() -> Result<()> {
    check_none("http://a/")?;
    check_none("http://%E3%81%82")?; // precent-encoding
    check_none("http://%e3%81%82")?; // precent-encoding (lowercase)
    check_expand("http://あ", "http://%E3%81%82", &[])?; // non-ascii

    // only one hex digit
    check_expand("%2", "%252", &[])?;
    check_expand("%2/", "%252/", &[])?;

    // only percent sign
    check_expand("%", "%25", &[])?;
    check_expand("%/", "%25/", &[])?;

    // invalid hex digit(G)
    check_expand("%2G", "%252G", &[])?;
    check_expand("%2G/", "%252G/", &[])?;

    // invalid hex digit(X, Y)
    check_expand("%XY", "%25XY", &[])?;
    check_expand("%XY/", "%25XY/", &[])?;

    // invalid utf-8 sequence
    check_none("%F8%28")?;
    check_none("%F8%28/")?;

    check_expand("http://%E3%81%82", "http://%E3%81%82", &[])?; // precent-encoding
    check_expand("http://%e3%81%82", "http://%e3%81%82", &[])?; // precent-encoding (lowercase)
    check_expand("http://あ", "http://%E3%81%82", &[])?; // non-ascii

    // overlong encoding
    check_none("%C0%A0")?;
    check_none("%C0%A0/")?;

    // incomplete utf-8 sequence
    check_expand("%E4%B8%", "%E4%B8%25", &[])?;
    check_expand("%E4%B8%/", "%E4%B8%25/", &[])?;

    // incomplete multibyte character
    check_none("%D0")?;
    check_none("%D0/")?;

    Ok(())
}

#[test]
fn err() {
    // invalid variable
    check_err("{aaa");
}

#[test]
fn simple_expansion() -> Result<()> {
    check_both("http://a/{b}", "http://a/xxx", &[("b", "xxx")])?; // unreserved
    check_both("http://a/{b}", "http://a/%2F", &[("b", "/")])?; // reserved
    check_both(
        // percent-encoding
        "http://a/{b}",
        "http://a/%25E3%2581%2582",
        &[("b", "%E3%81%82")],
    )?;
    check_both("http://a/{b}", "http://a/%2525", &[("b", "%25")])?; // percent-encoding
    check_both("http://a/{b}", "http://a/%E3%81%82", &[("b", "あ")])?; // non-ascii
    check_both("http://a/{b}", "http://a/", &[("b", "")])?; // empty
    check_both("http://a/{b}/c", "http://a/xxx/c", &[("b", "xxx")])?;
    check_not_match("http://a/{b}/c", "http://a/xxx/yyy/c")?;
    Ok(())
}

#[test]
fn reserved_expansion() -> Result<()> {
    check_both("http://a/{+b}", "http://a/xxx", &[("b", "xxx")])?; // unreserved
    check_both("http://a/{+b}", "http://a//", &[("b", "/")])?; // reserved
    check_both(
        // percent-encoding
        "http://a/{+b}",
        "http://a/%E3%81%82",
        &[("b", "%E3%81%82")],
    )?;
    check_both("http://a/{+b}", "http://a/%25", &[("b", "%25")])?; // percent-encoding
    check_both("http://a/{+b}", "http://a/%E3%81%82", &[("b", "%E3%81%82")])?; // non-ascii
    check_both("http://a/{+b}", "http://a/", &[("b", "")])?; // empty
    check_both("http://a/{+b}/c", "http://a/xxx/yyy/c", &[("b", "xxx/yyy")])?;
    Ok(())
}

#[test]
fn fragment_expansion() -> Result<()> {
    check_both("http://a/{#b}", "http://a/#xxx", &[("b", "xxx")])?; // unreserved
    check_both("http://a/{#b}", "http://a/#/", &[("b", "/")])?; // reserved
    check_both(
        "http://a/{#b}",
        "http://a/#%E3%81%82",
        &[("b", "%E3%81%82")],
    )?; // non-ascii
    check_both("http://a/{#b}", "http://a/#%25", &[("b", "%25")])?; // percent-encoding
    check_both(
        "http://a/{#b}",
        "http://a/#%E3%81%82",
        &[("b", "%E3%81%82")],
    )?; // non-ascii
    check_both("http://a/{#b}", "http://a/#", &[("b", "")])?; // empty
    check_both("http://a/{#b}", "http://a/", &[])?; // empty
    check_both(
        "http://a/{#b}/c",
        "http://a/#xxx/yyy/c",
        &[("b", "xxx/yyy")],
    )?;
    Ok(())
}

#[track_caller]
fn check_both(template: &str, e: &str, vars: &[(&str, &str)]) -> Result<()> {
    let template = UriTemplate::new(template)?;
    let mut input_vars = BTreeMap::new();
    for (k, v) in vars {
        input_vars.insert(k.to_string(), v.to_string());
    }
    let args = format!("expand: template = `{template}`, input = `{e}`, vars = `{vars:?}`");
    let a = template.expand(&input_vars);
    assert_eq!(a, e, "expand: {args}");
    if let Some(captures) = template.captures(&a) {
        let mut output_vars = BTreeMap::new();
        for (k, v) in captures.iter() {
            if let Some(v) = v {
                output_vars.insert(k.to_string(), v.value()?.to_string());
            }
        }
        assert_eq!(output_vars, input_vars, "capture: {args}");
    } else {
        panic!("failed to capture: {args}");
    }
    Ok(())
}

#[track_caller]
fn check_expand(template: &str, e: &str, vars: &[(&str, &str)]) -> Result<()> {
    let template = UriTemplate::new(template)?;
    let mut input_vars = BTreeMap::new();
    for (k, v) in vars {
        input_vars.insert(k.to_string(), v.to_string());
    }
    let args = format!("expand: template = `{template}`, input = `{e}`, vars = `{vars:?}`");
    let a = template.expand(&input_vars);
    assert_eq!(a, e, "expand: {args}");
    Ok(())
}
#[track_caller]
fn check_none(input: &str) -> Result<()> {
    check_both(input, input, &[])
}

#[track_caller]
fn check_not_match(template: &str, input: &str) -> Result<()> {
    let template = UriTemplate::new(template)?;
    let c = template.captures(input);
    assert!(
        c.is_none(),
        "expect not match, template = `{template}`, input = `{input}`"
    );
    Ok(())
}

#[track_caller]
fn check_err(template: &str) {
    let ret = UriTemplate::new(template);
    assert!(ret.is_err(), "expect error, template = `{template}`");
}
