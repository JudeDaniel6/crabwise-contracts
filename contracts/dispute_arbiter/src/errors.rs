use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DisputeError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NotFound = 4,
    /// No resolution outcome can be applied because the dispute is not `Open`
    /// (#42). Returned by `resolve_for_client`, `resolve_for_artist`,
    /// `partial_resolve` and `auto_resolve` once an outcome has been recorded.
    InvalidStatus = 5,
    /// `open_dispute` was called for a commission that already has a dispute.
    AlreadyResolved = 6,
    AutoResolveNotDue = 7,
    /// The client share of a partial resolution is above `10_000` bps (100 %),
    /// so the requested split is not an outcome the contract can pay out (#42).
    InvalidShareBps = 8,
    ArithmeticOverflow = 9,
    /// The resolution note exceeds `shared::validation::MAX_MEMO_LEN` (#765),
    /// so the resolution outcome cannot be recorded with it (#42). The note is
    /// written to persistent storage, so it has to be bounded before the
    /// write, not after. Appended, never renumbered: error code values are
    /// permanent per `docs/UPGRADE_AND_ROLLBACK.md`.
    NoteTooLong = 10,
    /// Evidence arrived after the dispute's deadline (#977).
    EvidenceWindowClosed = 11,
    /// A submitted evidence blob exceeds `MAX_EVIDENCE_LEN` (#977).
    EvidenceTooLong = 12,
    /// The configured dispute TTL is zero or above the documented maximum
    /// (#977).
    InvalidTtl = 13,
}

impl core::fmt::Display for DisputeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::AlreadyInitialized => write!(f, "already initialized"),
            Self::NotInitialized => write!(f, "not initialized"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "dispute not found"),
            Self::InvalidStatus => write!(
                f,
                "invalid outcome: the dispute is not open for resolution"
            ),
            Self::AlreadyResolved => write!(f, "a dispute already exists for this commission"),
            Self::AutoResolveNotDue => write!(f, "auto-resolve not yet due"),
            Self::InvalidShareBps => write!(
                f,
                "invalid outcome: client share must be between 0 and 10000 bps"
            ),
            Self::ArithmeticOverflow => write!(f, "arithmetic operation would overflow"),
            Self::NoteTooLong => write!(
                f,
                "invalid outcome: resolution note exceeds the maximum length"
            ),
            Self::EvidenceWindowClosed => write!(f, "the evidence window has closed"),
            Self::EvidenceTooLong => write!(f, "evidence exceeds maximum length"),
            Self::InvalidTtl => write!(f, "dispute TTL is zero or above the maximum"),
        }
    }
}

pub fn get_suggestion(error: DisputeError) -> Symbol {
    match error {
        DisputeError::AlreadyInitialized => symbol_short!("DUP"),
        DisputeError::NotInitialized => symbol_short!("NO_INIT"),
        DisputeError::Unauthorized => symbol_short!("AUTH"),
        DisputeError::NotFound => symbol_short!("NOT_FOUND"),
        DisputeError::InvalidStatus => symbol_short!("BAD_STS"),
        DisputeError::AlreadyResolved => symbol_short!("RESOLVED"),
        DisputeError::AutoResolveNotDue => symbol_short!("NOT_DUE"),
        DisputeError::InvalidShareBps => symbol_short!("BAD_BPS"),
        DisputeError::ArithmeticOverflow => symbol_short!("OVERFL"),
        DisputeError::NoteTooLong => symbol_short!("LONG_NOTE"),
        DisputeError::EvidenceWindowClosed => symbol_short!("EVID_CLSD"),
        DisputeError::EvidenceTooLong => symbol_short!("EVID_LONG"),
        DisputeError::InvalidTtl => symbol_short!("BAD_TTL"),
    }
}
