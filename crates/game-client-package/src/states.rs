use bevy::prelude::*;

/// Represents the different states of a client application lifecycle.
///
/// This enum is used to define and manage the various stages that a client application
/// can be in during its runtime. Each variant represents a specific state of the
/// application's lifecycle, enabling better organization and control flow.
///
/// # Variants
///
/// - `Booting`: The default state, representing the initial phase where the
///   application is starting up.
/// - `WindowVisible`: Represents the state where the application window is
///   visible to the user, indicating that the UI is initialized but other states may still
///   need preparation.
/// - `Before(BeforeState)`: Represents a transitional state where the application
///   is in the "before" phase. This variant contains a [BeforeState] struct to handle
///   specific logic associated with this phase.
/// - `Loading(LoadingState)`: Represents a state where the application is loading
///   resources or performing asynchronous work. This variant contains a [LoadingState]
///   struct to encapsulate details related to the loading process.
/// - `InGame(InGameState)`: Represents the state where the application is
///   actively running the main functionality (e.g., gameplay, simulation, etc.).
///   This variant contains an [InGameState] struct for managing in-game behavior.
///
/// # Traits
/// The `ClientState` enum implements the following traits:
///
/// * `States`: Enables integration with state management systems.
/// * `Default`: Provides a default value, which is `Booting`.
/// * `Debug`: Allows `ClientState` to be formatted using the `{:?}` formatter.
/// * `Clone`: Enables cloning of `ClientState` instances.
/// * `Eq`: Provides equality comparison for `ClientState` instances.
/// * `PartialEq`: Enables partial equality comparison for `ClientState`.
/// * `Hash`: Allows `ClientState` to be used in hashing-based collections (e.g., `HashMap`).
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ClientState {
    #[default]
    Booting,
    WindowVisible,
    Before(BeforeState),
    Loading(LoadingState),
    InGame(InGameState)
}

/// Represents the states that can occur before the main gameplay.
///
/// The `BeforeState` enum is used to define and manage the states in an application
/// that take place before entering gameplay, such as showing the splash screen or
/// navigating the main menu. This enum is marked with the `States` derive macro, which
/// likely allows it to be used in a state management system.
///
/// # Variants
///
/// - `SplashScreen`: The default state, representing the initial splash screen.
/// - `MainMenu`: Represents the main menu of the application.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BeforeState {
    #[default]
    SplashScreen,
    MainMenu
}

/// Represents the various states of a loading process within the application.
///
/// This enum is derived with several traits to support state management, debugging, cloning,
/// equality checks, partial equality, and hashing.
///
/// ## Variants
///
/// - `UiPreLoad`:
///   A state indicating that the UI is performing preloading tasks before the main loading process begins.
/// - `Begin`:
///   The initial default state where the loading process starts. This is the first state that
///   the application transitions into during loading.
/// - `Progress`:
///   A state signifying that the loading process is ongoing and tasks are in progress.
/// - `After`:
///   A state indicating that the loading process has completed and post-loading actions are being executed.
///
/// ## Usage
///
/// This enum is typically used in state management systems where different phases of a
/// loading process need to be tracked and handled separately.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum LoadingState {
    UiPreLoad,
    #[default]
    Begin,
    Progress,
    After
}

/// Represents the various states of the game.
///
/// This enum is used to track the current state of the game, allowing the application
/// to manage different behaviors based on whether the game is actively being played
/// or paused. The `InGameState` enum derives several common traits for convenient
/// use in state management and comparisons.
///
/// # Variants
///
/// - `Playing`: The default state, representing that the game is currently in progress.
/// - `Paused`: Represents that the game is temporarily halted.
/// 
/// ## Usage
///
/// This enum is typically used in state management systems where different phases of
/// in game phases need to be tracked and handled separately, such as pausing the game.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum InGameState {
    #[default]
    Playing,
    Paused
}

/// Checks if the current `ClientState` matches the `Before` state.
///
/// # Arguments
/// * `state` - A `Res<State<ClientState>>` reference to the current state.
///
/// # Returns
/// * `true` if the current state is [ClientState::Before], otherwise `false`.
///
/// # Example
/// ```ignore
/// fn build(&self, app: &mut App) {
///     app.add_system(StartUp, example_sytsem.run_if(is_before_state));
/// }
/// ```
pub fn is_before_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::Before(_))
}

/// Checks if the current `ClientState` matches the `Loading` state.
///
/// # Arguments
/// * `state` - A `Res<State<ClientState>>` reference to the current state.
///
/// # Returns
/// * `true` if the current state is [ClientState::Loading], otherwise `false`.
///
/// # Example
/// ```ignore
/// fn build(&self, app: &mut App) {
///     app.add_system(StartUp, example_sytsem.run_if(is_loading_state));
/// }
/// ```
pub fn is_loading_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::Loading(_))
}

/// Checks if the current `ClientState` matches the `InGame` state.
///
/// # Arguments
/// * `state` - A `Res<State<ClientState>>` reference to the current state.
///
/// # Returns
/// * `true` if the current state is [ClientState::InGame], otherwise `false`.
///
/// # Example
/// ```ignore
/// fn build(&self, app: &mut App) {
///    app.add_system(StartUp, example_sytsem.run_if(is_in_game_state));
/// }
/// ```
pub fn is_in_game_state(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::InGame(_))
}