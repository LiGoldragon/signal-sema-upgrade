# signal-sema-upgrade Architecture

`signal-sema-upgrade` owns the ordinary peer-callable vocabulary for the
`sema-upgrade` component. Runtime state, migration modules, filesystem
access, and daemon policy live in `sema-upgrade`.

## Constraints

- The contract describes upgrade requests; it does not perform upgrades.
- An upgrade attempt names one component and exactly one source version
  plus one target version.
- Supported migrations are explicit records emitted from the daemon's
  compiled migration index.
- Unsupported version pairs must be rejected as typed replies, not hidden
  behind generic frame rejection.
- The contract uses `signal-frame` for frames and `signal-sema` for
  observer classification.

## First Prototype

The first supported migration is the `persona-spirit` state-schema bump
from `0.1.0` to `0.1.1`. The version record is numeric in the signal
contract:

```nota
(AttemptUpgrade (persona-spirit (0 1 0) (0 1 1)))
```

The displayed version may be rendered as `0.1` or `0.1.1` by daemon
tools, but the typed contract keeps the patch slot explicit.
