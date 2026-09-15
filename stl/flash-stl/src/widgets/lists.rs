//! List widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(46, "SectionList", WidgetCategory::Lists, "List with section headers", "UITableView", "RecyclerView", true, false, PrimaryProp::None, false),
    widget!(47, "LazyColumn", WidgetCategory::Lists, "Vertically lazy list", "UICollectionView", "RecyclerView", true, false, PrimaryProp::None, false),
    widget!(48, "LazyRow", WidgetCategory::Lists, "Horizontally lazy list", "UICollectionView", "RecyclerView", true, false, PrimaryProp::None, false),
    widget!(49, "RefreshIndicator", WidgetCategory::Lists, "Pull-to-refresh wrapper", "UIRefreshControl", "SwipeRefreshLayout", true, false, PrimaryProp::None, false),
    widget!(50, "ListTile", WidgetCategory::Lists, "Standard list row", "UITableViewCell", "MaterialListItem", true, true, PrimaryProp::Title, true),
];
