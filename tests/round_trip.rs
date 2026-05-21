use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply as FrameReply, RequestPayload,
    SessionEpoch, SubReply,
};
use signal_sema_upgrade::{
    Attempt, Completion, ComponentName, Frame, FrameBody, Inspection, InspectionReported,
    MigrationIdentifier, Operation, Rejection, RejectionReason, Reply, ReportQuery,
    SupportedMigration, Version,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn component() -> ComponentName {
    ComponentName::new("persona-spirit")
}

fn source() -> Version {
    Version::new(0, 1, 0)
}

fn target() -> Version {
    Version::new(0, 1, 1)
}

fn migration() -> MigrationIdentifier {
    MigrationIdentifier::new("persona-spirit-0-1-0-to-0-1-1")
}

fn supported_migration() -> SupportedMigration {
    SupportedMigration {
        component: component(),
        source: source(),
        target: target(),
        identifier: migration(),
    }
}

fn attempt() -> Attempt {
    Attempt {
        component: component(),
        source: source(),
        target: target(),
    }
}

fn completion() -> Completion {
    Completion {
        component: component(),
        source: source(),
        target: target(),
        migration: migration(),
        changed_records: 0,
    }
}

fn round_trip_request(operation: Operation) -> Operation {
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: operation.clone().into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request frame, got {other:?}"),
    }
}

fn round_trip_reply(reply: Reply) -> Reply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: FrameReply::committed(NonEmpty::single(SubReply::Ok(reply.clone()))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            FrameReply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted frame reply, got {other:?}"),
        },
        other => panic!("expected reply frame, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota");
    assert_eq!(recovered, value);
    assert!(
        CANONICAL.contains(expected),
        "examples/canonical.nota missing line: {expected}"
    );
}

#[test]
fn requests_round_trip_through_signal_frames() {
    let operations = [
        Operation::Inspect(Inspection::All),
        Operation::Inspect(Inspection::Component(component())),
        Operation::AttemptUpgrade(attempt()),
        Operation::Report(ReportQuery::Component(component())),
    ];

    for operation in operations {
        assert_eq!(round_trip_request(operation.clone()), operation);
    }
}

#[test]
fn replies_round_trip_through_signal_frames() {
    let replies = [
        Reply::InspectionReported(InspectionReported {
            migrations: vec![supported_migration()],
        }),
        Reply::UpgradeCompleted(completion()),
        Reply::UpgradeRejected(Rejection {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: RejectionReason::UnsupportedMigration,
        }),
        Reply::Reported(signal_sema_upgrade::Reported {
            completions: vec![completion()],
            rejections: vec![],
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn contract_owned_operation_kind_is_generated() {
    assert_eq!(
        Operation::Inspect(Inspection::All).kind(),
        signal_sema_upgrade::OperationKind::Inspect
    );
    assert_eq!(
        Operation::AttemptUpgrade(attempt()).kind(),
        signal_sema_upgrade::OperationKind::AttemptUpgrade
    );
}

#[test]
fn canonical_nota_examples_round_trip() {
    round_trip_nota(Operation::Inspect(Inspection::All), "(Inspect All)");
    round_trip_nota(
        Operation::Inspect(Inspection::Component(component())),
        "(Inspect (Component persona-spirit))",
    );
    round_trip_nota(
        Operation::AttemptUpgrade(attempt()),
        "(AttemptUpgrade (persona-spirit (0 1 0) (0 1 1)))",
    );
    round_trip_nota(
        Operation::Report(ReportQuery::Component(component())),
        "(Report (Component persona-spirit))",
    );
    round_trip_nota(
        Reply::InspectionReported(InspectionReported {
            migrations: vec![supported_migration()],
        }),
        "(InspectionReported ([(persona-spirit (0 1 0) (0 1 1) persona-spirit-0-1-0-to-0-1-1)]))",
    );
    round_trip_nota(
        Reply::UpgradeCompleted(completion()),
        "(UpgradeCompleted (persona-spirit (0 1 0) (0 1 1) persona-spirit-0-1-0-to-0-1-1 0))",
    );
    round_trip_nota(
        Reply::UpgradeRejected(Rejection {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: RejectionReason::UnsupportedMigration,
        }),
        "(UpgradeRejected (persona-spirit (0 1 0) (0 1 2) UnsupportedMigration))",
    );
}
