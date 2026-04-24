use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use game_client_package::states::{
    BeforeState, ClientState, InGameState, LoadingState, is_before_state, is_in_game_state,
    is_loading_state,
};

#[derive(Resource, Default, Clone)]
struct StateCheckResult {
    before: bool,
    loading: bool,
    in_game: bool,
}

fn evaluate_before_state(mut result: ResMut<StateCheckResult>, state: Res<State<ClientState>>) {
    result.before = is_before_state(state);
}

fn evaluate_loading_state(mut result: ResMut<StateCheckResult>, state: Res<State<ClientState>>) {
    result.loading = is_loading_state(state);
}

fn evaluate_in_game_state(mut result: ResMut<StateCheckResult>, state: Res<State<ClientState>>) {
    result.in_game = is_in_game_state(state);
}

fn run_checks_for_state(target: ClientState) -> StateCheckResult {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(StateCheckResult::default());
    app.add_systems(
        Update,
        (
            evaluate_before_state,
            evaluate_loading_state,
            evaluate_in_game_state,
        ),
    );

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(target);
    app.update();

    app.world().resource::<StateCheckResult>().clone()
}

#[test]
fn state_predicates_match_expected_variants() {
    let before = run_checks_for_state(ClientState::Before(BeforeState::MainMenu));
    assert!(before.before);
    assert!(!before.loading);
    assert!(!before.in_game);

    let loading = run_checks_for_state(ClientState::Loading(LoadingState::Progress));
    assert!(!loading.before);
    assert!(loading.loading);
    assert!(!loading.in_game);

    let in_game = run_checks_for_state(ClientState::InGame(InGameState::Paused));
    assert!(!in_game.before);
    assert!(!in_game.loading);
    assert!(in_game.in_game);
}

#[test]
fn state_enums_have_expected_defaults() {
    assert_eq!(ClientState::default(), ClientState::Booting);
    assert_eq!(BeforeState::default(), BeforeState::SplashScreen);
    assert_eq!(LoadingState::default(), LoadingState::Begin);
    assert_eq!(InGameState::default(), InGameState::Playing);
}
