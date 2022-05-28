//! The store of named records.

use ink_prelude::string::String;
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    traits,
};

use crate::cns::{Error::*, Result};

use super::entity::Record;

pub type Name = String;

pub const NAME_MAX_LEN: usize = 100;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NameStore(pub HashMap<Name, Record>);

impl NameStore {
    pub fn create(&mut self, name: Name, record: Record) -> Result<()> {
        if name.len() > NAME_MAX_LEN {
            return Err(NameTooLong);
        }
        match name.chars().next() {
            Some(char) if char.is_ascii_alphabetic() => {}
            _ => return Err(NameMustStartWithALetter),
        };

        match self.0.entry(name) {
            Occupied(_) => Err(NameAlreadyTaken),

            Vacant(e) => {
                e.insert(record);
                Ok(())
            }
        }
    }

    pub fn get(&self, name: &Name) -> Result<&Record> {
        self.0.get(name).ok_or(NameDoesNotExist)
    }

    pub fn get_mut(&mut self, name: &Name) -> Result<&mut Record> {
        self.0.get_mut(name).ok_or(NameDoesNotExist)
    }
}
