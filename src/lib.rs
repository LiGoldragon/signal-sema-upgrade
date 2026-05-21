//! Signal contract for the ordinary `sema-upgrade` surface.
//!
//! This crate carries the peer-callable vocabulary for inspecting
//! compiled migration support, attempting a schema upgrade, and reading
//! reports. Runtime migration logic lives in `sema-upgrade`.

use nota_codec::{NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_sema::SemaObservation;

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct ComponentName(String);

impl ComponentName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct MigrationIdentifier(String);

impl MigrationIdentifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SupportedMigration {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub identifier: MigrationIdentifier,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Inspection {
    All,
    Component(ComponentName),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum ReportQuery {
    All,
    Component(ComponentName),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct InspectionReported {
    pub migrations: Vec<SupportedMigration>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub migration: MigrationIdentifier,
    pub changed_records: u64,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum RejectionReason {
    UnsupportedMigration,
    ComponentMismatch,
    MigrationFailed,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub reason: RejectionReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Reported {
    pub completions: Vec<Completion>,
    pub rejections: Vec<Rejection>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum UnimplementedReason {
    NotBuiltYet,
    IntegrationNotLanded,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RequestUnimplemented {
    pub reason: UnimplementedReason,
}

signal_channel! {
    channel SemaUpgrade {
        operation Inspect(Inspection),
        operation AttemptUpgrade(Attempt),
        operation Report(ReportQuery),
    }
    reply Reply {
        InspectionReported(InspectionReported),
        UpgradeCompleted(Completion),
        UpgradeRejected(Rejection),
        Reported(Reported),
        RequestUnimplemented(RequestUnimplemented),
    }
    observable {
        filter default;
        operation_event OperationReceived;
        effect_event EffectEmitted;
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EffectEmitted {
    pub observation: SemaObservation,
}
