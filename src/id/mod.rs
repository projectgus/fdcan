//! CAN Identifiers
mod api;
mod internal;

pub type Id = api::Id;
pub type StandardId = api::StandardId;
pub type ExtendedId = api::ExtendedId;

pub(crate) type IdReg = internal::IdReg;
