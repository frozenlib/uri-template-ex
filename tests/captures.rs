use uri_template_ex::Captures;

#[test]
fn captures_empty() {
    let empty = Captures::empty();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(empty.iter().next().is_none());
    assert!(empty.name("a").is_none());
    assert!(empty.get(0).is_none());
}
