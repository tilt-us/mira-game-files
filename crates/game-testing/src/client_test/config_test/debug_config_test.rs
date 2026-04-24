use bevy::prelude::App;
use game_client_package::config::debug_config::AppBuildInfo;

#[test]
fn app_build_info_can_be_inserted_as_resource() {
    let mut app = App::new();
    let build_info = AppBuildInfo {
        name: String::from("Mira: Fallen Gates"),
        version: String::from("0.1.0"),
        bevy_version: String::from("0.18.1"),
    };

    app.insert_resource(build_info.clone());

    let stored = app.world().resource::<AppBuildInfo>();
    assert_eq!(stored.name, "Mira: Fallen Gates");
    assert_eq!(stored.version, "0.1.0");
    assert_eq!(stored.bevy_version, "0.18.1");
}
