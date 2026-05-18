use bevy::prelude::*;
use serde::Deserialize;

/// Maps a logical animation key to its clip index in a character GLB.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CharacterAnimation {
    pub key: String,
    pub index: usize,
}

/// Temporary setup data used to build runtime animation state after the GLTF scene is spawned.
#[derive(Component, Debug, Clone)]
pub struct CharacterAnimationLoadout {
    pub model_asset_path: String,
    pub clips: Vec<CharacterAnimation>,
}

/// Runtime animation controller stored on the party root entity.
#[derive(Component, Debug, Clone)]
pub struct CharacterAnimationController {
    pub animation_player_entity: Entity,
    pub nodes: CharacterAnimationNodes,
    pub current_state: CharacterAnimationState,
}

/// Supported locomotion states for character animation selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterAnimationState {
    Idle,
    SlowWalk,
    Walk,
    Sprint,
    Jump,
}

/// Animation node mapping resolved from character JSON animation keys.
#[derive(Debug, Clone, Default)]
pub struct CharacterAnimationNodes {
    pub idle: Option<AnimationNodeIndex>,
    pub idle_alt: Option<AnimationNodeIndex>,
    pub slow_walk: Option<AnimationNodeIndex>,
    pub walk: Option<AnimationNodeIndex>,
    pub sprint: Option<AnimationNodeIndex>,
    pub jump: Option<AnimationNodeIndex>,
}

impl CharacterAnimationNodes {
    pub fn node_for_state(&self, state: CharacterAnimationState) -> Option<AnimationNodeIndex> {
        match state {
            CharacterAnimationState::Idle => self.idle.or(self.idle_alt),
            CharacterAnimationState::SlowWalk => self.slow_walk.or(self.walk).or(self.idle),
            CharacterAnimationState::Walk => self.walk.or(self.slow_walk).or(self.idle),
            CharacterAnimationState::Sprint => self.sprint.or(self.walk).or(self.idle),
            CharacterAnimationState::Jump => self.jump.or(self.idle),
        }
    }

    pub fn fallback(&self) -> Option<AnimationNodeIndex> {
        self.idle
            .or(self.idle_alt)
            .or(self.walk)
            .or(self.slow_walk)
            .or(self.sprint)
            .or(self.jump)
    }
}
