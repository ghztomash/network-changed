use netdev::Interface;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct Interfaces(HashMap<String, Interface>);

impl Interfaces {
    pub fn new(interfaces: Vec<Interface>) -> Self {
        let mut set = HashMap::new();
        for interface in interfaces {
            set.insert(interface.name.to_owned(), interface);
        }
        Self(set)
    }

    pub fn diff(&self, other: &Self) -> InterfacesDiff {
        let lhs = self.0.to_owned();
        let rhs = other.0.to_owned();
        let mut updated = HashMap::new();
        let mut added = HashMap::new();
        let mut removed = HashMap::new();

        for (key, value) in &lhs {
            if !rhs.contains_key(key) {
                removed.insert(key.to_string(), value.clone());
            }
        }

        for (key, new_value) in rhs {
            if let Some(old_value) = lhs.get(&key) {
                if new_value != old_value.clone() {
                    updated.insert(key, new_value);
                }
            } else {
                added.insert(key, new_value);
            }
        }

        InterfacesDiff {
            updated,
            added,
            removed,
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct InterfacesDiff {
    pub updated: HashMap<String, Interface>,
    pub added: HashMap<String, Interface>,
    pub removed: HashMap<String, Interface>,
}

#[cfg(test)]
mod should {
    use super::*;

    fn interface(name: &str) -> Interface {
        let mut interface = Interface::dummy();
        interface.name = name.to_string();
        interface
    }

    fn interface_with_flag(name: &str, flag: u32) -> Interface {
        let mut interface = Interface::dummy();
        interface.name = name.to_string();
        interface.flags = flag;
        interface
    }

    #[test]
    fn diff_same() {
        let old = Interfaces::new(vec![interface("eth0")]);
        let new = Interfaces::new(vec![interface("eth0")]);

        let diff = Interfaces::diff(&old, &new);
        let expected_diff = InterfacesDiff {
            updated: HashMap::new(),
            added: HashMap::new(),
            removed: HashMap::new(),
        };
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn diff_updated() {
        let old = Interfaces::new(vec![interface("eth0")]);
        let new = Interfaces::new(vec![interface_with_flag("eth0", 1)]);
        let diff = Interfaces::diff(&old, &new);
        let expected_diff = InterfacesDiff {
            updated: vec![("eth0".to_string(), interface_with_flag("eth0", 1))]
                .into_iter()
                .collect(),
            added: HashMap::new(),
            removed: HashMap::new(),
        };
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn diff_added() {
        let old = Interfaces::new(vec![interface("eth0")]);
        let new = Interfaces::new(vec![interface("eth0"), interface("eth1")]);
        let diff = Interfaces::diff(&old, &new);
        let expected_diff = InterfacesDiff {
            updated: HashMap::new(),
            added: vec![("eth1".to_string(), interface("eth1"))]
                .into_iter()
                .collect(),
            removed: HashMap::new(),
        };
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn diff_removed() {
        let old = Interfaces::new(vec![interface("eth0"), interface("eth1")]);
        let new = Interfaces::new(vec![interface("eth0")]);
        let diff = Interfaces::diff(&old, &new);
        let expected_diff = InterfacesDiff {
            updated: HashMap::new(),
            added: HashMap::new(),
            removed: vec![("eth1".to_string(), interface("eth1"))]
                .into_iter()
                .collect(),
        };
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn diff_changed() {
        let old = Interfaces::new(vec![interface("eth0"), interface("eth1")]);
        let new = Interfaces::new(vec![interface_with_flag("eth0", 1), interface("eth2")]);
        let diff = Interfaces::diff(&old, &new);
        let expected_diff = InterfacesDiff {
            updated: vec![("eth0".to_string(), interface_with_flag("eth0", 1))]
                .into_iter()
                .collect(),
            added: vec![("eth2".to_string(), interface("eth2"))]
                .into_iter()
                .collect(),
            removed: vec![("eth1".to_string(), interface("eth1"))]
                .into_iter()
                .collect(),
        };
        assert_eq!(diff, expected_diff);
    }
}
