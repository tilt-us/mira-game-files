use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use game_shared::models::camera::{
    CameraOcclusionFade, OcclusionFadeMaterialBinding, OcclusionFadeMaterialOwner,
    OrbitFollowCamera,
};
use game_shared::models::player::Player;

const OCCLUSION_ALPHA_BLOCK: f32 = 0.05;
const OCCLUSION_SEGMENT_EPSILON: f32 = 0.02;
const OCCLUSION_BLOCK_DISTANCE_TO_VIEW_LINE: f32 = 0.45;

/// Tags material-carrying mesh entities with the occlusion root character entity they belong to.
///
/// The system climbs each mesh entity's parent chain until it finds an ancestor that has
/// [`CameraOcclusionFade`]. That ancestor becomes the owner root used for subsequent fade updates.
pub fn attach_occlusion_material_owners(
    mut commands: Commands,
    parent_query: Query<&ChildOf>,
    mut material_entities: Query<
        Entity,
        (
            With<MeshMaterial3d<StandardMaterial>>,
            Without<OcclusionFadeMaterialOwner>,
        ),
    >,
    occlusion_roots: Query<(), With<CameraOcclusionFade>>,
) {
    for entity in &mut material_entities {
        let Some(root) = find_occlusion_root(entity, &parent_query, &occlusion_roots) else {
            continue;
        };

        commands
            .entity(entity)
            .insert(OcclusionFadeMaterialOwner { root });
    }
}

/// Clones shared GLTF materials so each party submesh can be faded independently.
///
/// GLTF scenes often share material handles across many entities. This system creates
/// per-entity copies and stores the baseline alpha/alpha-mode in
/// [`OcclusionFadeMaterialBinding`] to support reversible runtime fading.
pub fn bind_occlusion_material_instances(
    mut commands: Commands,
    mut material_entities: Query<
        (
            Entity,
            &mut MeshMaterial3d<StandardMaterial>,
            &OcclusionFadeMaterialOwner,
        ),
        Without<OcclusionFadeMaterialBinding>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut mesh_material, owner) in &mut material_entities {
        let Some(source_material) = materials.get(&mesh_material.0).cloned() else {
            continue;
        };

        let base_alpha = source_material.base_color.alpha();
        let base_alpha_mode = source_material.alpha_mode;
        let new_material = materials.add(source_material);

        mesh_material.0 = new_material.clone();

        commands
            .entity(entity)
            .insert(OcclusionFadeMaterialBinding {
                owner_root: owner.root,
                material: new_material,
                base_alpha,
                base_alpha_mode,
            });
    }
}

/// Updates per-character target and current alpha based on line-of-sight blocking.
///
/// Behavior:
/// - Active player entity always resolves to full opacity (`1.0`).
/// - Non-player party members resolve to `0.05` only when they block the camera-to-player line.
/// - `current_alpha` eases toward `target_alpha` using exponential smoothing derived from
///   [`OrbitFollowCamera::occlusion_fade_speed`].
pub fn update_character_occlusion_targets(
    time: Res<Time>,
    camera_query: Query<(&GlobalTransform, &OrbitFollowCamera), With<Camera3d>>,
    player_query: Query<(Entity, &GlobalTransform), With<Player>>,
    mut fades: Query<(Entity, &GlobalTransform, &mut CameraOcclusionFade)>,
) {
    let Some((player_entity, player_transform)) = player_query.iter().next() else {
        return;
    };

    let Some((camera_transform, orbit_camera)) = camera_query.iter().next() else {
        return;
    };

    let player_target = player_transform.translation() + Vec3::Y * orbit_camera.target_height;
    let camera_position = camera_transform.translation();

    let fade_alpha =
        1.0 - (-orbit_camera.occlusion_fade_speed.max(0.001) * time.delta_secs()).exp();

    for (entity, entity_transform, mut fade) in &mut fades {
        fade.target_alpha = if entity == player_entity {
            1.0
        } else {
            compute_occlusion_target_alpha(
                entity_transform.translation(),
                camera_position,
                player_target,
            )
        };
        fade.current_alpha = fade
            .current_alpha
            .lerp(fade.target_alpha, fade_alpha.clamp(0.0, 1.0));
    }
}

/// Applies the latest occlusion alpha to all bound submesh materials.
///
/// This includes all character parts (for example hair, teeth, eyes, and clothing), because
/// each submesh material is driven by [`OcclusionFadeMaterialBinding`].
pub fn apply_character_occlusion_material_alpha(
    mut materials: ResMut<Assets<StandardMaterial>>,
    fade_query: Query<&CameraOcclusionFade>,
    mut bindings: Query<&mut OcclusionFadeMaterialBinding>,
) {
    for binding in &mut bindings {
        let Ok(fade) = fade_query.get(binding.owner_root) else {
            continue;
        };

        let Some(material) = materials.get_mut(&binding.material) else {
            continue;
        };

        // Apply fade as a global alpha cap per material (instead of multiplying),
        // so opaque face/hair/teeth materials fade consistently with body parts.
        let blended_alpha = fade.current_alpha.clamp(0.0, binding.base_alpha);

        material.base_color.set_alpha(blended_alpha);
        material.alpha_mode = if blended_alpha + 0.001 < binding.base_alpha {
            AlphaMode::Blend
        } else {
            binding.base_alpha_mode
        };
    }
}

/// Walks up the parent hierarchy to find the closest ancestor marked with [`CameraOcclusionFade`].
fn find_occlusion_root(
    entity: Entity,
    parent_query: &Query<&ChildOf>,
    occlusion_roots: &Query<(), With<CameraOcclusionFade>>,
) -> Option<Entity> {
    let mut current = entity;

    while let Ok(parent) = parent_query.get(current) {
        current = parent.parent();
        if occlusion_roots.contains(current) {
            return Some(current);
        }
    }

    None
}

/// Resolves the occlusion alpha for a character relative to the camera-to-player segment.
///
/// Returns:
/// - `0.05` if the character lies between camera and player and is close enough to the view line.
/// - `1.0` otherwise.
fn compute_occlusion_target_alpha(
    character_position: Vec3,
    camera_position: Vec3,
    player_position: Vec3,
) -> f32 {
    let segment = player_position - camera_position;
    let segment_length_squared = segment.length_squared();
    if segment_length_squared <= f32::EPSILON {
        return 1.0;
    }

    let projected = (character_position - camera_position).dot(segment) / segment_length_squared;
    if !(OCCLUSION_SEGMENT_EPSILON..=(1.0 - OCCLUSION_SEGMENT_EPSILON)).contains(&projected) {
        return 1.0;
    }

    let closest_point = camera_position + segment * projected;
    let distance_to_view_line = character_position.distance(closest_point);
    if distance_to_view_line <= OCCLUSION_BLOCK_DISTANCE_TO_VIEW_LINE {
        OCCLUSION_ALPHA_BLOCK
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occlusion_block_sets_fixed_alpha_only_when_on_view_line() {
        let camera = Vec3::new(0.0, 1.0, -5.0);
        let player = Vec3::new(0.0, 1.0, 0.0);
        let blocking = Vec3::new(0.1, 1.0, -2.0);
        let side = Vec3::new(1.2, 1.0, -2.0);

        assert_eq!(
            compute_occlusion_target_alpha(blocking, camera, player),
            OCCLUSION_ALPHA_BLOCK
        );
        assert_eq!(compute_occlusion_target_alpha(side, camera, player), 1.0);
    }

    #[test]
    fn occlusion_only_applies_when_character_is_between_camera_and_player() {
        let camera = Vec3::new(0.0, 1.0, -5.0);
        let player = Vec3::new(0.0, 1.0, 0.0);

        let between = Vec3::new(0.1, 1.0, -2.0);
        let behind_player = Vec3::new(0.1, 1.0, 1.0);
        let behind_camera = Vec3::new(0.1, 1.0, -6.0);

        assert!(compute_occlusion_target_alpha(between, camera, player) < 1.0);
        assert_eq!(
            compute_occlusion_target_alpha(behind_player, camera, player),
            1.0
        );
        assert_eq!(
            compute_occlusion_target_alpha(behind_camera, camera, player),
            1.0
        );
    }
}
