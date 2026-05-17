use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy::scene::SceneRoot;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::character::world::CharacterWorldData;
use game_shared::models::http::account::AccountResource;
use game_shared::models::player::{
    PartyCompanion, PartySpawnedCharacter, Player, PlayerMovementStats,
};
use game_shared::models::world::TestWorldFloor;
use std::f32::consts::PI;

const PLAYER_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 0.9, 0.0);
const LAYER_WORLD: u32 = 1 << 0;
const LAYER_PLAYER: u32 = 1 << 1;
const LAYER_COMPANION: u32 = 1 << 2;
const COMPANION_SPAWN_MIN_DISTANCE: f32 = 2.0;
const COMPANION_SPAWN_MAX_DISTANCE: f32 = 10.0;
const COMPANION_SPAWN_HEIGHT: f32 = 0.9;
const COMPANION_SOLID_CLEARANCE: f32 = 1.2;
const COMPANION_PARTY_CLEARANCE: f32 = 1.8;
const COMPANION_SPAWN_ATTEMPTS: usize = 64;

/// Places the active player and remaining party members into the world.
///
/// This function resolves the active team from [`AccountResource`], reads the
/// active character ID from `current_use`, loads character world metadata from
/// `assets/entities/characters/{id}.json`, and spawns a player entity with the
/// corresponding GLB scene from `assets/dev/models/{model_name}`.
///
/// # Data Flow
/// 1. Read the account response and resolve the active team (`team.active == true`).
/// 2. Read active character ID from `current_use`.
/// 3. Resolve the active character data from [`CharacterGroup`].
/// 4. Load [`CharacterWorldData`] by character ID.
/// 5. Spawn the controlled `Player` entity at the player spawn point.
/// 6. Spawn non-controlled party companions around the player within a 10 meter radius.
///
/// # Parameters
/// - `commands`: Used to despawn existing player entities and spawn the new one.
/// - `account_resource`: Contains account response data including teams.
/// - `character_group`: Contains the generated character collection from account data.
/// - `asset_server`: Loads the GLB scene referenced by `model_name`.
/// - `existing_party_entities`: Query for already spawned player and companion entities.
///
/// # Behavior Notes
/// - Existing player/companion entities are despawned before spawning the current party.
/// - Companion spawn points are validated against static colliders and spacing constraints.
/// - If required data is missing (no account, no active team, missing character, missing
///   world definition file), an error is logged and no spawn occurs.
pub fn place_to_world(
    mut commands: Commands,
    account_resource: Res<AccountResource>,
    character_group: Res<CharacterGroup>,
    asset_server: Res<AssetServer>,
    existing_party_entities: Query<Entity, Or<(With<Player>, With<PartyCompanion>)>>,
    static_collider_query: Query<
        (Entity, &Transform, &Collider, Option<&RigidBody>, Has<TestWorldFloor>),
    >,
) {
    for entity in &existing_party_entities {
        commands.entity(entity).despawn_children().despawn();
    }

    let Some(account_response) = account_resource.0.as_ref() else {
        error!("Account Resource is None!");
        return;
    };

    let Some(active_team) = account_response.teams.iter().find(|team| team.active) else {
        error!("No active team found in account response!");
        return;
    };

    let active_character_id = u64::from(active_team.current_use);

    let Some(active_character) = character_group
        .characters
        .iter()
        .find(|character| character.base.id == active_character_id)
        .cloned()
    else {
        error!(
            "Active character with id {} is missing in CharacterGroup!",
            active_character_id
        );
        return;
    };

    let world_data = match CharacterWorldData::build_from_id(active_character_id) {
        Ok(world_data) => world_data,
        Err(message) => {
            error!(
                "Failed to load world character data for id {}: {}",
                active_character_id, message
            );
            return;
        }
    };

    spawn_party_player(
        &mut commands,
        &asset_server,
        active_character.clone(),
        world_data.clone(),
    );

    let mut occupied_party_positions = vec![PLAYER_SPAWN_POSITION];
    let excluded_entities = existing_party_entities.iter().collect::<Vec<_>>();

    for companion in character_group
        .characters
        .iter()
        .filter(|character| character.base.id != active_character_id)
    {
        let world_data = match CharacterWorldData::build_from_id(companion.base.id) {
            Ok(world_data) => world_data,
            Err(message) => {
                warn!(
                    "Failed to load world character data for companion id {}: {}",
                    companion.base.id, message
                );
                continue;
            }
        };

        let Some(position) = find_companion_spawn_position(
            PLAYER_SPAWN_POSITION,
            companion.base.id,
            &occupied_party_positions,
            &static_collider_query,
            &excluded_entities,
        ) else {
            warn!(
                "Could not find valid spawn location for companion id {}",
                companion.base.id
            );
            continue;
        };

        occupied_party_positions.push(position);
        spawn_party_companion(&mut commands, &asset_server, companion.clone(), world_data, position);
    }

    info!(
        "Placed active player character id {} using model '{}'",
        active_character_id, world_data.model_name
    );
}

/// Spawns a party player entity in the game world.
///
/// This function creates a dynamic entity representing a player character and
/// attaches various components for movement, collision, rendering, and other gameplay-related properties.
/// It also spawns a child entity for the player's 3D model.
///
/// # Parameters
///
/// - `commands`: A mutable reference to the [`Commands`] resource, used to spawn and configure entities.
/// - `asset_server`: A reference to the [`AssetServer`] resource, used to load game assets such as models.
/// - `character`: A [`Character`] object containing data about the player's character, such as stats or customization.
/// - `world_data`: A [`CharacterWorldData`] object containing world-specific data for the character, such as its display name and model name.
///
/// # Notes
///
/// - The player's 3D model is loaded using the `world_data.model_name` provided, and it is assumed to follow the path `dev/models/{model_name}#Scene0`.
/// - The player's initial position is specified by the constant `PLAYER_SPAWN_POSITION`.
/// - The ground detection logic uses a slightly scaled collider for broader detection coverage.
///
/// # See Also
/// - [`Commands`]: Used for entity creation and configuration.
/// - [`AssetServer`]: Handles loading of external assets.
/// - [`SceneRoot`]: Used to link 3D models to entities.
fn spawn_party_player(
    commands: &mut Commands,
    asset_server: &AssetServer,
    character: game_shared::models::character::Character,
    world_data: CharacterWorldData,
) {
    let model_asset_path = format!("dev/models/{}", world_data.model_name);
    let scene: Handle<Scene> = asset_server.load(format!("{model_asset_path}#Scene0"));

    let collider = Collider::capsule(0.4, 1.0);
    let mut ground_probe_shape = collider.clone();
    ground_probe_shape.set_scale(Vector::ONE * 0.98, 10);

    commands
        .spawn((
            PartySpawnedCharacter,
            Player,
            character,
            PlayerMovementStats::default(),
            Name::new(world_data.display_name),
            Transform::from_translation(PLAYER_SPAWN_POSITION),
            RigidBody::Dynamic,
            collider,
            CollisionLayers::from_bits(LAYER_PLAYER, LAYER_WORLD),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(2.0),
            ShapeCaster::new(
                ground_probe_shape,
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.3),
        ))
        .insert((
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(scene),
                Transform::from_xyz(0.0, -0.9, 0.0),
                Name::new("Player Scene"),
            ));
        });
}

/// Spawns a party companion entity in the game world with specified properties and components.
///
/// This function creates an entity in the game world that represents a companion for the player's party.
/// The companion is assigned a 3D model, physical properties for collision, and other gameplay-related components.
///
/// # Parameters
/// - `commands`: A mutable reference to the `Commands` object, used to spawn and manage entities in the game world.
/// - `asset_server`: A reference to the `AssetServer`, used to load 3D models or other assets.
/// - `character`: The character model representing the companion in the game. This is usually defined in the shared game models.
/// - `world_data`: A `CharacterWorldData` structure that contains information like the display name and model file associated with the companion.
/// - `position`: A `Vec3` representing the initial world position where the companion should be spawned.
///
/// # Asset Loading
/// The function uses the `asset_server` to load a 3D model whose path is constructed by appending `world_data.model_name`
/// to the prefix `"dev/models/"`. The model file is assumed to have a `Scene0` component that represents the main scene.
fn spawn_party_companion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    character: game_shared::models::character::Character,
    world_data: CharacterWorldData,
    position: Vec3,
) {
    let model_asset_path = format!("dev/models/{}", world_data.model_name);
    let scene: Handle<Scene> = asset_server.load(format!("{model_asset_path}#Scene0"));

    commands
        .spawn((
            PartySpawnedCharacter,
            PartyCompanion,
            character,
            Name::new(world_data.display_name),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::capsule(0.4, 1.0),
            CollisionLayers::from_bits(LAYER_COMPANION, LAYER_WORLD),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(2.0),
        ))
        .insert((
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(scene),
                Transform::from_xyz(0.0, -0.9, 0.0),
                Name::new("Companion Scene"),
            ));
        });
}


/// Finds a suitable spawn position for a companion character near a center location.
///
/// This function calculates potential spawn positions around a given `center` point for a companion
/// character, ensuring the position is clear and outside restricted areas or occupied positions.
/// The search uses a deterministic pattern based on a unique `character_id` and will attempt to find
/// a valid location within a configurable number of tries (`COMPANION_SPAWN_ATTEMPTS`).
///
/// # Parameters
/// - `center`: The central position (`Vec3`) around which the companion should spawn.
/// - `character_id`: A unique identifier for the character, used to seed the deterministic spawn position calculation.
/// - `occupied_party_positions`: A slice of `Vec3` representing positions already occupied by other party members.
/// - `static_collider_query`: A reference to a `Query` structure used to check for static colliders in the game world.
///   This query includes:
///   - `Entity`: The entity representing the collider.
///   - `Transform`: The spatial transform of the entity.
///   - `Collider`: The collider component of the entity.
///   - `Option<RigidBody>`: Optional rigid body of the entity.
///   - `Has<TestWorldFloor>`: Marker to target specific entities deemed as the world floor.
/// - `excluded_entities`: A slice of `Entity` values to exclude from collision checks (e.g., entities that should not influence spawn calculation).
///
/// # Returns
/// - `Option<Vec3>`: Returns `Some(Vec3)` representing a valid spawn position if one is found; otherwise, `None` if no valid position is available.
///
/// # Constants (assumed external)
/// - `COMPANION_SPAWN_ATTEMPTS`: Maximum number of attempts to find a valid spawn position.
/// - `COMPANION_SPAWN_MIN_DISTANCE`: Minimum distance from `center` where a companion may spawn.
/// - `COMPANION_SPAWN_MAX_DISTANCE`: Maximum distance from `center` where a companion may spawn.
/// - `COMPANION_SPAWN_HEIGHT`: Vertical offset for the spawn position above the game floor.
///
/// # Dependencies
/// - The function depends on an external utility `is_spawn_position_clear()` to verify the validity of each candidate position.
///
/// # Notes
/// - The function ensures deterministic results based on `character_id`, which is useful for consistent gameplay.
/// - A large number of attempts or overly restrictive conditions (e.g., many colliders) may result in no valid spawn position.
fn find_companion_spawn_position(
    center: Vec3,
    character_id: u64,
    occupied_party_positions: &[Vec3],
    static_collider_query: &Query<
        (Entity, &Transform, &Collider, Option<&RigidBody>, Has<TestWorldFloor>),
    >,
    excluded_entities: &[Entity],
) -> Option<Vec3> {
    let seed = seed_to_unit_interval(character_id);
    let phase = seed * PI * 2.0;
    let radius_span = COMPANION_SPAWN_MAX_DISTANCE - COMPANION_SPAWN_MIN_DISTANCE;

    for attempt in 0..COMPANION_SPAWN_ATTEMPTS {
        let attempt_ratio = attempt as f32 / (COMPANION_SPAWN_ATTEMPTS - 1) as f32;
        let radius = COMPANION_SPAWN_MIN_DISTANCE + radius_span * attempt_ratio;
        let angle = phase + attempt as f32 * 2.399_963_1;

        let candidate = center
            + Vec3::new(
                angle.cos() * radius,
                COMPANION_SPAWN_HEIGHT,
                angle.sin() * radius,
            );

        if is_spawn_position_clear(
            candidate,
            occupied_party_positions,
            static_collider_query,
            excluded_entities,
        ) {
            return Some(candidate);
        }
    }

    None
}

/// Checks if a candidate spawn position is clear of obstacles and meets the required clearance criteria.
///
/// This function verifies that the `candidate` position is not too close to any occupied positions
/// in the party or static colliders in the world, while excluding certain entities or floor-like colliders
/// from consideration. The function ensures that the candidate position adheres to the clearance rules
/// specified by constants `COMPANION_PARTY_CLEARANCE` and `COMPANION_SOLID_CLEARANCE`.
///
/// # Parameters
/// - `candidate`: The 3D position (`Vec3`) to check if it is a valid spawn position.
/// - `occupied_party_positions`: A slice of `Vec3` representing the positions that are already
///   occupied by party members.
/// - `static_collider_query`: A query that provides access to entities with their associated
///   transformations, colliders, rigid bodies, and the presence of the `TestWorldFloor` marker.
/// - `excluded_entities`: A slice of entities (`&[Entity]`) that should be excluded when checking
///   for collisions or proximity.
///
/// # Returns
/// - `true` if the candidate position satisfies all clearance conditions and is considered valid.
/// - `false` otherwise.
///
/// # Dependencies
/// This function relies on the `Vec3` type for 3D positions, the `Query` type for accessing
/// entity components, and the `Collider` type for collision checking. Constants `COMPANION_PARTY_CLEARANCE`
/// and `COMPANION_SOLID_CLEARANCE` must also be defined in the scope.
///
/// # Notes
/// - Ensure that `COMPANION_PARTY_CLEARANCE` and `COMPANION_SOLID_CLEARANCE` are properly set
///   to appropriate values for your application to define valid spawn clearances.
/// - The logic ignores dynamic colliders as they might move or change state.
fn is_spawn_position_clear(
    candidate: Vec3,
    occupied_party_positions: &[Vec3],
    static_collider_query: &Query<
        (Entity, &Transform, &Collider, Option<&RigidBody>, Has<TestWorldFloor>),
    >,
    excluded_entities: &[Entity],
) -> bool {
    if occupied_party_positions
        .iter()
        .any(|position| position.distance(candidate) < COMPANION_PARTY_CLEARANCE)
    {
        return false;
    }

    for (entity, transform, collider, rigid_body, is_floor) in static_collider_query.iter() {
        if excluded_entities.contains(&entity) || is_floor {
            continue;
        }

        let Some(rigid_body) = rigid_body else {
            continue;
        };

        if !rigid_body.is_static() {
            continue;
        }

        let distance = collider.distance_to_point(
            transform.translation,
            transform.rotation,
            candidate,
            true,
        );

        if distance < COMPANION_SOLID_CLEARANCE {
            return false;
        }
    }

    true
}

fn seed_to_unit_interval(seed: u64) -> f32 {
    let hashed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (hashed as f64 / u64::MAX as f64) as f32
}
