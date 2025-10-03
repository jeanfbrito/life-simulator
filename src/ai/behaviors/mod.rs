pub mod drinking;
pub mod eating;
pub mod follow;
/// Behaviors Module - Reusable AI behaviors for different entity types
///
/// This module contains behavior evaluation functions that can be mixed and matched
/// for different entity types. Each behavior is self-contained and can be used by
/// multiple species.
///
/// ## Available Behaviors:
///
/// ### Grazing (herbivores)
/// - Rabbits, Deer, Sheep, Horses
/// - Seeks grass tiles for foraging
/// - Low priority idle behavior
///
/// ### Drinking (all animals)
/// - All entities with thirst
/// - Seeks water sources
/// - Priority scales with thirst level
///
/// ### Eating (herbivores)
/// - Rabbits, Deer, Sheep, Horses
/// - Consumes grass to reduce hunger
/// - Priority scales with hunger level
///
/// ### Resting (all animals)
/// - All entities with energy
/// - Rests in place to regenerate energy
/// - Priority scales with tiredness
///
/// ## Future Behaviors:
/// - Hunting (carnivores) - Wolves, Bears seeking prey
/// - Fleeing (prey) - Rabbits fleeing from predators  
/// - Socializing (pack animals) - Wolves, Deer grouping behavior
/// - Breeding (all) - Reproduction when healthy
/// - Hoarding (some) - Squirrels collecting food
pub mod grazing;
pub mod resting;

pub use drinking::evaluate_drinking_behavior;
pub use eating::evaluate_eating_behavior;
pub use follow::evaluate_follow_behavior;
pub use grazing::evaluate_grazing_behavior;
pub use resting::evaluate_resting_behavior;
