use super::jobs::{
    ConversionRequest, ConversionScheduler, FilterScheduler, LoadScheduler, OperationRequest,
    OperationScheduler, SearchScheduler,
};
use super::locations::default_locations;
use super::state::{LoadedRow, SelectionState, SharedRows, SharedSelection, SortSpec, TabManager};
use super::{FileRow, LocationRow, MainWindow, TabRow};
use crate::clipboard::ClipboardStore;
use crate::filesystem::ListingOptions;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, atomic::AtomicU64};

pub(in crate::desktop) struct AppContext {
    pub(in crate::desktop) ui_weak: slint::Weak<MainWindow>,
    pub(in crate::desktop) entries: Rc<VecModel<FileRow>>,
    pub(in crate::desktop) locations: Rc<VecModel<LocationRow>>,
    pub(in crate::desktop) tab_model: Rc<VecModel<TabRow>>,
    pub(in crate::desktop) tabs: Rc<std::cell::RefCell<TabManager>>,
    pub(in crate::desktop) directory_rows: SharedRows,
    pub(in crate::desktop) selection: SharedSelection,
    pub(in crate::desktop) filter_generation: Arc<AtomicU64>,
    pub(in crate::desktop) sort_spec: Arc<Mutex<SortSpec>>,
    pub(in crate::desktop) listing_options: Arc<Mutex<ListingOptions>>,
    pub(in crate::desktop) clipboard: Option<Arc<ClipboardStore>>,
    pub(in crate::desktop) load_scheduler: Option<Arc<LoadScheduler>>,
    pub(in crate::desktop) filter_scheduler: Option<Arc<FilterScheduler>>,
    pub(in crate::desktop) operation_scheduler: Option<Arc<OperationScheduler>>,
    pub(in crate::desktop) conversion_scheduler: Option<Arc<ConversionScheduler>>,
    pub(in crate::desktop) search_scheduler: Option<Arc<SearchScheduler>>,
    pub(in crate::desktop) pending_operation: Rc<std::cell::RefCell<Option<OperationRequest>>>,
    pub(in crate::desktop) pending_conversion: Rc<std::cell::RefCell<Option<ConversionRequest>>>,
}

impl AppContext {
    pub(in crate::desktop) fn new(ui: &MainWindow, initial_path: PathBuf) -> Self {
        let entries = Rc::new(VecModel::<FileRow>::default());
        ui.set_entries(ModelRc::from(entries.clone()));
        let locations = Rc::new(VecModel::<LocationRow>::default());
        ui.set_locations(ModelRc::from(locations.clone()));
        let tab_model = Rc::new(VecModel::<TabRow>::default());
        ui.set_tabs(ModelRc::from(tab_model.clone()));

        locations.set_vec(
            default_locations(&initial_path)
                .into_iter()
                .map(|location| LocationRow {
                    label: SharedString::from(location.label),
                    path: SharedString::from(location.path.to_string_lossy().to_string()),
                })
                .collect::<Vec<_>>(),
        );
        ui.set_current_path(SharedString::from(
            initial_path.to_string_lossy().to_string(),
        ));
        ui.set_status_text("Carregando…".into());

        let ui_weak = ui.as_weak();
        let tabs = Rc::new(std::cell::RefCell::new(TabManager::new(initial_path)));
        let directory_rows: SharedRows = Arc::new(Mutex::new(Arc::from(Vec::<LoadedRow>::new())));
        let selection: SharedSelection = Arc::new(Mutex::new(SelectionState::default()));
        let filter_generation = Arc::new(AtomicU64::new(0));
        let sort_spec = Arc::new(Mutex::new(SortSpec::default()));
        let listing_options = Arc::new(Mutex::new(ListingOptions::default()));
        let clipboard = ClipboardStore::new().map(Arc::new).ok();
        let search_scheduler = SearchScheduler::new().map(Arc::new).ok();
        let load_scheduler = LoadScheduler::new(
            ui_weak.clone(),
            Arc::clone(&directory_rows),
            Arc::clone(&selection),
            Arc::clone(&filter_generation),
            Arc::clone(&sort_spec),
            Arc::clone(&listing_options),
            search_scheduler.clone(),
        )
        .map(Arc::new)
        .ok();
        let filter_scheduler = FilterScheduler::new(
            ui_weak.clone(),
            Arc::clone(&directory_rows),
            Arc::clone(&selection),
            Arc::clone(&filter_generation),
        )
        .map(Arc::new)
        .ok();
        let pending_operation = Rc::new(std::cell::RefCell::new(None::<OperationRequest>));
        let pending_conversion = Rc::new(std::cell::RefCell::new(None::<ConversionRequest>));
        let operation_scheduler = OperationScheduler::new(ui_weak.clone(), load_scheduler.clone())
            .map(Arc::new)
            .ok();
        let conversion_scheduler =
            ConversionScheduler::new(ui_weak.clone(), load_scheduler.clone())
                .map(Arc::new)
                .ok();
        Self {
            ui_weak,
            entries,
            locations,
            tab_model,
            tabs,
            directory_rows,
            selection,
            filter_generation,
            sort_spec,
            listing_options,
            clipboard,
            load_scheduler,
            filter_scheduler,
            operation_scheduler,
            conversion_scheduler,
            search_scheduler,
            pending_operation,
            pending_conversion,
        }
    }
}
