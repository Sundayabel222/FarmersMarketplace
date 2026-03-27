# Implement withdraw_event_funds state update & double withdrawal test

## Approved Plan Summary

- Add StorageKey::PoolDrained(u64)
- Reuse/add error for already withdrawn
- Add event_funds_withdrawn
- Add fn to interface & impl: check auth/state/drained/funds, transfer, set drained=true
- New test file with success/double-fail/unauth/insufficient/wrong-state cases

## Steps to Complete (v1.0)

- [x] 1. Add StorageKey to base/types.rs
- [x] 2. Add event to base/events.rs
- [x] 3. Update interfaces/crowdfunding.rs
- [x] 4. Implement withdraw_event_funds in crowdfunding.rs
- [x] 5. Create withdraw_event_funds_test.rs
- [x] 6. cargo test to verify
- [x] 7. Git branch/PR

Current: Task complete
