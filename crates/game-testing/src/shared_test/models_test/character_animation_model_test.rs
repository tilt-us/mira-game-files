use bevy::prelude::*;
use game_shared::models::character::animation::{CharacterAnimationNodes, CharacterAnimationState};

fn node(index: usize) -> AnimationNodeIndex {
    AnimationNodeIndex::new(index)
}

#[test]
fn node_for_state_uses_expected_fallback_chains() {
    let nodes = CharacterAnimationNodes {
        idle: None,
        idle_alt: Some(node(1)),
        slow_walk: None,
        walk: Some(node(3)),
        sprint: None,
        jump: None,
    };

    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Idle),
        Some(node(1))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::SlowWalk),
        Some(node(3))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Walk),
        Some(node(3))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Sprint),
        Some(node(3))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Jump),
        None
    );
}

#[test]
fn node_for_state_prefers_direct_mappings_before_fallbacks() {
    let nodes = CharacterAnimationNodes {
        idle: Some(node(0)),
        idle_alt: Some(node(1)),
        slow_walk: Some(node(2)),
        walk: Some(node(3)),
        sprint: Some(node(4)),
        jump: Some(node(5)),
    };

    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Idle),
        Some(node(0))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::SlowWalk),
        Some(node(2))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Walk),
        Some(node(3))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Sprint),
        Some(node(4))
    );
    assert_eq!(
        nodes.node_for_state(CharacterAnimationState::Jump),
        Some(node(5))
    );
}

#[test]
fn fallback_uses_priority_order_and_returns_none_when_empty() {
    let mut nodes = CharacterAnimationNodes::default();
    assert_eq!(nodes.fallback(), None);

    nodes.jump = Some(node(5));
    assert_eq!(nodes.fallback(), Some(node(5)));

    nodes.sprint = Some(node(4));
    assert_eq!(nodes.fallback(), Some(node(4)));

    nodes.slow_walk = Some(node(2));
    assert_eq!(nodes.fallback(), Some(node(2)));

    nodes.walk = Some(node(3));
    assert_eq!(nodes.fallback(), Some(node(3)));

    nodes.idle_alt = Some(node(1));
    assert_eq!(nodes.fallback(), Some(node(1)));

    nodes.idle = Some(node(0));
    assert_eq!(nodes.fallback(), Some(node(0)));
}
