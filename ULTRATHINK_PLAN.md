# UltraThink: Queue-Based AI Processing System
## Inspired by Dwarf Fortress LOD Architecture

## Date
2025-12-26

## Vision

**Core Concept**: In a simulator like real life, thoughts don't happen instantly. Decisions about where to wander, who to hunt, who to mate - these can wait in a queue and be processed when there's "mental capacity" available.

**Key Insight**: Only urgent things (hunger, thirst, fear) need immediate processing. Everything else can be queued and processed by priority over multiple ticks.

---

## Current Problem

**Status Quo** (failed DecisionTimer approach):
```
Every tick:
  For each of 500 entities:
    Check timer → Skip or Plan

Result:
  - Bursty workload (synchronized planning)
  - All-or-nothing (entity plans or doesn't)
  - No priority differentiation
  - TPS degraded despite faster ticks
```

**Why it failed**:
- Still processes entities synchronously every tick
- No smooth CPU budget
- No LOD (all entities treated equally)
- Doesn't address pathfinding bottleneck

---

## UltraThink Architecture

### 1. Think Queue System

**Concept**: Global queue of "thinking tasks" processed with fixed budget per tick

```rust
pub struct ThinkQueue {
    // Priority queue: urgent thoughts processed first
    high_priority: VecDeque<ThinkRequest>,   // Fear, hunger, thirst
    medium_priority: VecDeque<ThinkRequest>, // Reproduction, idle activities
    low_priority: VecDeque<ThinkRequest>,    // Wandering, exploration

    // Processing budget
    thinks_per_tick: usize,  // e.g., 50 thoughts per tick

    // Statistics
    queue_depth: (usize, usize, usize),
    thoughts_processed: u64,
}

pub struct ThinkRequest {
    entity: Entity,
    reason: ThinkReason,
    scheduled_tick: u64,  // When this was queued
    priority: ThinkPriority,
}

pub enum ThinkReason {
    // Urgent - process immediately
    FearTriggered,
    HungerCritical,
    ThirstCritical,
    Threatened,

    // Normal - can wait a few ticks
    HungerModerate,
    ThirstModerate,
    ActionCompleted,
    ActionFailed,
    ReproductionReady,

    // Low - can wait many ticks
    Idle,
    WanderTargetNeeded,
    ExplorationDesired,
    SocialInteraction,
}

pub enum ThinkPriority {
    Urgent,   // Process within 1-2 ticks
    Normal,   // Process within 5-10 ticks
    Low,      // Process within 20-50 ticks
}
```

### 2. Processing Flow

```rust
fn ultrathink_system(
    mut commands: Commands,
    mut think_queue: ResMut<ThinkQueue>,
    tick: Res<SimulationTick>,
    // ... other resources
) {
    let budget = think_queue.thinks_per_tick;
    let mut processed = 0;

    // Process up to budget, prioritizing urgent thoughts
    while processed < budget {
        // Try high priority first
        if let Some(request) = think_queue.high_priority.pop_front() {
            process_think_request(request, &mut commands);
            processed += 1;
            continue;
        }

        // Then medium priority
        if let Some(request) = think_queue.medium_priority.pop_front() {
            process_think_request(request, &mut commands);
            processed += 1;
            continue;
        }

        // Finally low priority
        if let Some(request) = think_queue.low_priority.pop_front() {
            process_think_request(request, &mut commands);
            processed += 1;
            continue;
        }

        // Queue empty - done for this tick
        break;
    }

    // Log queue depth for monitoring
    if tick.0 % 50 == 0 {
        info!(
            "🧠 ThinkQueue depth: {} urgent, {} normal, {} low | Processed {}/{}",
            think_queue.high_priority.len(),
            think_queue.medium_priority.len(),
            think_queue.low_priority.len(),
            processed,
            budget
        );
    }
}
```

### 3. Automatic Think Scheduling

**Triggers automatically queue think requests:**

```rust
// Fear trigger system
fn fear_trigger_system(
    mut think_queue: ResMut<ThinkQueue>,
    query: Query<(Entity, &FearState), Changed<FearState>>,
) {
    for (entity, fear) in query.iter() {
        if fear.is_fearful() {
            think_queue.schedule_urgent(entity, ThinkReason::FearTriggered);
        }
    }
}

// Hunger/thirst trigger system
fn needs_trigger_system(
    mut think_queue: ResMut<ThinkQueue>,
    query: Query<(Entity, &Hunger, &Thirst), Or<(Changed<Hunger>, Changed<Thirst>)>>,
) {
    for (entity, hunger, thirst) in query.iter() {
        if hunger.current < 20.0 || thirst.current < 20.0 {
            think_queue.schedule_urgent(entity, ThinkReason::HungerCritical);
        } else if hunger.current < 50.0 || thirst.current < 50.0 {
            think_queue.schedule_normal(entity, ThinkReason::HungerModerate);
        }
    }
}

// Action completion trigger
fn action_completion_trigger(
    mut think_queue: ResMut<ThinkQueue>,
    action_queue: Res<ActionQueue>,
) {
    for (entity, _tick) in action_queue.recently_completed() {
        think_queue.schedule_normal(entity, ThinkReason::ActionCompleted);
    }
}

// Idle entities self-queue for low-priority thinking
fn idle_scheduler_system(
    mut think_queue: ResMut<ThinkQueue>,
    query: Query<Entity, (With<Creature>, Without<CurrentAction>)>,
    tick: Res<SimulationTick>,
) {
    // Only schedule idle thoughts every 20 ticks
    if tick.0 % 20 != 0 {
        return;
    }

    for entity in query.iter() {
        if !think_queue.contains(entity) {
            think_queue.schedule_low(entity, ThinkReason::Idle);
        }
    }
}
```

### 4. Level of Detail (LOD) System

**Entities farther from "attention focus" get lower priority:**

```rust
pub struct ThinkLOD {
    // Distance from player/camera (if applicable)
    // Or could be "importance score" based on:
    // - Proximity to other entities
    // - Recent interesting events
    // - Species rarity
    distance_from_focus: f32,
}

impl ThinkLOD {
    pub fn calculate_priority(&self, base_reason: ThinkReason) -> ThinkPriority {
        match base_reason {
            // Always urgent regardless of distance
            ThinkReason::FearTriggered |
            ThinkReason::HungerCritical |
            ThinkReason::ThirstCritical => ThinkPriority::Urgent,

            // Downgrade based on distance
            ThinkReason::HungerModerate |
            ThinkReason::ActionCompleted => {
                if self.distance_from_focus < 100.0 {
                    ThinkPriority::Normal
                } else {
                    ThinkPriority::Low
                }
            }

            // Already low priority
            _ => ThinkPriority::Low,
        }
    }
}
```

### 5. Dynamic Budget Scaling

**Adjust thinks_per_tick based on performance:**

```rust
pub struct AdaptiveThinkBudget {
    base_budget: usize,
    current_budget: usize,
    target_tick_time_ms: f32,

    // Performance tracking
    recent_tick_times: VecDeque<f32>,
    adjustment_interval: u64,
}

impl AdaptiveThinkBudget {
    pub fn adjust(&mut self, actual_tick_time_ms: f32, tick: u64) {
        self.recent_tick_times.push_back(actual_tick_time_ms);
        if self.recent_tick_times.len() > 10 {
            self.recent_tick_times.pop_front();
        }

        // Adjust every 50 ticks
        if tick % self.adjustment_interval != 0 {
            return;
        }

        let avg_tick_time: f32 = self.recent_tick_times.iter().sum::<f32>()
            / self.recent_tick_times.len() as f32;

        if avg_tick_time < self.target_tick_time_ms * 0.8 {
            // We have headroom - increase budget
            self.current_budget = (self.current_budget * 110 / 100).min(200);
            info!("🧠 Increasing think budget to {}", self.current_budget);
        } else if avg_tick_time > self.target_tick_time_ms * 1.2 {
            // Over budget - decrease
            self.current_budget = (self.current_budget * 90 / 100).max(10);
            warn!("⚠️ Decreasing think budget to {}", self.current_budget);
        }
    }
}
```

---

## Implementation Phases

### Phase 1: Core Queue Infrastructure (2-3 hours)

**Files to create:**
- `src/ai/ultrathink/mod.rs` - Module root
- `src/ai/ultrathink/queue.rs` - ThinkQueue implementation
- `src/ai/ultrathink/request.rs` - ThinkRequest, ThinkReason, ThinkPriority

**Tasks:**
1. Create ThinkQueue resource with priority queues
2. Implement schedule_urgent/normal/low methods
3. Create ultrathink_system that processes N requests per tick
4. Add basic logging/metrics

**Expected outcome**: Queue infrastructure working, manual scheduling only

### Phase 2: Automatic Scheduling (2-3 hours)

**Files to modify:**
- `src/ai/triggers/` - Add think queue scheduling to existing triggers
- `src/ai/queue.rs` - Schedule thinks on action completion/failure

**Tasks:**
1. Integrate with fear trigger system
2. Integrate with hunger/thirst needs monitoring
3. Add action completion/failure scheduling
4. Create idle entity scheduler

**Expected outcome**: Entities automatically queue thinking based on events

### Phase 3: LOD System (1-2 hours)

**Files to create:**
- `src/ai/ultrathink/lod.rs` - LOD calculation and priority adjustment

**Tasks:**
1. Create ThinkLOD component
2. Implement distance-based priority downgrade
3. Add importance scoring (optional)

**Expected outcome**: Distant/unimportant entities get lower priority

### Phase 4: Adaptive Budget (1-2 hours)

**Files to create:**
- `src/ai/ultrathink/adaptive.rs` - Dynamic budget adjustment

**Tasks:**
1. Create AdaptiveThinkBudget resource
2. Implement tick time tracking
3. Add automatic budget adjustment
4. Tune thresholds for stability

**Expected outcome**: System self-regulates to maintain target TPS

### Phase 5: Remove Old Timer System (1 hour)

**Files to modify:**
- Rollback all DecisionTimer changes from Phase 5
- Remove timer checks from planners
- Clean up spawn functions

**Expected outcome**: Clean codebase using only UltraThink

---

## Expected Performance

### Conservative Estimates

**Assumptions:**
- 500 entities
- 50 thinks per tick (10% of entities think per tick)
- Average think time: 0.88ms (from profiling)

**Calculation:**
```
Current: 500 entities × 0.88ms = 440ms per tick
UltraThink: 50 entities × 0.88ms = 44ms per tick

Speedup: 440ms / 44ms = 10x improvement
Expected TPS: 1 / (44ms + 60ms other) ≈ 9.6 TPS ✅
```

### Optimistic Estimates (with LOD)

**With LOD adjustments:**
- 60% entities at low priority (process every 20-50 ticks)
- 30% entities at normal priority (process every 5-10 ticks)
- 10% entities at urgent priority (process within 1-2 ticks)

**Effective think budget needed:**
```
Urgent: 50 entities / 1.5 avg ticks = 33 per tick
Normal: 150 entities / 7.5 avg ticks = 20 per tick
Low: 300 entities / 35 avg ticks = 8.5 per tick
Total: 61.5 thinks per tick (similar to base estimate)
```

**With adaptive budget:**
- Start at 50 thinks per tick
- Adjust to 60-80 based on actual tick time
- Maintain 10 TPS target automatically

---

## Advantages Over DecisionTimer

| Aspect | DecisionTimer | UltraThink |
|--------|---------------|------------|
| **CPU Load** | Bursty (all rabbits on tick 10, 20, 30...) | Smooth (always process N entities) |
| **Priority** | No differentiation | Urgent/Normal/Low queues |
| **Scalability** | O(entities) every N ticks | O(budget) every tick |
| **LOD** | Not supported | Natural priority downgrade |
| **Adaptability** | Fixed intervals | Dynamic budget adjustment |
| **Queue Visibility** | No queue | Full queue metrics |
| **Pathfinding** | Not addressed | Can be queued/prioritized too |

---

## Pathfinding Integration (Bonus)

**UltraThink can also manage pathfinding:**

```rust
pub enum ThinkReason {
    // ... existing reasons ...

    // Pathfinding requests
    PathfindingNeeded {
        from: IVec2,
        to: IVec2,
        urgency: PathUrgency
    },
}

pub enum PathUrgency {
    Immediate,  // Fleeing from predator
    Normal,     // Moving to food/water
    Lazy,       // Wandering
}
```

**Benefits:**
- Expensive pathfinding spread across ticks
- Urgent paths (fleeing) processed first
- Lazy paths (wandering) can wait
- Prevents pathfinding spikes

---

## Metrics & Monitoring

**Dashboard metrics:**
```
🧠 UltraThink Status - Tick 150
├── Queue Depth: 45 urgent | 120 normal | 230 low
├── Processing: 50/50 budget used (100%)
├── Avg Wait Time: 1.2 ticks urgent | 8.5 normal | 35 low
├── Throughput: 50 thinks/tick × 10 TPS = 500 thinks/sec
└── Backlog: None (queue stable)
```

**Health indicators:**
- ✅ Queue depth stable or decreasing
- ✅ Wait times within targets
- ✅ Budget utilization 80-100%
- ⚠️ Queue depth growing → increase budget
- ❌ Urgent queue backlog → critical issue

---

## Rollout Strategy

### Step 1: Test Harness (Day 1)
- Implement core queue without integration
- Test with manual scheduling
- Verify priority ordering works
- Measure overhead of queue itself

### Step 2: Parallel Mode (Day 2)
- Run UltraThink alongside existing planners
- Both systems plan independently
- Compare decisions for correctness
- Measure performance delta

### Step 3: Cutover (Day 3)
- Disable old planner systems
- Enable UltraThink fully
- Monitor queue health
- Tune budget for 10 TPS

### Step 4: LOD & Polish (Day 4)
- Add LOD system
- Enable adaptive budget
- Stress test with 1000 entities
- Document final performance

---

## Success Criteria

### Must Have
- ✅ Achieve 10 TPS sustained with 500 entities
- ✅ Urgent thoughts processed within 1-2 ticks
- ✅ Queue remains stable (not growing)
- ✅ All entities eventually get thinking time
- ✅ No behavioral regressions

### Nice to Have
- 🎯 15+ TPS with 500 entities
- 🎯 10 TPS with 1000 entities
- 🎯 Adaptive budget working smoothly
- 🎯 LOD visibly reducing far-entity processing
- 🎯 Pathfinding integrated into queue

---

## Risk Mitigation

**Risk 1: Queue backlog**
- Mitigation: Adaptive budget increases when queue grows
- Fallback: Manual budget tuning

**Risk 2: Urgent thoughts starved**
- Mitigation: Strict priority - urgent always processed first
- Fallback: Separate urgent budget allocation

**Risk 3: Behavioral changes**
- Mitigation: Parallel mode testing validates correctness
- Fallback: Tune wait time targets to match current behavior

**Risk 4: Implementation complexity**
- Mitigation: Phased rollout with testing at each phase
- Fallback: Keep old planner code until fully validated

---

## Next Steps

1. **Review this plan** - Get user approval on architecture
2. **Create ultrathink module** - Scaffold files and basic structure
3. **Implement Phase 1** - Core queue infrastructure
4. **Test Phase 1** - Verify queue works with manual scheduling
5. **Iterate through phases** - Each phase adds capability

**Estimated Total Effort**: 8-12 hours for full implementation
**Expected Performance Gain**: 10x speedup → 10 TPS target achieved ✅

---

**This is the proper solution** - queue-based, priority-driven, LOD-capable, and adaptive. Exactly what you originally envisioned! 🎯
