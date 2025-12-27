/// Think Request Types for UltraThink Queue System
use bevy::prelude::*;
use std::fmt;

/// Priority level for think requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkPriority {
    /// Process within 1-2 ticks - critical survival needs
    Urgent,
    /// Process within 5-10 ticks - normal needs and activities
    Normal,
    /// Process within 20-50 ticks - idle activities
    Low,
}

/// Reason why an entity needs to think
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl ThinkReason {
    /// Get the default priority for this reason
    pub fn default_priority(&self) -> ThinkPriority {
        match self {
            ThinkReason::FearTriggered
            | ThinkReason::HungerCritical
            | ThinkReason::ThirstCritical
            | ThinkReason::Threatened => ThinkPriority::Urgent,

            ThinkReason::HungerModerate
            | ThinkReason::ThirstModerate
            | ThinkReason::ActionCompleted
            | ThinkReason::ActionFailed
            | ThinkReason::ReproductionReady => ThinkPriority::Normal,

            ThinkReason::Idle
            | ThinkReason::WanderTargetNeeded
            | ThinkReason::ExplorationDesired
            | ThinkReason::SocialInteraction => ThinkPriority::Low,
        }
    }
}

impl fmt::Display for ThinkReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThinkReason::FearTriggered => write!(f, "FearTriggered"),
            ThinkReason::HungerCritical => write!(f, "HungerCritical"),
            ThinkReason::ThirstCritical => write!(f, "ThirstCritical"),
            ThinkReason::Threatened => write!(f, "Threatened"),
            ThinkReason::HungerModerate => write!(f, "HungerModerate"),
            ThinkReason::ThirstModerate => write!(f, "ThirstModerate"),
            ThinkReason::ActionCompleted => write!(f, "ActionCompleted"),
            ThinkReason::ActionFailed => write!(f, "ActionFailed"),
            ThinkReason::ReproductionReady => write!(f, "ReproductionReady"),
            ThinkReason::Idle => write!(f, "Idle"),
            ThinkReason::WanderTargetNeeded => write!(f, "WanderTargetNeeded"),
            ThinkReason::ExplorationDesired => write!(f, "ExplorationDesired"),
            ThinkReason::SocialInteraction => write!(f, "SocialInteraction"),
        }
    }
}

/// A request for an entity to think
#[derive(Debug, Clone, Copy)]
pub struct ThinkRequest {
    pub entity: Entity,
    pub reason: ThinkReason,
    pub scheduled_tick: u64,
    pub priority: ThinkPriority,
}

impl ThinkRequest {
    /// Create a new think request
    pub fn new(entity: Entity, reason: ThinkReason, scheduled_tick: u64) -> Self {
        let priority = reason.default_priority();
        Self {
            entity,
            reason,
            scheduled_tick,
            priority,
        }
    }

    /// Create a new think request with explicit priority (for LOD system later)
    pub fn new_with_priority(
        entity: Entity,
        reason: ThinkReason,
        scheduled_tick: u64,
        priority: ThinkPriority,
    ) -> Self {
        Self {
            entity,
            reason,
            scheduled_tick,
            priority,
        }
    }
}

impl fmt::Display for ThinkRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThinkRequest(entity={:?}, reason={}, priority={:?}, tick={})",
            self.entity, self.reason, self.priority, self.scheduled_tick
        )
    }
}
