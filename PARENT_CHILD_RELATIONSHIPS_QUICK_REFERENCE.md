# Parent-Child Relationships - Quick Reference

## Components

### ParentOf
Attached to parent entities. Tracks all offspring.

```rust
#[derive(Component)]
pub struct ParentOf {
    pub children: Vec<Entity>,
    pub first_birth_tick: u64,
}
```

**Methods**:
- `ParentOf::new(tick)` - Create with first birth tick
- `add_child(entity)` - Add offspring
- `remove_child(entity)` - Remove offspring
- `child_count()` -> usize
- `has_child(entity)` -> bool

### ChildOf
Attached to offspring entities. Tracks parent and birth time.

```rust
#[derive(Component)]
pub struct ChildOf {
    pub parent: Entity,
    pub born_tick: u64,
}
```

## API Functions

### Establish Relationships

**Commands variant** (use in systems):
```rust
establish_parent_child_relationship(parent, child, tick, &mut commands);
```

**Immediate variant** (use in tests/initialization):
```rust
establish_parent_child_immediate(parent, child, tick, &mut world);
```

### Remove Relationships

**Commands variant**:
```rust
remove_parent_child_relationship(parent, child, &mut commands);
```

**Immediate variant**:
```rust
remove_parent_child_immediate(parent, child, &mut world);
```

### Query Functions

```rust
get_parent(child, &world) -> Option<Entity>
get_children(parent, &world) -> Vec<Entity>
has_child(parent, child, &world) -> bool
has_parent(child, &world) -> bool
child_count(parent, &world) -> usize
child_birth_tick(child, &world) -> Option<u64>
```

## Systems

### Establish Birth Relationships
```rust
pub fn establish_birth_relationships(
    commands: Commands,
    tick: Res<SimulationTick>,
    newborns: Query<(Entity, &Mother), Added<Mother>>,
)
```
Creates ChildOf on newborns with Mother component.

### Establish ParentOf from Mother
```rust
pub fn establish_parent_of_from_mother(
    parents: Query<&mut ParentOf>,
    children: Query<(Entity, &Mother), Added<Mother>>,
    tick: Res<SimulationTick>,
)
```
Updates ParentOf component on parents to track offspring.

### Cleanup Orphaned Children
```rust
pub fn cleanup_orphaned_children_when_parent_dies(
    commands: Commands,
    children: Query<(Entity, &ChildOf)>,
    parents: Query<Entity, With<ParentOf>>,
)
```
Removes ChildOf from children when parents are removed.

## Usage Example

```rust
// At birth time
let parent = /*mother entity*/;
let child = /*newborn entity*/;
let current_tick = 100;

// Establish relationship (in system)
establish_parent_child_relationship(parent, child, current_tick, &mut commands);

// Query relationships
let children = get_children(parent, &world);
let parent_entity = get_parent(child, &world);

// Calculate age
let child_age = current_tick - child_birth_tick(child, &world).unwrap();

// Remove relationship (when child dies)
remove_parent_child_relationship(parent, child, &mut commands);
```

## Integration with Birth System

The relationship system integrates automatically:

1. Birth systems create babies with `Mother(parent_entity)` component
2. `establish_birth_relationships` system detects `Added<Mother>`
3. Creates `ChildOf` component on newborn
4. `establish_parent_of_from_mother` updates parent's `ParentOf`
5. On parent death, `cleanup_orphaned_children_when_parent_dies` removes relationships

## Test Coverage

- 11 component unit tests
- 11 system function tests
- 5 birth integration tests
- 8 comprehensive integration tests
- **Total: 35 tests, all passing**

## Files

| File | Purpose |
|------|---------|
| `src/entities/parent_child_relationships.rs` | Components + unit tests |
| `src/entities/birth_relationships.rs` | Birth system integration |
| `src/ai/parent_child_relationship_system.rs` | API functions + tests |
| `tests/parent_child_relationship_integration.rs` | Integration tests |

---

See `PARENT_CHILD_RELATIONSHIPS_DELIVERY.md` for complete documentation.
