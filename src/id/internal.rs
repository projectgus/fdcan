/// Private identifier-related types.
use core::cmp::{Ord, Ordering};
use crate::message_ram::enums::{IdType, RemoteTransmissionRequest};
use crate::id::{Id, StandardId, ExtendedId};

/// Identifier of a CAN message.
///
/// FdCan be either a standard identifier (11bit, Range: 0..0x7FF) or a
/// extendended identifier (29bit , Range: 0..0x1FFFFFFF).
///
/// The `Ord` trait can be used to determine the frame’s priority this ID
/// belongs to.
/// Lower identifier values have a higher priority. Additionally standard frames
/// have a higher priority than extended frames and data frames have a higher
/// priority than remote frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IdReg(u32);

impl IdReg {
    const STANDARD_SHIFT: u32 = 18;
    const STANDARD_MASK: u32 = 0x1FFC0000;

    const EXTENDED_SHIFT: u32 = 0;
    const EXTENDED_MASK: u32 = 0x1FFFFFFF;

    const XTD_SHIFT: u32 = 30;
    const XTD_MASK: u32 = 1 << Self::XTD_SHIFT;

    const RTR_SHIFT: u32 = 29;
    const RTR_MASK: u32 = 1 << Self::RTR_SHIFT;

    /// Creates a new standard identifier (11bit, Range: 0..0x7FF)
    ///
    /// Panics for IDs outside the allowed range.
    fn new_standard(id: StandardId) -> Self {
        Self(u32::from(id.as_raw()) << Self::STANDARD_SHIFT)
    }

    /// Creates a new extendended identifier (29bit , Range: 0..0x1FFFFFFF).
    ///
    /// Panics for IDs outside the allowed range.
    fn new_extended(id: ExtendedId) -> Self {
        Self(id.as_raw() << Self::EXTENDED_SHIFT | (1 << Self::XTD_SHIFT))
    }

    pub(crate) fn as_raw_id(&self) -> u32 {
        self.0 & Self::EXTENDED_MASK
    }

    pub(crate) fn from_register(
        id: u32,
        rtr: RemoteTransmissionRequest,
        xtd: IdType,
    ) -> Self {
        let rtr: u32 = match rtr {
            RemoteTransmissionRequest::TransmitDataFrame => 0,
            RemoteTransmissionRequest::TransmitRemoteFrame => 1 << Self::RTR_SHIFT,
        };
        let xtd: u32 = match xtd {
            IdType::StandardId => 0,
            IdType::ExtendedId => 1 << Self::XTD_SHIFT,
        };
        Self(id | rtr | xtd)
    }

    /// Sets the remote transmission (RTR) flag. This marks the identifier as
    /// being part of a remote frame.
    #[must_use = "returns a new IdReg without modifying `self`"]
    pub(crate) fn with_rtr(self, rtr: bool) -> Self {
        if rtr {
            Self(self.0 | (1 << Self::RTR_SHIFT))
        } else {
            Self(self.0 & !Self::RTR_MASK)
        }
    }

    /// Returns the identifier.
    pub fn to_id(self) -> Id {
        if self.is_extended() {
            Id::Extended(unsafe {
                ExtendedId::new_unchecked(
                    (self.0 & Self::EXTENDED_MASK) >> Self::EXTENDED_SHIFT,
                )
            })
        } else {
            Id::Standard(unsafe {
                StandardId::new_unchecked(
                    ((self.0 & Self::STANDARD_MASK) >> Self::STANDARD_SHIFT) as u16,
                )
            })
        }
    }

    /// Returns `true` if the identifier is an extended identifier.
    pub fn is_extended(self) -> bool {
        (self.0 & Self::XTD_MASK) != 0
    }

    /// Returns `true` if the identifier is a standard identifier.
    pub fn is_standard(self) -> bool {
        !self.is_extended()
    }

    /// Returns `true` if the identifer is part of a remote frame (RTR bit set).
    pub(crate) fn rtr(self) -> bool {
        self.0 & Self::RTR_MASK != 0
    }
}
impl From<Id> for IdReg {
    fn from(id: Id) -> Self {
        match id {
            Id::Standard(s) => IdReg::new_standard(s),
            Id::Extended(e) => IdReg::new_extended(e),
        }
    }
}
impl From<IdReg> for Id {
    fn from(idr: IdReg) -> Self {
        idr.to_id()
    }
}
impl From<IdReg> for IdType {
    #[inline]
    fn from(id: IdReg) -> Self {
        if id.is_standard() {
            IdType::StandardId
        } else {
            IdType::ExtendedId
        }
    }
}
impl From<IdReg> for RemoteTransmissionRequest {
    #[inline]
    fn from(id: IdReg) -> Self {
        if id.rtr() {
            RemoteTransmissionRequest::TransmitRemoteFrame
        } else {
            RemoteTransmissionRequest::TransmitDataFrame
        }
    }
}

/// `IdReg` is ordered by priority.
impl Ord for IdReg {
    fn cmp(&self, other: &Self) -> Ordering {
        // When the IDs match, data frames have priority over remote frames.
        let rtr = self.rtr().cmp(&other.rtr()).reverse();

        let id_a = self.to_id();
        let id_b = other.to_id();
        match (id_a, id_b) {
            (Id::Standard(a), Id::Standard(b)) => {
                // Lower IDs have priority over higher IDs.
                a.as_raw().cmp(&b.as_raw()).reverse().then(rtr)
            }
            (Id::Extended(a), Id::Extended(b)) => {
                a.as_raw().cmp(&b.as_raw()).reverse().then(rtr)
            }
            (Id::Standard(a), Id::Extended(b)) => {
                // Standard frames have priority over extended frames if their Base IDs match.
                a.as_raw()
                    .cmp(&b.standard_id().as_raw())
                    .reverse()
                    .then(Ordering::Greater)
            }
            (Id::Extended(a), Id::Standard(b)) => a
                .standard_id()
                .as_raw()
                .cmp(&b.as_raw())
                .reverse()
                .then(Ordering::Less),
        }
    }
}

impl PartialOrd for IdReg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Id> for IdType {
    #[inline]
    fn from(id: Id) -> Self {
        match id {
            Id::Standard(id) => id.into(),
            Id::Extended(id) => id.into(),
        }
    }
}

impl From<ExtendedId> for IdType {
    fn from(_id: ExtendedId) -> Self {
        IdType::ExtendedId
    }
}

impl From<StandardId> for IdType {
    fn from(_id: StandardId) -> Self {
        IdType::StandardId
    }
}
