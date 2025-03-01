use std::cmp::Eq;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::str;
use std::{borrow::Cow, fmt};

pub trait Vars {
    fn var(&mut self, index: usize, name: &str) -> Option<Cow<str>>;
}
impl Vars for () {
    fn var(&mut self, _index: usize, _name: &str) -> Option<Cow<str>> {
        None
    }
}
impl<K> Vars for &HashMap<K, &str>
where
    K: std::borrow::Borrow<str> + Hash + Eq,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self.get(name)?))
    }
}
impl<K> Vars for &HashMap<K, String>
where
    K: std::borrow::Borrow<str> + Hash + Eq,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self.get(name)?))
    }
}
impl<K> Vars for &HashMap<K, &dyn fmt::Display>
where
    K: std::borrow::Borrow<str> + Hash + Eq,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(self.get(name)?.to_string().into())
    }
}
impl<K> Vars for &BTreeMap<K, &str>
where
    K: std::borrow::Borrow<str> + Ord,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self.get(name)?))
    }
}
impl<K> Vars for &BTreeMap<K, String>
where
    K: std::borrow::Borrow<str> + Ord,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self.get(name)?))
    }
}
impl<K> Vars for &BTreeMap<K, &dyn fmt::Display>
where
    K: std::borrow::Borrow<str> + Ord,
{
    fn var(&mut self, _index: usize, name: &str) -> Option<Cow<str>> {
        Some(self.get(name)?.to_string().into())
    }
}

impl Vars for &[&str] {
    fn var(&mut self, index: usize, _name: &str) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self.get(index)?))
    }
}
impl Vars for &[&dyn fmt::Display] {
    fn var(&mut self, index: usize, _name: &str) -> Option<Cow<str>> {
        Some(Cow::Owned(self.get(index)?.to_string()))
    }
}
