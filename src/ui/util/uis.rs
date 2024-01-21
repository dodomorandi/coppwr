// Copyright 2023-2024 Dimitris Papaioannou <dimtpap@protonmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeMap;

use eframe::egui;

/// Displays a grid with 2 columns.
/// Useful for displaying key-value pairs.
pub fn key_val_table(
    ui: &mut egui::Ui,
    min_scrolled_height: f32,
    max_height: f32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::ScrollArea::vertical()
        .min_scrolled_height(min_scrolled_height)
        .max_height(max_height)
        .show(ui, |ui| {
            egui::Grid::new("kvtable")
                .num_columns(2)
                .max_col_width(ui.available_width())
                .striped(true)
                .show(ui, add_contents);
        });
}

/// Displays all the key-value pairs of the iterator using [`key_val_table`].
pub fn key_val_display(
    ui: &mut egui::Ui,
    min_scrolled_height: f32,
    max_height: f32,
    header: &str,
    kv: impl Iterator<Item = (impl Into<egui::WidgetText>, impl Into<egui::WidgetText>)>,
) {
    ui.collapsing(header, |ui| {
        key_val_table(ui, min_scrolled_height, max_height, |ui| {
            for (k, v) in kv {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            }
        });
    });
}

/// Displays the key-value pairs of a map with the ability to delete them and add new ones.
pub fn map_editor(
    ui: &mut egui::Ui,
    min_scrolled_height: f32,
    max_height: f32,
    map: &mut BTreeMap<Box<str>, String>,
    user_additions: &mut EditableKVList,
) {
    key_val_table(ui, min_scrolled_height, max_height, |ui| {
        map.retain(|k, v| {
            ui.label(k.as_ref());
            let keep = ui
                .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    let keep = !ui.button("Delete").clicked();
                    egui::TextEdit::singleline(v)
                        .hint_text("Value")
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    keep
                })
                .inner;
            ui.end_row();
            keep
        });
    });

    ui.separator();

    ui.label("Add items");
    user_additions.show(ui);
}

#[derive(Default)]
pub struct EditableKVList {
    list: Vec<(String, String)>,
}

impl EditableKVList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.list.retain_mut(|(k, v)| {
            ui.horizontal(|ui| {
                let keep = !ui.button("Delete").clicked();
                ui.add(
                    egui::TextEdit::singleline(k)
                        .hint_text("Key")
                        .desired_width(ui.available_width() / 2.5),
                );
                ui.add(
                    egui::TextEdit::singleline(v)
                        .hint_text("Value")
                        .desired_width(ui.available_width()),
                );
                keep
            })
            .inner
        });

        if ui.button("Add").clicked() {
            self.list.push((String::new(), String::new()));
        }
    }

    pub const fn list(&self) -> &Vec<(String, String)> {
        &self.list
    }

    pub fn list_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.list
    }

    pub fn take(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.list)
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }
}

/// Like [`map_editor`] but it stores the map in itself.
#[derive(Default)]
pub struct MapEditor {
    properties: BTreeMap<Box<str>, String>,
    user_additions: EditableKVList,
}

impl MapEditor {
    pub fn set_map(&mut self, map: BTreeMap<Box<str>, String>) {
        self.properties = map;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, min_scrolled_height: f32, max_height: f32) {
        map_editor(
            ui,
            min_scrolled_height,
            max_height,
            &mut self.properties,
            &mut self.user_additions,
        );
    }

    pub fn take(&mut self) -> BTreeMap<Box<str>, String> {
        self.properties.extend(
            self.user_additions
                .take()
                .into_iter()
                .map(|(k, v)| (k.into_boxed_str(), v)),
        );

        std::mem::take(&mut self.properties)
    }
}

mod kv_matcher {
    use eframe::egui;

    #[derive(PartialEq, Eq)]
    enum StringMatchMode {
        Substring,
        StartsWith,
        EndsWith,
        Exact,
    }

    impl StringMatchMode {
        fn matches(&self, haystack: &str, needle: &str) -> bool {
            match self {
                Self::Substring => haystack.contains(needle),
                Self::StartsWith => haystack.starts_with(needle),
                Self::EndsWith => haystack.ends_with(needle),
                Self::Exact => haystack == needle,
            }
        }

        fn show_selector(&mut self, ui: &mut egui::Ui, id_source: impl std::hash::Hash) {
            const fn as_user_str(mode: &StringMatchMode) -> &'static str {
                match mode {
                    StringMatchMode::Substring => "contains",
                    StringMatchMode::StartsWith => "starts with",
                    StringMatchMode::EndsWith => "ends with",
                    StringMatchMode::Exact => "is",
                }
            }

            egui::ComboBox::from_id_source(id_source)
                .selected_text(as_user_str(self))
                .show_ui(ui, |ui| {
                    for mode in [
                        Self::Substring,
                        Self::StartsWith,
                        Self::EndsWith,
                        Self::Exact,
                    ] {
                        let text = as_user_str(&mode);
                        ui.selectable_value(self, mode, text);
                    }
                });
        }
    }

    struct StringFilter {
        needle: String,
        match_mode: StringMatchMode,
    }

    impl StringFilter {
        fn test(&self, value: &str) -> bool {
            self.match_mode.matches(value, &self.needle)
        }

        fn show(&mut self, ui: &mut egui::Ui, label: &str, text_edit_width: f32) {
            ui.label(label);
            self.match_mode.show_selector(ui, label);
            egui::TextEdit::singleline(&mut self.needle)
                .hint_text(label)
                .desired_width(text_edit_width)
                .show(ui);
        }
    }

    impl Default for StringFilter {
        fn default() -> Self {
            Self {
                needle: String::new(),
                match_mode: StringMatchMode::Substring,
            }
        }
    }

    /// User-configurable filter for key-value pair collections.
    pub struct KvMatcher {
        filters: Vec<(StringFilter, StringFilter)>,
    }

    impl KvMatcher {
        pub const fn new() -> Self {
            Self {
                filters: Vec::new(),
            }
        }

        pub fn matches(
            &self,
            kv: impl Iterator<Item = (impl AsRef<str>, impl AsRef<str>)> + Clone,
        ) -> bool {
            self.filters.iter().all(|(key_filter, value_filter)| {
                kv.clone()
                    .any(|(k, v)| key_filter.test(k.as_ref()) && value_filter.test(v.as_ref()))
            })
        }

        pub fn show(&mut self, ui: &mut egui::Ui) {
            let mut i = 0usize;
            self.filters.retain_mut(|(key_filter, value_filter)| {
                let keep = ui
                    .push_id(i, |ui| {
                        ui.horizontal(|ui| {
                            let keep = !ui.button("Delete").clicked();

                            key_filter.show(ui, "Key", ui.available_width() / 4.);
                            value_filter.show(ui, "Value", f32::INFINITY);

                            keep
                        })
                        .inner
                    })
                    .inner;

                i += 1;

                keep
            });

            if ui.button("Add").clicked() {
                self.filters
                    .push((StringFilter::default(), StringFilter::default()));
            }
        }
    }
}

pub use kv_matcher::KvMatcher;
