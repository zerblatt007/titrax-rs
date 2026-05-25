use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{
    glib, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox,
    ListBoxRow, Orientation, PolicyType, ScrolledWindow, SelectionMode,
};
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::app::AppState;
use crate::config::{Config, LastActive};
use crate::data;
use crate::sort;

pub fn build_window(app: &Application) {
    let config = Config::load();
    let mut state = AppState::new(config.font_size);
    let _ = data::ensure_data_dir();
    state.load_today();

    install_local_icon_paths();

    // Auto-resume last active project if it was today
    if let Some(ref last) = config.last_active {
        if last.date == data::today_string() {
            if let Some(idx) = state
                .projects
                .iter()
                .position(|p| p.name == last.project_name)
            {
                state.select_project(idx);
            }
        }
    }

    let state = Rc::new(RefCell::new(state));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("TimeTracker {}", env!("CARGO_PKG_VERSION")))
        .default_width(config.window_width)
        .default_height(config.window_height)
        .build();
    window.set_icon_name(Some("titrax"));

    let root_box = GtkBox::new(Orientation::Horizontal, 8);
    root_box.set_margin_top(8);
    root_box.set_margin_bottom(8);
    root_box.set_margin_start(8);
    root_box.set_margin_end(8);

    // Left area: total display + project list.
    let left_box = GtkBox::new(Orientation::Vertical, 4);
    left_box.set_hexpand(true);

    // Right area: controls stacked vertically.
    let side_box = GtkBox::new(Orientation::Vertical, 4);
    let controls_box = GtkBox::new(Orientation::Vertical, 4);
    let plus_minus_box = GtkBox::new(Orientation::Horizontal, 4);

    // Track the highlighted (selected) row independently of active_index.
    let selected_index: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    let suppress_selection: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // Button stack
    let btn_plus = Button::with_label("+");
    let btn_minus = Button::with_label("-");
    let btn_add = Button::with_label("Add");
    let btn_sort = Button::with_label("Sort A-Å");
    let btn_pause = Button::with_label("Pause");
    let btn_edit_time = Button::with_label("Edit Time");
    let btn_delete = Button::with_label("Delete");
    let btn_move5 = Button::with_label("Move 5 min");
    plus_minus_box.append(&btn_plus);
    plus_minus_box.append(&btn_minus);
    controls_box.append(&plus_minus_box);
    controls_box.append(&btn_add);
    controls_box.append(&btn_sort);
    controls_box.append(&btn_edit_time);
    controls_box.append(&btn_delete);
    controls_box.append(&btn_move5);
    side_box.append(&controls_box);

    let total_label = Label::new(None);
    total_label.set_xalign(1.0);
    total_label.set_margin_top(4);
    total_label.set_margin_bottom(2);

    // Keep Pause in the upper-left corner, above the total display.
    left_box.append(&btn_pause);
    left_box.append(&total_label);

    // Scrolled project list
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Single);
    list_box.add_css_class("compact-project-list");
    install_compact_list_css();
    scrolled.set_child(Some(&list_box));
    left_box.append(&scrolled);

    root_box.append(&left_box);
    root_box.append(&side_box);
    window.set_child(Some(&root_box));

    // Initial button sensitivity
    refresh_button_sensitivity(
        &btn_plus,
        &btn_minus,
        &btn_pause,
        &btn_edit_time,
        &btn_delete,
        &btn_move5,
        &state,
        &selected_index,
    );

    // Populate list with today's projects
    populate_list(&list_box, &state, &selected_index, &suppress_selection);
    update_total_label(&total_label, &state);

    install_drag_reorder(
        &list_box,
        &state,
        &selected_index,
        &suppress_selection,
        &total_label,
        &btn_plus,
        &btn_minus,
        &btn_pause,
        &btn_edit_time,
        &btn_delete,
        &btn_move5,
    );

    // --- Signal: row selected (highlights a row without activating it) ---
    {
        let selected_index = selected_index.clone();
        let state = state.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete = btn_delete.clone();
        let btn_move5 = btn_move5.clone();
        let suppress_selection = suppress_selection.clone();
        list_box.connect_row_selected(move |_, row_opt| {
            if suppress_selection.get() {
                return;
            }
            selected_index.set(row_opt.map(|r| r.index() as usize));
            refresh_button_sensitivity(
                &btn_plus,
                &btn_minus,
                &btn_pause,
                &btn_edit_time,
                &btn_delete,
                &btn_move5,
                &state,
                &selected_index,
            );
        });
    }

    // --- Signal: row activated (left-click) ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete = btn_delete.clone();
        let btn_move5 = btn_move5.clone();
        let selected_index = selected_index.clone();
        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let mut s = state.borrow_mut();
            if s.active_index != Some(index) {
                s.select_project(index);
            }
            drop(s);
            update_list_appearance(&list_box_clone, &state);
            refresh_button_sensitivity(
                &btn_plus,
                &btn_minus,
                &btn_pause,
                &btn_edit_time,
                &btn_delete,
                &btn_move5,
                &state,
                &selected_index,
            );
        });
    }

    // --- Signal: right-click on hovered row transfers 5 minutes from active ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete = btn_delete.clone();
        let btn_move5 = btn_move5.clone();
        let selected_index = selected_index.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(3);
        gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);
        gesture.connect_pressed(move |gesture, _, _x, y| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            if let Some(row) = list_box_clone.row_at_y(y as i32) {
                let index = row.index() as usize;
                list_box_clone.select_row(Some(&row));
                selected_index.set(Some(index));
                let active = state.borrow().active_index;
                if let Some(from) = active {
                    if from != index {
                        state.borrow_mut().transfer_minutes(from, index, 5);
                        state.borrow().save_times();
                        update_list_appearance(&list_box_clone, &state);
                        update_total_label(&total_label, &state);
                        refresh_button_sensitivity(
                            &btn_plus,
                            &btn_minus,
                            &btn_pause,
                            &btn_edit_time,
                            &btn_delete,
                            &btn_move5,
                            &state,
                            &selected_index,
                        );
                    }
                }
            }
        });
        list_box.add_controller(gesture);
    }

    // --- Signal: + button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let selected_index = selected_index.clone();
        btn_plus.connect_clicked(move |_| {
            if let Some(idx) = selected_index.get() {
                state.borrow_mut().increment_minutes(idx, 10);
                update_list_appearance(&list_box_clone, &state);
                update_total_label(&total_label, &state);
                state.borrow().save_times();
            }
        });
    }

    // --- Signal: - button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let selected_index = selected_index.clone();
        btn_minus.connect_clicked(move |_| {
            if let Some(idx) = selected_index.get() {
                state.borrow_mut().increment_minutes(idx, -10);
                update_list_appearance(&list_box_clone, &state);
                update_total_label(&total_label, &state);
                state.borrow().save_times();
            }
        });
    }

    // --- Signal: Add button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let selected_index = selected_index.clone();
        let suppress_selection = suppress_selection.clone();
        let window_clone = window.clone();
        btn_add.connect_clicked(move |_| {
            show_add_dialog(
                &window_clone,
                &state,
                &list_box_clone,
                &selected_index,
                &suppress_selection,
                &total_label,
            );
        });
    }

    // --- Signal: Sort button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let selected_index = selected_index.clone();
        let suppress_selection = suppress_selection.clone();
        btn_sort.connect_clicked(move |_| {
            {
                let mut s = state.borrow_mut();
                sort::sort_projects(&mut s.projects);
                s.active_index = None;
                s.paused = true;
            }
            populate_list(
                &list_box_clone,
                &state,
                &selected_index,
                &suppress_selection,
            );
            update_total_label(&total_label, &state);
            state.borrow().save_projects();
        });
    }

    // --- Signal: Pause button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause_c = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete = btn_delete.clone();
        let btn_move5 = btn_move5.clone();
        let selected_index = selected_index.clone();
        btn_pause.connect_clicked(move |_| {
            state.borrow_mut().deselect();
            update_list_appearance(&list_box_clone, &state);
            refresh_button_sensitivity(
                &btn_plus,
                &btn_minus,
                &btn_pause_c,
                &btn_edit_time,
                &btn_delete,
                &btn_move5,
                &state,
                &selected_index,
            );
        });
    }

    // --- Signal: Edit Time button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let selected_index = selected_index.clone();
        let suppress_selection = suppress_selection.clone();
        let window_clone = window.clone();
        btn_edit_time.connect_clicked(move |_| {
            if let Some(idx) = selected_index.get() {
                show_edit_time_dialog(
                    &window_clone,
                    &state,
                    &list_box_clone,
                    &selected_index,
                    &suppress_selection,
                    &total_label,
                    idx,
                );
            }
        });
    }

    // --- Signal: Delete button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete_c = btn_delete.clone();
        let btn_move5 = btn_move5.clone();
        let selected_index = selected_index.clone();
        let suppress_selection = suppress_selection.clone();
        btn_delete.connect_clicked(move |_| {
            if let Some(idx) = selected_index.get() {
                state.borrow_mut().delete_project(idx);
                selected_index.set(None);
                populate_list(
                    &list_box_clone,
                    &state,
                    &selected_index,
                    &suppress_selection,
                );
                update_total_label(&total_label, &state);
                state.borrow().save_projects();
                refresh_button_sensitivity(
                    &btn_plus,
                    &btn_minus,
                    &btn_pause,
                    &btn_edit_time,
                    &btn_delete_c,
                    &btn_move5,
                    &state,
                    &selected_index,
                );
            }
        });
    }

    // --- Signal: Move 5 min button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        let btn_plus = btn_plus.clone();
        let btn_minus = btn_minus.clone();
        let btn_pause = btn_pause.clone();
        let btn_edit_time = btn_edit_time.clone();
        let btn_delete = btn_delete.clone();
        let btn_move5_c = btn_move5.clone();
        let selected_index = selected_index.clone();
        btn_move5.connect_clicked(move |_| {
            let active = state.borrow().active_index;
            let selected = selected_index.get();
            if let (Some(from), Some(to)) = (active, selected) {
                if from != to {
                    state.borrow_mut().transfer_minutes(from, to, 5);
                    state.borrow().save_times();
                    update_list_appearance(&list_box_clone, &state);
                    update_total_label(&total_label, &state);
                    refresh_button_sensitivity(
                        &btn_plus,
                        &btn_minus,
                        &btn_pause,
                        &btn_edit_time,
                        &btn_delete,
                        &btn_move5_c,
                        &state,
                        &selected_index,
                    );
                }
            }
        });
    }

    // --- Auto-save timer (every 10 minutes) ---
    {
        let state = state.clone();
        glib::timeout_add_seconds_local(600, move || {
            let mut s = state.borrow_mut();
            s.rollover_if_new_day();
            s.save_times();
            glib::ControlFlow::Continue
        });
    }

    // --- 1-minute tick timer ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let total_label = total_label.clone();
        glib::timeout_add_seconds_local(60, move || {
            {
                let mut s = state.borrow_mut();
                s.rollover_if_new_day();
                s.tick();
            }
            update_list_appearance(&list_box_clone, &state);
            update_total_label(&total_label, &state);
            glib::ControlFlow::Continue
        });
    }

    // --- Active row pulse timer ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        glib::timeout_add_local(Duration::from_millis(500), move || {
            if state.borrow().active_index.is_some() {
                update_list_appearance(&list_box_clone, &state);
            }
            glib::ControlFlow::Continue
        });
    }

    // --- Keyboard shortcuts: Ctrl+/- for font size, Ctrl+0 to reset ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let evk = gtk4::EventControllerKey::new();
        evk.connect_key_pressed(move |_, key, _, mods| {
            use gtk4::gdk::ModifierType;
            if mods.contains(ModifierType::CONTROL_MASK) {
                match key {
                    gtk4::gdk::Key::equal
                    | gtk4::gdk::Key::plus
                    | gtk4::gdk::Key::KP_Add => {
                        let mut s = state.borrow_mut();
                        s.font_size = (s.font_size + 1).min(32);
                        save_font_size(s.font_size);
                        drop(s);
                        update_list_appearance(&list_box_clone, &state);
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::minus | gtk4::gdk::Key::KP_Subtract => {
                        let mut s = state.borrow_mut();
                        s.font_size = (s.font_size - 1).max(6);
                        save_font_size(s.font_size);
                        drop(s);
                        update_list_appearance(&list_box_clone, &state);
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::_0 => {
                        let mut s = state.borrow_mut();
                        s.font_size = 12;
                        save_font_size(s.font_size);
                        drop(s);
                        update_list_appearance(&list_box_clone, &state);
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }
            glib::Propagation::Proceed
        });
        window.add_controller(evk);
    }

    // --- Window close: persist state and config ---
    {
        let state = state.clone();
        window.connect_close_request(move |win| {
            let mut s = state.borrow_mut();
            s.rollover_if_new_day();
            s.save_times();
            let mut cfg = Config::load();
            cfg.window_width = win.width();
            cfg.window_height = win.height();
            cfg.font_size = s.font_size;
            cfg.last_active = s.active_index.map(|i| LastActive {
                date: data::today_string(),
                project_name: s.projects[i].name.clone(),
            });
            cfg.save();
            // Explicitly quit the application so app.run() returns in main(),
            // ensuring the LockGuard is dropped and LOCK file is removed.
            if let Some(application) = win.application() {
                application.quit();
            }
            glib::Propagation::Proceed
        });
    }

    window.present();

    // Some GTK setups keep an implicit initial selection/focus state until
    // first interaction. Clear it once after the window is shown so the first
    // right-click transfer behaves like subsequent transfers.
    {
        let list_box = list_box.clone();
        let selected_index = selected_index.clone();
        glib::idle_add_local_once(move || {
            selected_index.set(None);
            list_box.unselect_all();
        });
    }
}

fn populate_list(
    list_box: &ListBox,
    state: &Rc<RefCell<AppState>>,
    selected_index: &Rc<Cell<Option<usize>>>,
    suppress_selection: &Rc<Cell<bool>>,
) {
    suppress_selection.set(true);
    // Remove all existing rows
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    let s = state.borrow();
    for (i, project) in s.projects.iter().enumerate() {
        let is_active = s.active_index == Some(i);
        let row = make_row(project, is_active, s.font_size);
        list_box.append(&row);
    }

    match selected_index.get() {
        Some(index) => {
            if let Some(row) = list_box.row_at_index(index as i32) {
                list_box.select_row(Some(&row));
            } else {
                selected_index.set(None);
                list_box.unselect_all();
            }
        }
        None => list_box.unselect_all(),
    }
    suppress_selection.set(false);
}

fn update_list_appearance(list_box: &ListBox, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    let pulse_on = active_row_pulse_phase();
    let mut row_opt = list_box.first_child();
    let mut i = 0usize;
    while let Some(widget) = row_opt {
        if let Some(row) = widget.downcast_ref::<ListBoxRow>() {
            if let Some(project) = s.projects.get(i) {
                let is_active = s.active_index == Some(i);
                if let Some(label) = row.child().and_then(|c| c.downcast::<Label>().ok()) {
                    label.set_markup(&row_markup(project, is_active, s.font_size));
                }
                apply_row_style(row, is_active, pulse_on);
            }
            row_opt = row.next_sibling();
        } else {
            break;
        }
        i += 1;
    }
}

fn install_drag_reorder(
    list_box: &ListBox,
    state: &Rc<RefCell<AppState>>,
    selected_index: &Rc<Cell<Option<usize>>>,
    suppress_selection: &Rc<Cell<bool>>,
    total_label: &Label,
    btn_plus: &Button,
    btn_minus: &Button,
    btn_pause: &Button,
    btn_edit_time: &Button,
    btn_delete: &Button,
    btn_move5: &Button,
) {
    let drag_source = gtk4::DragSource::builder()
        .actions(gdk::DragAction::MOVE)
        .build();

    let list_for_prepare = list_box.clone();
    drag_source.connect_prepare(move |_, _x, y| {
        let row = list_for_prepare.row_at_y(y as i32)?;
        let index = row.index() as u32;
        Some(gdk::ContentProvider::for_value(&index.to_value()))
    });
    list_box.add_controller(drag_source);

    let drop_target = gtk4::DropTarget::new(u32::static_type(), gdk::DragAction::MOVE);
    let list_for_drop = list_box.clone();
    let state_for_drop = state.clone();
    let selected_for_drop = selected_index.clone();
    let suppress_for_drop = suppress_selection.clone();
    let total_for_drop = total_label.clone();
    let btn_plus = btn_plus.clone();
    let btn_minus = btn_minus.clone();
    let btn_pause = btn_pause.clone();
    let btn_edit_time = btn_edit_time.clone();
    let btn_delete = btn_delete.clone();
    let btn_move5 = btn_move5.clone();
    drop_target.connect_drop(move |_, value, _x, y| {
        let from = match value.get::<u32>() {
            Ok(v) => v as usize,
            Err(_) => return false,
        };

        let to = {
            let s = state_for_drop.borrow();
            if s.projects.is_empty() {
                return false;
            }
            list_for_drop
                .row_at_y(y as i32)
                .map(|row| row.index() as usize)
                .unwrap_or_else(|| s.projects.len().saturating_sub(1))
        };

        if from == to {
            return false;
        }

        {
            let mut s = state_for_drop.borrow_mut();
            s.move_project(from, to);
            s.save_projects();
        }

        selected_for_drop.set(Some(to));
        populate_list(
            &list_for_drop,
            &state_for_drop,
            &selected_for_drop,
            &suppress_for_drop,
        );
        update_total_label(&total_for_drop, &state_for_drop);
        refresh_button_sensitivity(
            &btn_plus,
            &btn_minus,
            &btn_pause,
            &btn_edit_time,
            &btn_delete,
            &btn_move5,
            &state_for_drop,
            &selected_for_drop,
        );
        true
    });
    list_box.add_controller(drop_target);
}

fn row_markup(project: &crate::app::Project, active: bool, font_size: i32) -> String {
    // Show blank for 0-minute projects, time for others.
    // <tt> (monospace) ensures 5 spaces == "HH:MM" width, keeping the name
    // column aligned regardless of whether time is shown.
    let time_display = if project.minutes > 0 {
        data::format_hhmm(project.minutes)
    } else {
        "     ".to_string()
    };
    let name = glib::markup_escape_text(&project.name);
    if active {
        format!(
            "<span font_size=\"{0}pt\" weight=\"bold\" foreground=\"#2080ff\"><tt>{1}</tt> {2}</span>",
            font_size, time_display, name
        )
    } else {
        format!(
            "<span font_size=\"{0}pt\"><tt>{1}</tt> {2}</span>",
            font_size, time_display, name
        )
    }
}

fn make_row(
    project: &crate::app::Project,
    active: bool,
    font_size: i32,
) -> ListBoxRow {
    let label = Label::new(None);
    label.set_markup(&row_markup(project, active, font_size));
    label.set_xalign(0.0);
    label.set_margin_top(0);
    label.set_margin_bottom(0);
    label.set_margin_start(4);
    label.set_margin_end(4);
    let row = ListBoxRow::new();
    row.set_child(Some(&label));
    apply_row_style(&row, active, active_row_pulse_phase());
    row
}

fn active_row_pulse_phase() -> bool {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    ((millis / 700) % 2) == 0
}

fn apply_row_style(row: &ListBoxRow, active: bool, pulse_on: bool) {
    row.remove_css_class("active-project");
    row.remove_css_class("active-project-pulse");
    if active {
        row.add_css_class("active-project");
        if pulse_on {
            row.add_css_class("active-project-pulse");
        }
    }
}

fn install_compact_list_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        ".compact-project-list row { min-height: 0; padding-top: 0; padding-bottom: 0; }\
         .compact-project-list row label { padding-top: 0; padding-bottom: 0; }\
         .compact-project-list row:selected { background-color: #404040; color: #f0f0f0; }\
         .compact-project-list row:selected label { color: #f0f0f0; }\
            .compact-project-list row.active-project { box-shadow: inset 4px 0 0 #5a6a7d; }\
            .compact-project-list row.active-project-pulse { box-shadow: inset 4px 0 0 #8aa0ba; }",
    );
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn install_local_icon_paths() {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let icon_theme = gtk4::IconTheme::for_display(&display);
    for path in [
        "share/icons/hicolor/64x64/apps",
        "share/icons/hicolor/scalable/apps",
        "share/icons",
    ] {
        let candidate = std::path::Path::new(path);
        if candidate.exists() {
            icon_theme.add_search_path(candidate);
        }
    }
}

fn refresh_button_sensitivity(
    btn_plus: &Button,
    btn_minus: &Button,
    btn_pause: &Button,
    btn_edit_time: &Button,
    btn_delete: &Button,
    btn_move5: &Button,
    state: &Rc<RefCell<AppState>>,
    selected_index: &Rc<Cell<Option<usize>>>,
) {
    let s = state.borrow();
    let sel = selected_index.get();
    let has_selection = sel.is_some();
    let move5_enabled = s.active_index.is_some() && sel.is_some() && sel != s.active_index;

    btn_plus.set_sensitive(has_selection);
    btn_minus.set_sensitive(has_selection);
    btn_pause.set_sensitive(s.active_index.is_some());
    btn_edit_time.set_sensitive(has_selection);
    btn_delete.set_sensitive(has_selection);
    btn_move5.set_sensitive(move5_enabled);
}

fn total_display_text(state: &AppState) -> String {
    let sign = if state.adjusted_minutes >= 0 { '+' } else { '-' };
    let abs_delta = state.adjusted_minutes.abs();
    format!(
        "{}{}{:02}",
        data::format_hhmm(state.total_minutes),
        sign,
        abs_delta
    )
}

fn update_total_label(label: &Label, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    label.set_label(&total_display_text(&s));
}

fn save_font_size(font_size: i32) {
    let mut cfg = Config::load();
    cfg.font_size = font_size;
    cfg.save();
}

fn show_add_dialog(
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    list_box: &ListBox,
    selected_index: &Rc<Cell<Option<usize>>>,
    suppress_selection: &Rc<Cell<bool>>,
    total_label: &Label,
) {
    let dialog = gtk4::Dialog::with_buttons(
        Some("Add Project"),
        Some(window),
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        &[
            ("Cancel", gtk4::ResponseType::Cancel),
            ("Add", gtk4::ResponseType::Accept),
        ],
    );
    let content = dialog.content_area();
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Project name"));
    entry.set_margin_top(8);
    entry.set_margin_bottom(8);
    entry.set_margin_start(8);
    entry.set_margin_end(8);
    content.append(&entry);

    let state_c = state.clone();
    let lb = list_box.clone();
    let selected = selected_index.clone();
    let suppress = suppress_selection.clone();
    let total = total_label.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk4::ResponseType::Accept {
            let name = entry.text().to_string().trim().to_string();
            if !name.is_empty() {
                state_c.borrow_mut().add_project(name);
                populate_list(&lb, &state_c, &selected, &suppress);
                update_total_label(&total, &state_c);
                state_c.borrow().save_projects();
            }
        }
        dlg.close();
    });
    dialog.present();
}

fn show_edit_time_dialog(
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    list_box: &ListBox,
    selected_index: &Rc<Cell<Option<usize>>>,
    suppress_selection: &Rc<Cell<bool>>,
    total_label: &Label,
    index: usize,
) {
    let current_minutes = state
        .borrow()
        .projects
        .get(index)
        .map(|p| p.minutes)
        .unwrap_or(0);
    let current_str = data::format_hhmm(current_minutes);

    let dialog = gtk4::Dialog::with_buttons(
        Some("Edit Time"),
        Some(window),
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        &[
            ("Cancel", gtk4::ResponseType::Cancel),
            ("Set", gtk4::ResponseType::Accept),
        ],
    );
    let content = dialog.content_area();
    let entry = Entry::new();
    entry.set_text(&current_str);
    entry.set_placeholder_text(Some("hh:mm"));
    entry.set_margin_top(8);
    entry.set_margin_bottom(8);
    entry.set_margin_start(8);
    entry.set_margin_end(8);
    content.append(&entry);

    let state_c = state.clone();
    let lb = list_box.clone();
    let selected = selected_index.clone();
    let suppress = suppress_selection.clone();
    let total = total_label.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk4::ResponseType::Accept {
            let text = entry.text().to_string();
            let minutes = data::parse_hhmm(&text);
            state_c.borrow_mut().set_time(index, minutes);
            populate_list(&lb, &state_c, &selected, &suppress);
            update_total_label(&total, &state_c);
            state_c.borrow().save_times();
        }
        dlg.close();
    });
    dialog.present();
}

