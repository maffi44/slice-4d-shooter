# Context for keep the captured flag attached when its carrier changes ID

[Current task](flag-carrier-id-changes.md)

## 2026-11-20 code review on `4007599`

`FlagStatus::Captured(id)` drives the direct `GiveMeTargetPosition` sent each tick. The flag's common-message handler handles transform updates and disconnects but falls through on `IWasChangedMyId`. When the carrier changes ID, the request can be dropped or delivered to a different actor that acquired the old ID. The same stored ID is compared with the sender of `PlayerDied`. This was traced in code during review.

`MoveWBonusSpot` already rewrites a stored ID when the notification's `from` matches. On collision, the displaced actor's notification is queued first. The actor-container tests verify changes in the table and emitted notifications, without exercising the flag. See [flag](../../../src/actor/flag.rs), [bonus spot](../../../src/actor/move_w_bonus.rs) and [message ordering](../message-ordering/).
