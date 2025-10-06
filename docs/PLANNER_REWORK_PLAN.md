# Event-Driven Planner Rework

The goal is to shift the Tick-Queued Utility AI (TQUAI) to an event-driven planner that reacts immediately to important stimuli while staying lightweight enough for 1,000+ entities.

## 1. Design & Scoping

**Tasks**
- Sketch the updated control flow: `trigger → ReplanQueue → planner drain → optional cancel → queue new action`.
- Identify every trigger source (stat thresholds, fear spikes, combat damage, manual overrides).
- Decide tick cadence (`every_n_ticks(2)` vs. `every tick`).

**Deliverable**

```
Stimulus/Event ─▶ enqueue(entity, priority)
                   │
                   ▼
            ReplanQueue { high[], normal[] }
                   │
        (tick, every_n_ticks(1))
                   │ drain order: high → normal (budget)
                   ▼
       cancel_if_active(entity) ─▶ plan_species_actions(entity)
                   │                        │
                   └──────► ActionQueue ◄───┘
```

**Trigger Sources**
- Hunger, thirst, energy crossing configured critical thresholds (normal priority).
- Fear spikes / predator proximity, combat damage, or explicit panic effects (high priority).
- Completion/failure of any `Action` (normal priority) to ensure the agent fills the gap immediately.
- Long-idle timer (normal priority) to recover entities that somehow stalled without an action.

**Cadence Decision**
- Planner drain runs every simulation tick (`every_n_ticks(1)`), matching the 10 Hz base rate. This keeps urgent requests responsive (worst-case 100 ms delay) while dramatically cutting work versus per-frame execution. A per-tick budget will prevent starvation under heavy load.

**Exit Criteria**
- Design notes (diagram, triggers, cadence) captured in this doc ✅
- Stakeholder sign-off on trigger list and cadence (pending review).

**Validation**
- Peer review the design notes before implementation starts.

## 2. Replan Queue Resource

**Tasks**
- Create a `ReplanQueue` resource with two priority lanes (high/normal) backed by a deduping set.
- Expose helper functions `push(entity, priority)` and `drain(max, priority)`.
- Ensure pushes ignore despawned entities gracefully.

**Exit Criteria**
- Unit tests covering dedupe, priority ordering, and draining behavior.

**Validation**
- `cargo test replan_queue::*` (or equivalent target).

## 3. Trigger Emitters

**Tasks**
- Update stat tick systems to enqueue when thresholds cross (critical hunger/thirst/energy).
- Hook fear/predator detection and combat damage to enqueue high-priority requests.
- Emit normal priority from long-idle/wander timers.
- Guard against spam: only enqueue on state transition.

**Exit Criteria**
- Tests (unit or integration) verifying each trigger enqueues exactly once per threshold crossing.

**Validation**
- Added tests pass and log output shows one enqueue per event under debug instrumentation.

## 4. Action Cancellation Support

**Tasks**
- Extend `Action` trait with `fn cancel(&mut self, world, entity)` (default no-op).
- Allow `ActionQueue` to cancel active/queued actions when a high-priority request demands replanning.
- Ensure cancellation resets `CurrentAction` and removes `Path`, `MoveOrder`, etc., if needed.

**Exit Criteria**
- Unit test where an action is canceled mid-execution and the entity becomes idle in the same tick.

**Validation**
- `cargo test action_cancellation` (new test module).

## 5. Tick-Scheduled Planner Drain

**Tasks**
- Move species planners to run with `.run_if(should_tick).run_if(every_n_ticks(N))`.
- Each tick, drain up to `BUDGET` entries from the high-priority lane, then the normal lane.
- For drained entities, attempt cancellation (if active) and run the planner once.
- Keep existing idle-entity planning as a fallback (still run each tick but cheap due to early skip).

**Exit Criteria**
- Systems compile; planner logs confirm tick cadence.
- Benchmark run shows ~6× reduction in planner invocations compared to frame-based behavior.

**Validation**
- `cargo check` succeeds.
- Capture a 60-second sim profile demonstrating tick-frequency planner logs.

## 6. End-to-End Testing

**Tasks**
- Scenario: predator enters rabbit area → fear trigger → planner cancels graze and queues escape action within one tick.
- Scenario: thirst hits critical → normal priority replan triggers drink action within two ticks.
- Scenario: background idle rabbits do not spam queue.

**Exit Criteria**
- Manual playtest or automated integration test confirming all scenarios.

**Validation**
- Record logs showing `ReplanQueue` events and confirm timing (Δtick ≤1 for combat/fear, ≤2 for stat thresholds).

## 7. Documentation & Cleanup

**Tasks**
- Update README / AI docs describing event-driven planning.
- Document new APIs (`ReplanQueue`, `Action::cancel`) and trigger responsibilities.
- Remove obsolete frame-time diagnostics and ensure logging respects cadence.

**Exit Criteria**
- Docs merged; code comments reflect new flow.

**Validation**
- Final `cargo fmt`, `cargo clippy`, and regression test run clean.
