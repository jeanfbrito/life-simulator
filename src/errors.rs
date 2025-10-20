//! Common error types for the Life Simulator
//!
//! This module provides standardized error handling across all systems
//! to replace unwrap() calls and improve stability.

use bevy::prelude::*;
use std::fmt;

/// Core error type for the Life Simulator
#[derive(Debug, Clone)]
pub enum LifeSimulatorError {
    /// Resource grid related errors
    ResourceGrid(String),
    
    /// Pathfinding related errors
    Pathfinding(String),
    
    /// Entity system errors
    Entity(String),
    
    /// World loading and serialization errors
    WorldLoading(String),
    
    /// AI system errors
    AI(String),
    
    /// Vegetation system errors
    Vegetation(String),
    
    /// Terrain generation errors
    Terrain(String),
    
    /// Network/API errors
    Network(String),
    
    /// Configuration errors
    Config(String),
    
    /// Generic error with custom message
    Generic(String),
}

impl fmt::Display for LifeSimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifeSimulatorError::ResourceGrid(msg) => write!(f, "Resource Grid Error: {}", msg),
            LifeSimulatorError::Pathfinding(msg) => write!(f, "Pathfinding Error: {}", msg),
            LifeSimulatorError::Entity(msg) => write!(f, "Entity Error: {}", msg),
            LifeSimulatorError::WorldLoading(msg) => write!(f, "World Loading Error: {}", msg),
            LifeSimulatorError::AI(msg) => write!(f, "AI Error: {}", msg),
            LifeSimulatorError::Vegetation(msg) => write!(f, "Vegetation Error: {}", msg),
            LifeSimulatorError::Terrain(msg) => write!(f, "Terrain Error: {}", msg),
            LifeSimulatorError::Network(msg) => write!(f, "Network Error: {}", msg),
            LifeSimulatorError::Config(msg) => write!(f, "Configuration Error: {}", msg),
            LifeSimulatorError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for LifeSimulatorError {}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, LifeSimulatorError>;

/// Error conversion helpers
impl LifeSimulatorError {
    /// Create a resource grid error
    pub fn resource_grid<S: Into<String>>(msg: S) -> Self {
        Self::ResourceGrid(msg.into())
    }
    
    /// Create a pathfinding error
    pub fn pathfinding<S: Into<String>>(msg: S) -> Self {
        Self::Pathfinding(msg.into())
    }
    
    /// Create an entity error
    pub fn entity<S: Into<String>>(msg: S) -> Self {
        Self::Entity(msg.into())
    }
    
    /// Create a world loading error
    pub fn world_loading<S: Into<String>>(msg: S) -> Self {
        Self::WorldLoading(msg.into())
    }
    
    /// Create an AI error
    pub fn ai<S: Into<String>>(msg: S) -> Self {
        Self::AI(msg.into())
    }
    
    /// Create a vegetation error
    pub fn vegetation<S: Into<String>>(msg: S) -> Self {
        Self::Vegetation(msg.into())
    }
    
    /// Create a terrain error
    pub fn terrain<S: Into<String>>(msg: S) -> Self {
        Self::Terrain(msg.into())
    }
    
    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }
    
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
}

/// Trait for converting common error types to LifeSimulatorError
pub trait IntoLifeSimulatorError<T> {
    fn into_life_simulator_error(self) -> Result<T>;
}

/// Implement conversions for common error types
impl<T> IntoLifeSimulatorError<T> for std::sync::PoisonError<std::sync::MutexGuard<'_, T>> {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::generic(format!("Mutex poison error: {}", self)))
    }
}

impl<T> IntoLifeSimulatorError<T> for std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>> {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::generic(format!("RwLock read poison error: {}", self)))
    }
}

impl<T> IntoLifeSimulatorError<T> for std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>> {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::generic(format!("RwLock write poison error: {}", self)))
    }
}

impl<T> IntoLifeSimulatorError<T> for std::io::Error {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::network(format!("IO error: {}", self)))
    }
}

impl<T> IntoLifeSimulatorError<T> for serde_json::Error {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::world_loading(format!("JSON serialization error: {}", self)))
    }
}

impl<T> IntoLifeSimulatorError<T> for ron::Error {
    fn into_life_simulator_error(self) -> Result<T> {
        Err(LifeSimulatorError::world_loading(format!("RON serialization error: {}", self)))
    }
}

/// Helper macros for common error patterns
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error);
        }
    };
}

#[macro_export]
macro_rules! ok_or_error {
    ($option:expr, $error:expr) => {
        $option.ok_or_else(|| $error)
    };
}

#[macro_export]
macro_rules! error_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| LifeSimulatorError::generic(format!("{}: {}", $context, e)))
    };
}
