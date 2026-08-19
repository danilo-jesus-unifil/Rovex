mod clipboard;
mod confirmation;
mod conversions;
mod dialogs;
mod dragdrop;
mod filter;
mod lifecycle;
mod navigation;
mod operations;
mod preview;
mod properties;
mod search;
mod selection;
mod sorting;
mod terminal;
mod visibility;

use super::context::AppContext;

pub(in crate::desktop) use navigation::update_tab_visuals;

pub(in crate::desktop) fn register_all(ctx: &AppContext) {
    navigation::register(ctx);
    dragdrop::register(ctx);
    selection::register(ctx);
    operations::register(ctx);
    preview::register(ctx);
    properties::register(ctx);
    search::register(ctx);
    conversions::register(ctx);
    confirmation::register(ctx);
    clipboard::register(ctx);
    lifecycle::register(ctx);
    filter::register(ctx);
    sorting::register(ctx);
    terminal::register(ctx);
    visibility::register(ctx);
}
