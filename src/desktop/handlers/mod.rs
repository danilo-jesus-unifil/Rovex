mod confirmation;
mod conversions;
mod dialogs;
mod filter;
mod lifecycle;
mod navigation;
mod operations;
mod selection;
mod sorting;
mod visibility;

use super::context::AppContext;

pub(in crate::desktop) use navigation::update_tab_visuals;

pub(in crate::desktop) fn register_all(ctx: &AppContext) {
    navigation::register(ctx);
    selection::register(ctx);
    operations::register(ctx);
    conversions::register(ctx);
    confirmation::register(ctx);
    lifecycle::register(ctx);
    filter::register(ctx);
    sorting::register(ctx);
    visibility::register(ctx);
}
