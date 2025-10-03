/// Species Systems Registry - Centralized system information
///
/// This module provides a centralized registry for species-specific systems
/// without attempting dynamic system registration (which is complex in Bevy).
/// Instead, it provides information about which systems each species has,
/// allowing for more readable and maintainable plugin code.

use bevy::prelude::*;

// ============================================================================
// SPECIES SYSTEMS DESCRIPTOR
// ============================================================================

/// Descriptor containing information about systems for a specific species
#[derive(Debug, Clone)]
pub struct SpeciesSystemsDescriptor {
    /// Species identifier
    pub species: &'static str,

    /// Whether this species has a mate matching system
    pub has_mate_matching: bool,

    /// Whether this species has a birth system
    pub has_birth_system: bool,

    /// Whether this species has an AI planner system
    pub has_planner_system: bool,
}

impl SpeciesSystemsDescriptor {
    /// Create a new descriptor for a species
    pub const fn new(species: &'static str) -> Self {
        Self {
            species,
            has_mate_matching: false,
            has_birth_system: false,
            has_planner_system: false,
        }
    }

    /// Add mate matching system
    pub const fn with_mate_matching(mut self) -> Self {
        self.has_mate_matching = true;
        self
    }

    /// Add birth system
    pub const fn with_birth_system(mut self) -> Self {
        self.has_birth_system = true;
        self
    }

    /// Add planner system
    pub const fn with_planner_system(mut self) -> Self {
        self.has_planner_system = true;
        self
    }
}

// ============================================================================
// SPECIES SYSTEMS REGISTRY
// ============================================================================

/// Global registry containing all species systems information
pub static SPECIES_SYSTEMS_REGISTRY: SpeciesSystemsRegistry = SpeciesSystemsRegistry::new();

/// Registry that holds all species system descriptors
pub struct SpeciesSystemsRegistry {
    descriptors: &'static [SpeciesSystemsDescriptor],
}

impl SpeciesSystemsRegistry {
    /// Create the species systems registry with all registered species
    pub const fn new() -> Self {
        const DESCRIPTORS: &[SpeciesSystemsDescriptor] = &[
            // Rabbit systems
            SpeciesSystemsDescriptor::new("Rabbit")
                .with_mate_matching()
                .with_birth_system()
                .with_planner_system(),

            // Deer systems
            SpeciesSystemsDescriptor::new("Deer")
                .with_mate_matching()
                .with_birth_system()
                .with_planner_system(),

            // Raccoon systems
            SpeciesSystemsDescriptor::new("Raccoon")
                .with_mate_matching()
                .with_birth_system()
                .with_planner_system(),
        ];

        Self {
            descriptors: DESCRIPTORS,
        }
    }

    /// Get all species system descriptors
    pub fn get_descriptors(&self) -> &[SpeciesSystemsDescriptor] {
        self.descriptors
    }

    /// Get species that have mate matching systems
    pub fn get_species_with_mate_matching(&self) -> Vec<&'static str> {
        self.descriptors
            .iter()
            .filter(|d| d.has_mate_matching)
            .map(|d| d.species)
            .collect()
    }

    /// Get species that have birth systems
    pub fn get_species_with_birth_systems(&self) -> Vec<&'static str> {
        self.descriptors
            .iter()
            .filter(|d| d.has_birth_system)
            .map(|d| d.species)
            .collect()
    }

    /// Get species that have planner systems
    pub fn get_species_with_planner_systems(&self) -> Vec<&'static str> {
        self.descriptors
            .iter()
            .filter(|d| d.has_planner_system)
            .map(|d| d.species)
            .collect()
    }

    /// Find descriptor by species name
    pub fn find_by_species(&self, species: &str) -> Option<&SpeciesSystemsDescriptor> {
        self.descriptors.iter().find(|d| d.species == species)
    }

    /// Check if a species has a specific system type
    pub fn species_has_mate_matching(&self, species: &str) -> bool {
        self.find_by_species(species)
            .map(|d| d.has_mate_matching)
            .unwrap_or(false)
    }

    pub fn species_has_birth_system(&self, species: &str) -> bool {
        self.find_by_species(species)
            .map(|d| d.has_birth_system)
            .unwrap_or(false)
    }

    pub fn species_has_planner_system(&self, species: &str) -> bool {
        self.find_by_species(species)
            .map(|d| d.has_planner_system)
            .unwrap_or(false)
    }
}

// ============================================================================
// PLUGIN REGISTRATION HELPERS
// ============================================================================

/// Example of how to use the registry in plugin code
///
/// Instead of hard-coding species names, you can query the registry:
///
/// ```rust
/// impl Plugin for MyPlugin {
///     fn build(&self, app: &mut App) {
///         // Get all species that have mate matching systems
///         let species_with_mating = SPECIES_SYSTEMS_REGISTRY.get_species_with_mate_matching();
///         info!("Species with mate matching: {:?}", species_with_mating);
///
///         // Check if a specific species has a system
///         if SPECIES_SYSTEMS_REGISTRY.species_has_birth_system("Rabbit") {
///             app.add_systems(Update, rabbit_birth_system.run_if(should_run_tick_systems));
///         }
///     }
/// }
/// ```
///
/// This provides the benefits of centralized configuration while maintaining
/// type safety and avoiding complex dynamic system registration.

// ============================================================================
// LEGACY COMPATIBILITY FUNCTIONS
// ============================================================================

/// Get species names that have mate matching (for backward compatibility)
pub fn get_mate_matching_system_names() -> Vec<&'static str> {
    SPECIES_SYSTEMS_REGISTRY.get_species_with_mate_matching()
}

/// Get species names that have birth systems (for backward compatibility)
pub fn get_birth_system_names() -> Vec<&'static str> {
    SPECIES_SYSTEMS_REGISTRY.get_species_with_birth_systems()
}

/// Get species names that have planner systems (for backward compatibility)
pub fn get_planner_system_names() -> Vec<&'static str> {
    SPECIES_SYSTEMS_REGISTRY.get_species_with_planner_systems()
}