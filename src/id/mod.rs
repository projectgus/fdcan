//! CAN Identifiers
//!
//! If the embedded_can feature is enabled then the public Id types
//! are all aliased to the respective embedded_can types.

#[cfg(not(feature = "embedded_can"))]
mod api;

mod internal;

#[cfg(feature = "embedded_can")]
mod api {
    pub type Id = embedded_can::Id;
    pub type StandardId = embedded_can::StandardId;
    pub type ExtendedId = embedded_can::ExtendedId;
}

pub type Id = api::Id;
pub type StandardId = api::StandardId;
pub type ExtendedId = api::ExtendedId;

pub(crate) type IdReg = internal::IdReg;
