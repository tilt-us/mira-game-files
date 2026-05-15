use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy::scene::SceneRoot;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::character::world::CharacterWorldData;
use game_shared::models::http::account::AccountResource;
use game_shared::models::player::{Player, PlayerMovementStats};

/// Places the active player character into the world.
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
/// 5. Spawn a `Player` entity with character data, scene root and Avian3D physics body.
///
/// # Parameters
/// - `commands`: Used to despawn existing player entities and spawn the new one.
/// - `account_resource`: Contains account response data including teams.
/// - `character_group`: Contains the generated character collection from account data.
/// - `asset_server`: Loads the GLB scene referenced by `model_name`.
/// - `existing_players`: Query for already spawned player entities.
///
/// # Behavior Notes
/// - Existing `Player` entities are despawned before spawning the current active player.
/// - If required data is missing (no account, no active team, missing character, missing
///   world definition file), an error is logged and no spawn occurs.
pub fn place_to_world(
    mut commands: Commands,
    account_resource: Res<AccountResource>,
    character_group: Res<CharacterGroup>,
    asset_server: Res<AssetServer>,
    existing_players: Query<Entity, With<Player>>,
) {
    for entity in &existing_players {
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

    let model_asset_path = format!("dev/models/{}", world_data.model_name);
    let scene: Handle<Scene> = asset_server.load(format!("{model_asset_path}#Scene0"));

    let collider = Collider::capsule(0.4, 1.0);
    let mut ground_probe_shape = collider.clone();
    ground_probe_shape.set_scale(Vector::ONE * 0.98, 10);

    commands
        .spawn((
            Player,
            active_character,
            PlayerMovementStats::default(),
            Name::new(world_data.display_name),
            Transform::from_xyz(0.0, 0.9, 0.0),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            RigidBody::Dynamic,
            collider,
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
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(scene),
                Transform::from_xyz(0.0, -0.9, 0.0),
                Name::new("Player Scene"),
            ));
        });

    info!(
        "Placed active player character id {} using model '{}'",
        active_character_id, model_asset_path
    );
}
