#![allow(dead_code)]
/// Consideration system for Utility AI
///
/// Considerations evaluate context to produce utility scores (0.0 to 1.0).
/// Multiple considerations combine to determine action utility.
use bevy::prelude::*;
use crate::types::newtypes::Utility;

/// Response curve types for considerations
/// Transforms input value (0-1) to output score (0-1)
#[derive(Debug, Clone, Copy)]
pub enum ResponseCurve {
    /// Linear: output = input
    Linear,
    /// Quadratic: output = input^2 (slow start, fast finish)
    Quadratic,
    /// InverseQuadratic: output = 1 - (1-input)^2 (fast start, slow finish)
    InverseQuadratic,
    /// Exponential: output = input^exponent
    Exponential(f32),
    /// Boolean: 0 if input < threshold, 1 if >=
    Boolean(f32),
}

impl ResponseCurve {
    /// Apply the curve to a normalized input value [0, 1]
    pub fn evaluate(&self, input: f32) -> f32 {
        let clamped = input.clamp(0.0, 1.0);

        match self {
            ResponseCurve::Linear => clamped,
            ResponseCurve::Quadratic => clamped * clamped,
            ResponseCurve::InverseQuadratic => 1.0 - (1.0 - clamped).powi(2),
            ResponseCurve::Exponential(exp) => clamped.powf(*exp),
            ResponseCurve::Boolean(threshold) => {
                if clamped >= *threshold {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    /// Apply the curve and return a typed Utility value
    pub fn evaluate_utility(&self, input: f32) -> Utility {
        Utility::new(self.evaluate(input))
    }
}

/// A single consideration that contributes to action utility
pub trait Consideration: Send + Sync {
    /// Evaluate this consideration and return a score [0, 1]
    fn evaluate(&self, world: &World, entity: Entity) -> f32;

    /// Get the response curve for this consideration
    fn curve(&self) -> ResponseCurve {
        ResponseCurve::Linear
    }

    /// Get the final score (evaluation + curve)
    fn score(&self, world: &World, entity: Entity) -> f32 {
        let raw_value = self.evaluate(world, entity);
        self.curve().evaluate(raw_value)
    }

    /// Get consideration name for debugging
    fn name(&self) -> &'static str;
}

/// Set of considerations that combine to produce final utility
pub struct ConsiderationSet {
    considerations: Vec<Box<dyn Consideration>>,
    /// How to combine consideration scores
    combination_method: CombinationMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum CombinationMethod {
    /// Multiply all scores together (geometric mean effect)
    Multiply,
    /// Average all scores
    Average,
    /// Take minimum score (weakest link)
    Min,
    /// Take maximum score (strongest link)
    Max,
}

impl ConsiderationSet {
    pub fn new(method: CombinationMethod) -> Self {
        Self {
            considerations: Vec::new(),
            combination_method: method,
        }
    }

    pub fn add<C: Consideration + 'static>(&mut self, consideration: C) {
        self.considerations.push(Box::new(consideration));
    }

    /// Evaluate all considerations and combine into final utility (raw f32)
    pub fn evaluate(&self, world: &World, entity: Entity) -> f32 {
        if self.considerations.is_empty() {
            return 0.0;
        }

        let scores: Vec<f32> = self
            .considerations
            .iter()
            .map(|c| c.score(world, entity))
            .collect();

        match self.combination_method {
            CombinationMethod::Multiply => scores.iter().product(),
            CombinationMethod::Average => scores.iter().sum::<f32>() / scores.len() as f32,
            CombinationMethod::Min => scores.iter().cloned().fold(f32::INFINITY, f32::min),
            CombinationMethod::Max => scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
        }
    }

    /// Evaluate all considerations and combine into final Utility (typed)
    pub fn evaluate_utility(&self, world: &World, entity: Entity) -> Utility {
        Utility::new(self.evaluate(world, entity))
    }
}

// =============================================================================
// COMMON CONSIDERATIONS
// =============================================================================

/// Consideration: How thirsty is the entity?
/// Returns normalized thirst value [0, 1]
pub struct ThirstConsideration {
    /// Response curve (default: quadratic for urgency at high thirst)
    curve: ResponseCurve,
}

impl ThirstConsideration {
    pub fn new() -> Self {
        Self {
            curve: ResponseCurve::Quadratic, // Urgency increases rapidly
        }
    }

    pub fn with_curve(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ThirstConsideration {
    fn evaluate(&self, world: &World, entity: Entity) -> f32 {
        use crate::entities::stats::Thirst;

        if let Some(thirst) = world.get::<Thirst>(entity) {
            // Return normalized thirst (0-1, where 1 = very thirsty)
            thirst.0.normalized()
        } else {
            0.0
        }
    }

    fn curve(&self) -> ResponseCurve {
        self.curve
    }

    fn name(&self) -> &'static str {
        "Thirst"
    }
}

/// Consideration: Distance to target location
/// Returns inverse of distance (closer = higher score)
pub struct DistanceConsideration {
    target: IVec2,
    max_distance: f32,
    curve: ResponseCurve,
}

impl DistanceConsideration {
    pub fn new(target: IVec2, max_distance: f32) -> Self {
        Self {
            target,
            max_distance,
            curve: ResponseCurve::InverseQuadratic, // Prefer closer targets
        }
    }
}

impl Consideration for DistanceConsideration {
    fn evaluate(&self, world: &World, entity: Entity) -> f32 {
        use crate::entities::TilePosition;

        if let Some(pos) = world.get::<TilePosition>(entity) {
            let distance = pos.tile.as_vec2().distance(self.target.as_vec2());

            // Normalize: 0 distance = 1.0, max_distance = 0.0
            let normalized = 1.0 - (distance / self.max_distance).min(1.0);
            normalized.max(0.0)
        } else {
            0.0
        }
    }

    fn curve(&self) -> ResponseCurve {
        self.curve
    }

    fn name(&self) -> &'static str {
        "Distance"
    }
}
