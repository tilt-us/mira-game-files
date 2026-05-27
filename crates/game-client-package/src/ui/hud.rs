use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlInit, HtmlInnerContent};
use bevy_extended_ui::widgets::Headline;
use bevy_extended_ui_macros::html_fn;

#[html_fn("version_set")]
fn version_set(
    In(_event): In<HtmlInit>,
    mut q: Query<(&mut Headline, &mut HtmlInnerContent)>,
) {
    let version = env!("CARGO_PKG_VERSION");

    for (mut headline, mut inner) in q.iter_mut() {
        if inner.inner_bindings().iter().any(|b| b.contains("game_version")) {
            headline.text = version.to_string();
            inner.set_inner_text(version);
        }
    }
}