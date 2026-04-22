use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ClientState {
    #[default]
    Booting,
    WindowVisible,
    Before(BeforeState),
    Loading(LoadingState),
    InGame(InGameState)
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BeforeState {
    #[default]
    SplashScreen,
    MainMenu
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum LoadingState {
    UiPreLoad,
    #[default]
    Begin,
    Progress,
    After
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum InGameState {
    #[default]
    Playing,
    Paused
}

pub fn is_before_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::Before(_))
}

pub fn is_loading_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::Loading(_))
}

pub fn is_in_game_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::InGame(_))
}