use gtk4::prelude::*;
use gtk4::{
    glib, Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label,
    ListBox, ListBoxRow, Orientation, PolicyType, ScrolledWindow, SelectionMode, SpinButton,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::AppState;
use crate::config::{Config, LastActive};
use crate::data;
use crate::sort;

pub fn build_window(app: &Application) {
    let config = Config::load();
    let mut state = AppState::new(config.font_size);
    let _ = data::ensure_data_dir();
    state.load_today();

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
        .title("TimeTracker")
        .default_width(config.window_width)
        .default_height(config.window_height)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 4);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    // Header label
    let header = Label::new(Some("TimeTracker"));
    header.set_markup("<b>TimeTracker</b>");
    vbox.append(&header);

    // Scrolled project list
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Single);
    scrolled.set_child(Some(&list_box));
    vbox.append(&scrolled);

    // Button bar
    let btn_box = GtkBox::new(Orientation::Horizontal, 4);
    let btn_add = Button::with_label("Add");
    let btn_sort = Button::with_label("Sort A-Å");
    let btn_pause = Button::with_label("Pause");
    btn_box.append(&btn_add);
    btn_box.append(&btn_sort);
    btn_box.append(&btn_pause);
    vbox.append(&btn_box);

    window.set_child(Some(&vbox));

    // Populate list with today's projects
    populate_list(&list_box, &state);

    // --- Signal: row activated (left-click) ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let mut s = state.borrow_mut();
            if s.active_index == Some(index) {
                s.deselect();
            } else {
                s.select_project(index);
            }
            drop(s);
            update_list_appearance(&list_box_clone, &state);
        });
    }

    // --- Signal: Add button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        let window_clone = window.clone();
        btn_add.connect_clicked(move |_| {
            show_add_dialog(&window_clone, &state, &list_box_clone);
        });
    }

    // --- Signal: Sort button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        btn_sort.connect_clicked(move |_| {
            {
                let mut s = state.borrow_mut();
                sort::sort_projects(&mut s.projects);
                s.active_index = None;
                s.paused = true;
            }
            populate_list(&list_box_clone, &state);
            state.borrow().save();
        });
    }

    // --- Signal: Pause button ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        btn_pause.connect_clicked(move |_| {
            state.borrow_mut().deselect();
            update_list_appearance(&list_box_clone, &state);
        });
    }

    // --- Right-click context menu ---
    setup_context_menu(&list_box, &state, &window);

    // --- Auto-save timer (every 10 minutes) ---
    {
        let state = state.clone();
        glib::timeout_add_seconds_local(600, move || {
            state.borrow().save();
            glib::ControlFlow::Continue
        });
    }

    // --- 1-minute tick timer ---
    {
        let state = state.clone();
        let list_box_clone = list_box.clone();
        glib::timeout_add_seconds_local(60, move || {
            {
                let mut s = state.borrow_mut();
                s.tick();
            }
            update_list_appearance(&list_box_clone, &state);
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
                    gtk4::gdk::Key::equal | gtk4::gdk::Key::plus => {
                        let mut s = state.borrow_mut();
                        s.font_size = (s.font_size + 1).min(32);
                        drop(s);
                        update_list_appearance(&list_box_clone, &state);
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::minus => {
                        let mut s = state.borrow_mut();
                        s.font_size = (s.font_size - 1).max(6);
                        drop(s);
                        update_list_appearance(&list_box_clone, &state);
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::_0 => {
                        let mut s = state.borrow_mut();
                        s.font_size = 12;
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
            let s = state.borrow();
            s.save();
            let mut cfg = Config::load();
            cfg.window_width = win.width();
            cfg.window_height = win.height();
            cfg.font_size = s.font_size;
            cfg.last_active = s.active_index.map(|i| LastActive {
                date: data::today_string(),
                project_name: s.projects[i].name.clone(),
            });
            cfg.save();
            glib::Propagation::Proceed
        });
    }

    window.present();
}

fn populate_list(list_box: &ListBox, state: &Rc<RefCell<AppState>>) {
    // Remove all existing rows
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    let s = state.borrow();
    for (i, project) in s.projects.iter().enumerate() {
        let is_active = s.active_index == Some(i);
        let row = make_row(project, is_active, project.marked, s.font_size);
        list_box.append(&row);
    }
}

fn update_list_appearance(list_box: &ListBox, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    let mut row_opt = list_box.first_child();
    let mut i = 0usize;
    while let Some(widget) = row_opt {
        if let Some(row) = widget.downcast_ref::<ListBoxRow>() {
            if let Some(project) = s.projects.get(i) {
                let is_active = s.active_index == Some(i);
                if let Some(label) = row.child().and_then(|c| c.downcast::<Label>().ok()) {
                    label.set_markup(&row_markup(project, is_active, project.marked, s.font_size));
                }
            }
            row_opt = row.next_sibling();
        } else {
            break;
        }
        i += 1;
    }
}

fn row_markup(
    project: &crate::app::Project,
    active: bool,
    marked: bool,
    font_size: i32,
) -> String {
    let time_str = data::format_hhmm(project.minutes);
    let name = glib::markup_escape_text(&project.name);
    let mark_indicator = if marked { " ●" } else { "" };
    if active {
        format!(
            "<span font_size=\"{}pt\" weight=\"bold\" foreground=\"#2080ff\">{} {}{}</span>",
            font_size, time_str, name, mark_indicator
        )
    } else {
        format!(
            "<span font_size=\"{}pt\">{} {}{}</span>",
            font_size, time_str, name, mark_indicator
        )
    }
}

fn make_row(
    project: &crate::app::Project,
    active: bool,
    marked: bool,
    font_size: i32,
) -> ListBoxRow {
    let label = Label::new(None);
    label.set_markup(&row_markup(project, active, marked, font_size));
    label.set_xalign(0.0);
    label.set_margin_top(2);
    label.set_margin_bottom(2);
    label.set_margin_start(4);
    label.set_margin_end(4);
    let row = ListBoxRow::new();
    row.set_child(Some(&label));
    row
}

fn setup_context_menu(
    list_box: &ListBox,
    state: &Rc<RefCell<AppState>>,
    window: &ApplicationWindow,
) {
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3); // Right mouse button

    let state = state.clone();
    let list_box_clone = list_box.clone();
    let window_clone = window.clone();

    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        if let Some(row) = list_box_clone.row_at_y(y as i32) {
            let index = row.index() as usize;
            show_context_menu(&list_box_clone, &state, &window_clone, index, x, y);
        }
    });

    list_box.add_controller(gesture);
}

fn show_context_menu(
    list_box: &ListBox,
    state: &Rc<RefCell<AppState>>,
    window: &ApplicationWindow,
    index: usize,
    x: f64,
    y: f64,
) {
    let menu_box = GtkBox::new(Orientation::Vertical, 2);
    menu_box.set_margin_top(4);
    menu_box.set_margin_bottom(4);
    menu_box.set_margin_start(4);
    menu_box.set_margin_end(4);

    let popover = gtk4::Popover::new();
    popover.set_child(Some(&menu_box));
    popover.set_parent(list_box);
    let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
    popover.set_pointing_to(Some(&rect));

    // "Mark as source" button
    {
        let btn = Button::with_label("Mark as source");
        btn.add_css_class("flat");
        let state_c = state.clone();
        let lb = list_box.clone();
        let popover_c = popover.clone();
        btn.connect_clicked(move |_| {
            popover_c.popdown();
            state_c.borrow_mut().mark_source(index);
            populate_list(&lb, &state_c);
            state_c.borrow().save();
        });
        menu_box.append(&btn);
    }

    // Transfer buttons (only if a different project is marked)
    let marked_idx = state.borrow().marked_index;
    if let Some(from) = marked_idx {
        if from != index {
            let from_name = state
                .borrow()
                .projects
                .get(from)
                .map(|p| p.name.clone())
                .unwrap_or_default();

            // Transfer 5 minutes
            {
                let label = format!("Transfer 5 min from {}", from_name);
                let btn = Button::with_label(&label);
                btn.add_css_class("flat");
                let state_c = state.clone();
                let lb = list_box.clone();
                let popover_c = popover.clone();
                btn.connect_clicked(move |_| {
                    popover_c.popdown();
                    state_c.borrow_mut().transfer_minutes(from, index, 5);
                    populate_list(&lb, &state_c);
                    state_c.borrow().save();
                });
                menu_box.append(&btn);
            }

            // Transfer custom amount
            {
                let label = format!("Transfer custom from {}…", from_name);
                let btn = Button::with_label(&label);
                btn.add_css_class("flat");
                let state_c = state.clone();
                let lb = list_box.clone();
                let window_c = window.clone();
                let popover_c = popover.clone();
                btn.connect_clicked(move |_| {
                    popover_c.popdown();
                    show_transfer_dialog(&window_c, &state_c, &lb, from, index);
                });
                menu_box.append(&btn);
            }
        }
    }

    // Edit time directly
    {
        let btn = Button::with_label("Edit time directly…");
        btn.add_css_class("flat");
        let state_c = state.clone();
        let lb = list_box.clone();
        let window_c = window.clone();
        let popover_c = popover.clone();
        btn.connect_clicked(move |_| {
            popover_c.popdown();
            show_edit_time_dialog(&window_c, &state_c, &lb, index);
        });
        menu_box.append(&btn);
    }

    // Delete project
    {
        let btn = Button::with_label("Delete project");
        btn.add_css_class("flat");
        let state_c = state.clone();
        let lb = list_box.clone();
        let popover_c = popover.clone();
        btn.connect_clicked(move |_| {
            popover_c.popdown();
            state_c.borrow_mut().delete_project(index);
            populate_list(&lb, &state_c);
            state_c.borrow().save();
        });
        menu_box.append(&btn);
    }

    popover.popup();
}

fn show_add_dialog(
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    list_box: &ListBox,
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
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk4::ResponseType::Accept {
            let name = entry.text().to_string().trim().to_string();
            if !name.is_empty() {
                state_c.borrow_mut().add_project(name);
                populate_list(&lb, &state_c);
                state_c.borrow().save();
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
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk4::ResponseType::Accept {
            let text = entry.text().to_string();
            let minutes = data::parse_hhmm(&text);
            state_c.borrow_mut().set_time(index, minutes);
            populate_list(&lb, &state_c);
            state_c.borrow().save();
        }
        dlg.close();
    });
    dialog.present();
}

fn show_transfer_dialog(
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    list_box: &ListBox,
    from: usize,
    to: usize,
) {
    let dialog = gtk4::Dialog::with_buttons(
        Some("Transfer Minutes"),
        Some(window),
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        &[
            ("Cancel", gtk4::ResponseType::Cancel),
            ("Transfer", gtk4::ResponseType::Accept),
        ],
    );
    let content = dialog.content_area();
    let adj = Adjustment::new(5.0, 1.0, 480.0, 1.0, 10.0, 0.0);
    let spin = SpinButton::new(Some(&adj), 1.0, 0);
    spin.set_margin_top(8);
    spin.set_margin_bottom(8);
    spin.set_margin_start(8);
    spin.set_margin_end(8);
    content.append(&spin);

    let state_c = state.clone();
    let lb = list_box.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk4::ResponseType::Accept {
            let minutes = spin.value() as u32;
            state_c.borrow_mut().transfer_minutes(from, to, minutes);
            populate_list(&lb, &state_c);
            state_c.borrow().save();
        }
        dlg.close();
    });
    dialog.present();
}
