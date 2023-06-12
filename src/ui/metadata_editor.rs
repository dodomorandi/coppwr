// Copyright 2023 Dimitris Papaioannou <dimtpap@protonmail.com>
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

use std::collections::{btree_map::Entry, BTreeMap};

use eframe::egui;

use crate::backend::{ObjectMethod, Request};
use crate::ui::Tool;

struct Property {
    subject: u32,
    type_: Option<String>,
    value: String,
}

impl Property {
    fn set_request(&self, key: String) -> ObjectMethod {
        ObjectMethod::MetadataSetProperty {
            subject: self.subject,
            key,
            type_: self.type_.as_ref().cloned(),
            value: Some(self.value.clone()),
        }
    }

    fn clear_request(&self, key: String) -> ObjectMethod {
        ObjectMethod::MetadataSetProperty {
            subject: self.subject,
            key,
            type_: self.type_.as_ref().cloned(),
            value: None,
        }
    }
}

struct Metadata {
    name: String,
    properties: BTreeMap<String, Property>,
    user_properties: Vec<(String, Property)>,
}

pub struct MetadataEditor {
    metadatas: BTreeMap<u32, Metadata>,
}

impl Tool for MetadataEditor {
    fn draw(&mut self, ui: &mut egui::Ui, rsx: &pipewire::channel::Sender<Request>) {
        self.draw(ui, rsx);
    }
}

impl MetadataEditor {
    pub fn new() -> Self {
        Self {
            metadatas: BTreeMap::new(),
        }
    }

    pub fn add_metadata(&mut self, id: u32, name: &str) {
        self.metadatas.entry(id).or_insert(Metadata {
            name: name.to_string(),
            properties: BTreeMap::new(),
            user_properties: Vec::new(),
        });
    }

    pub fn add_property(
        &mut self,
        id: u32,
        name: String,
        subject: u32,
        key: String,
        type_: Option<String>,
        value: String,
    ) {
        let prop = Property {
            subject,
            type_,
            value,
        };
        match self.metadatas.entry(id) {
            Entry::Occupied(e) => {
                let properties = &mut e.into_mut().properties;
                match properties.entry(key) {
                    Entry::Occupied(e) => {
                        *e.into_mut() = prop;
                    }
                    Entry::Vacant(e) => {
                        e.insert(prop);
                    }
                }
            }
            Entry::Vacant(e) => {
                let metadata = Metadata {
                    name,
                    properties: BTreeMap::new(),
                    user_properties: Vec::new(),
                };
                e.insert(metadata).properties.insert(key, prop);
            }
        }
    }

    pub fn remove_metadata(&mut self, id: u32) {
        self.metadatas.remove(&id);
    }

    pub fn remove_property(&mut self, id: u32, key: &str) {
        self.metadatas.entry(id).and_modify(|m| {
            m.properties.remove(key);
        });
    }

    pub fn clear_properties(&mut self, id: u32) {
        self.metadatas.entry(id).and_modify(|m| {
            m.properties.clear();
        });
    }

    fn draw(&mut self, ui: &mut egui::Ui, rsx: &pipewire::channel::Sender<Request>) {
        for (id, metadata) in &mut self.metadatas {
            ui.heading(&metadata.name);
            ui.horizontal(|ui| {
                ui.label(format!("ID: {id}"));
                if ui.small_button("Clear").clicked() {
                    rsx.send(Request::CallObjectMethod(*id, ObjectMethod::MetadataClear))
                        .ok();
                }
            });
            egui::Grid::new(&metadata.name)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (key, prop) in &mut metadata.properties {
                        ui.label(key);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.small_button("Clear").clicked() {
                                rsx.send(Request::CallObjectMethod(
                                    *id,
                                    prop.clear_request(key.clone()),
                                ))
                                .ok();
                            }
                            if ui.small_button("Set").clicked() {
                                rsx.send(Request::CallObjectMethod(
                                    *id,
                                    prop.set_request(key.clone()),
                                ))
                                .ok();
                            }
                            let input = ui.add(
                                egui::TextEdit::singleline(&mut prop.value)
                                    .hint_text("Value")
                                    .desired_width(f32::INFINITY),
                            );
                            if let Some(type_) = prop.type_.as_ref() {
                                input.on_hover_text(format!(
                                    "Type: {type_}\nSubject: {}",
                                    prop.subject
                                ));
                            } else {
                                input.on_hover_text(prop.subject.to_string());
                            }
                        });

                        ui.end_row();
                    }
                });

            if ui.button("Add Property").clicked() {
                metadata.user_properties.push((
                    String::new(),
                    Property {
                        subject: 0,
                        type_: None,
                        value: String::new(),
                    },
                ));
            }
            metadata.user_properties.retain_mut(|(key, prop)| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(key)
                            .hint_text("Key")
                            .desired_width(ui.available_width() / 2.),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut prop.value)
                            .hint_text("Value")
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Subject");
                    ui.add(egui::widgets::DragValue::new(&mut prop.subject));

                    if ui.checkbox(&mut prop.type_.is_some(), "Type").changed() {
                        if prop.type_.is_none() {
                            prop.type_ = Some(String::new());
                        } else {
                            prop.type_ = None;
                        }
                    }
                    if let Some(ref mut type_) = prop.type_ {
                        ui.add(
                            egui::TextEdit::singleline(type_)
                                .hint_text("Type")
                                .desired_width(f32::INFINITY),
                        );
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        rsx.send(Request::CallObjectMethod(
                            *id,
                            prop.set_request(key.clone()),
                        ))
                        .ok();
                    }
                    !ui.button("Delete").clicked()
                })
                .inner
            });
            ui.separator();
        }
    }
}
