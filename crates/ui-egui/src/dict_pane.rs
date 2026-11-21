//! Ornagai English-Myanmar Two-Way Dictionary Task Pane for HtetWord.
//!
//! Provides instant bidirectional word lookup (English ⇄ Myanmar),
//! audio pronunciation (TTS), reverse Myanmar-to-English indexing,
//! live autocomplete suggestions, selection auto-sync, rich definition card
//! with Pyidaungsu typography, and one-click "Insert into Document" feature.

use egui::{Color32, CornerRadius, RichText, Stroke, Ui};
use serde_json::json;
use wordcraft_engine::dict::{self, DictItem};

use crate::WordApp;

/// State for the Dictionary task pane.
#[derive(Clone, Debug, Default)]
pub struct DictPaneState {
    pub query: String,
    pub last_searched: String,
    pub current_item: Option<DictItem>,
    pub reverse_results: Vec<(String, DictItem)>,
    pub not_found: bool,
    pub suggestions: Vec<String>,
    pub is_reverse_mode: bool,
    pub auto_sync_selection: bool,
}

/// Render the Dictionary task pane body.
pub fn show(app: &mut WordApp, ui: &mut Ui) {
    ui.add_space(6.0);

    // Header banner with Liquid Glass rose styling and Mode indicator
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(232, 74, 127, 20))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(232, 74, 127, 50)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📖").size(22.0));
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("အင်္ဂလိပ် ⇄ မြန်မာ အဘိဓာန်").strong().size(14.0).color(Color32::from_rgb(0x9E, 0x1B, 0x46)));
                        let mode_badge = if app.dict_state.is_reverse_mode {
                            "🇲🇲 မြန်မာ → အင်္ဂလိပ်"
                        } else {
                            "🇬🇧 အင်္ဂလိပ် → မြန်မာ"
                        };
                        egui::Frame::new()
                            .fill(Color32::from_rgb(0xFD, 0xDF, 0xEC))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(5, 1))
                            .show(ui, |ui| {
                                ui.label(RichText::new(mode_badge).size(9.5).color(Color32::from_rgb(0x9E, 0x1B, 0x46)).strong());
                            });
                    });
                    ui.label(RichText::new("Ornagai Two-Way Dictionary (၁၁၁,၂၃၂ စကားလုံး + ၁၉၇,၆၈၃ ဝေါဟာရ)").size(10.5).color(Color32::from_rgb(0x7D, 0x56, 0x68)));
                });
            });
        });

    ui.add_space(8.0);

    // Auto-sync selection from active document if enabled
    let mut search_triggered = false;
    if app.dict_state.auto_sync_selection && !app.session.sel.is_collapsed() {
        let (a, b) = app.session.sel.ordered();
        let frag = app.session.doc.copy_range(&a, &b);
        let selected_text = frag.clipboard_text().trim().to_string();
        if !selected_text.is_empty() && selected_text != app.dict_state.last_searched {
            app.dict_state.query = selected_text;
            search_triggered = true;
        }
    }

    // Search bar with live autocomplete and instant lookup
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        let edit = ui.add(
            egui::TextEdit::singleline(&mut app.dict_state.query)
                .hint_text("အင်္ဂလိပ် သို့မဟုတ် မြန်မာစာ ရိုက်ထည့်ပါ (e.g. freedom, ချစ်ခြင်း)...")
                .desired_width(ui.available_width() - 85.0),
        );
        if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            search_triggered = true;
        }

        if ui.button(RichText::new("ရှာမည်").strong()).clicked() {
            search_triggered = true;
        }
    });

    // Toolbar row: Lookup Selection button, Auto-sync checkbox, Clear button
    ui.horizontal(|ui| {
        if ui.button(RichText::new("📄 ရွေးထားသည်ကို ရှာမည်").size(11.0)).on_hover_text("စာတမ်းထဲမှ ရွေးထားသော စကားလုံးကို တိုက်ရိုက် ရှာဖွေမည်").clicked() {
            if !app.session.sel.is_collapsed() {
                let (a, b) = app.session.sel.ordered();
                let frag = app.session.doc.copy_range(&a, &b);
                let text = frag.clipboard_text().trim().to_string();
                if !text.is_empty() {
                    let first_word = text.split_whitespace().next().unwrap_or("").trim_matches(|c: char| !c.is_alphabetic() && !wordcraft_engine::myanmar::is_myanmar_char(c)).to_string();
                    if !first_word.is_empty() {
                        app.dict_state.query = first_word;
                        search_triggered = true;
                    }
                }
            }
        }

        ui.checkbox(&mut app.dict_state.auto_sync_selection, RichText::new("🔄 အလိုအလျောက်").size(11.0)).on_hover_text("စာတမ်းထဲတွင် စကားလုံး Select ချလိုက်သည်နှင့် အလိုအလျောက် အဓိပ္ပာယ်ရှာမည်");

        if !app.dict_state.query.is_empty() && ui.button("✖").on_hover_text("ရှင်းထုတ်မည်").clicked() {
            app.dict_state.query.clear();
            app.dict_state.current_item = None;
            app.dict_state.reverse_results.clear();
            app.dict_state.suggestions.clear();
            app.dict_state.not_found = false;
        }
    });

    // Handle search action
    if search_triggered && !app.dict_state.query.trim().is_empty() {
        let q = app.dict_state.query.trim().to_string();
        app.dict_state.last_searched = q.clone();

        if dict::is_myanmar_text(&q) {
            // Myanmar -> English reverse lookup
            app.dict_state.is_reverse_mode = true;
            let results = dict::reverse_search(&q, 15);
            app.dict_state.reverse_results = results.clone();
            app.dict_state.suggestions.clear();
            if let Some((_, first_item)) = results.first() {
                app.dict_state.current_item = Some(first_item.clone());
                app.dict_state.not_found = false;
            } else {
                app.dict_state.current_item = None;
                app.dict_state.not_found = true;
            }
        } else {
            // English -> Myanmar forward lookup
            app.dict_state.is_reverse_mode = false;
            app.dict_state.reverse_results.clear();
            if let Some(item) = dict::lookup(&q) {
                app.dict_state.current_item = Some(item);
                app.dict_state.not_found = false;
                app.dict_state.suggestions.clear();
            } else {
                app.dict_state.current_item = None;
                app.dict_state.not_found = true;
                app.dict_state.suggestions = dict::suggest(&q, 8);
            }
        }
    } else if !app.dict_state.query.trim().is_empty() && app.dict_state.current_item.is_none() && !app.dict_state.is_reverse_mode {
        // Live auto-suggestions as user types English
        let q = app.dict_state.query.trim();
        if !dict::is_myanmar_text(q) {
            app.dict_state.suggestions = dict::suggest(q, 6);
        }
    }

    // Display suggestions if available
    if !app.dict_state.suggestions.is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new("အကြံပြု စကားလုံးများ (Suggestions):").size(11.0).color(Color32::from_rgb(0x8A, 0x5D, 0x72)));
        ui.horizontal_wrapped(|ui| {
            let suggestions = app.dict_state.suggestions.clone();
            for word in suggestions {
                if ui.button(RichText::new(&word).size(11.5)).clicked() {
                    app.dict_state.query = word.clone();
                    if let Some(item) = dict::lookup(&word) {
                        app.dict_state.current_item = Some(item);
                        app.dict_state.not_found = false;
                        app.dict_state.suggestions.clear();
                    }
                }
            }
        });
    }

    // Display Reverse Matches pills (when searching by Myanmar word)
    if app.dict_state.is_reverse_mode && !app.dict_state.reverse_results.is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new(format!("ကိုက်ညီသော အင်္ဂလိပ် စကားလုံးများ ({} ခု):", app.dict_state.reverse_results.len())).size(11.0).color(Color32::from_rgb(0x8A, 0x5D, 0x72)).strong());
        ui.horizontal_wrapped(|ui| {
            let rev_list = app.dict_state.reverse_results.clone();
            for (w, item) in rev_list {
                let is_selected = app.dict_state.current_item.as_ref().map_or(false, |cur| cur.word == w);
                let text = if is_selected {
                    RichText::new(&w).strong().color(Color32::from_rgb(0x9E, 0x1B, 0x46))
                } else {
                    RichText::new(&w)
                };
                if ui.button(text).clicked() {
                    app.dict_state.current_item = Some(item);
                }
            }
        });
    }

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    // Definition Card or Welcome Guide
    if let Some(item) = &app.dict_state.current_item.clone() {
        render_definition_card(app, ui, item);
    } else if app.dict_state.not_found {
        egui::Frame::new()
            .fill(Color32::from_rgba_unmultiplied(220, 100, 100, 15))
            .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(220, 100, 100, 50)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(RichText::new("ရှာဖွေမတွေ့ရှိပါ (Word not found)").color(Color32::from_rgb(0xC4, 0x1E, 0x3A)).strong());
                ui.label(RichText::new(format!("\"{}\" စကားလုံးကို အဘိဓာန်တွင် ရှာမတွေ့ပါ။ စာလုံးပေါင်း မှန်ကန်မှု ရှိမရှိ စစ်ဆေးပေးပါ။", app.dict_state.last_searched)).size(11.5));
            });
    } else {
        render_welcome_guide(app, ui);
    }
}

/// Render definition details with Pyidaungsu typography and speech synthesis.
fn render_definition_card(app: &mut WordApp, ui: &mut Ui, item: &DictItem) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(255, 250, 252, 230))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(232, 74, 127, 60)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(12.0)
        .show(ui, |ui| {
            // Headword, Audio TTS button, and Part of Speech pill
            ui.horizontal(|ui| {
                ui.label(RichText::new(&item.word).size(20.0).strong().color(Color32::from_rgb(0x9E, 0x1B, 0x46)));

                // Audio Pronunciation TTS button (🔊)
                if ui.button(RichText::new("🔊").size(15.0)).on_hover_text("အသံထွက် နားထောင်မည် (Pronounce word)").clicked() {
                    dict::speak(&item.word);
                }

                if !item.pos.is_empty() {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(0xFD, 0xDF, 0xEC))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&item.pos).size(11.0).color(Color32::from_rgb(0x9E, 0x1B, 0x46)));
                        });
                }
            });

            // Pronunciation (IPA + Myanmar phonetic approximation)
            if !item.pronunciation.is_empty() {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&item.pronunciation).size(12.5).italics().color(Color32::from_rgb(0x6E, 0x5A, 0x64)));
                });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Myanmar Definition
            ui.label(RichText::new("မြန်မာ အဓိပ္ပာယ်:").size(11.5).strong().color(Color32::from_rgb(0x7D, 0x4A, 0x62)));
            ui.add_space(2.0);
            ui.label(RichText::new(&item.definition).size(14.0).color(Color32::from_rgb(0x22, 0x15, 0x1C)));

            // Example sentences
            if !item.examples.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new("ဥပမာ သုံးစွဲပုံများ (Examples):").size(11.5).strong().color(Color32::from_rgb(0x7D, 0x4A, 0x62)));
                ui.add_space(2.0);
                for ex in &item.examples {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("•").color(Color32::from_rgb(0xE8, 0x4A, 0x7F)));
                        ui.label(RichText::new(ex).size(12.0).italics().color(Color32::from_rgb(0x40, 0x30, 0x38)));
                    });
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // Action buttons: Insert, Copy, Speak
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📥 စာသားထဲထည့်မည်").strong().color(Color32::from_rgb(0x9E, 0x1B, 0x46))).clicked() {
                    let _ = app.run("dict.insert", json!({"text": format!("{} : {}", item.word, item.definition)}));
                }

                if ui.button(RichText::new("📋 ကူးယူမည်").size(11.5)).clicked() {
                    ui.ctx().copy_text(item.plain_text.clone());
                }

                if ui.button(RichText::new("🔊 အသံထွက်").size(11.5)).clicked() {
                    dict::speak(&item.word);
                }
            });
        });
}

/// Render welcome suggestions when no word is being viewed.
fn render_welcome_guide(app: &mut WordApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new("📚 နှစ်ဖက်သွား အဘိဓာန် (English ⇄ Myanmar)").size(13.5).strong().color(Color32::from_rgb(0x8A, 0x5D, 0x72)));
        ui.add_space(4.0);
        ui.label(RichText::new("အင်္ဂလိပ် သို့မဟုတ် မြန်မာစာဖြင့် ရှာဖွေနိုင်ပြီး အသံထွက် နားထောင်ခြင်းနှင့် အဓိပ္ပာယ်များကို တစ်ခါတည်း ကြည့်ရှုနိုင်ပါသည်။").size(11.5).color(Color32::from_rgb(0x70, 0x60, 0x68)));
        ui.add_space(10.0);
    });

    ui.label(RichText::new("နမူနာ စကားလုံးများ (အင်္ဂလိပ် / မြန်မာ):").size(11.5).strong());
    ui.horizontal_wrapped(|ui| {
        let samples = [
            ("စာအုပ်", true),
            ("book", false),
            ("မေတ္တာ", true),
            ("love", false),
            ("ကျောင်း", true),
            ("school", false),
            ("ကွန်ပျူတာ", true),
            ("writing", false),
            ("မင်္ဂလာပါ", true),
            ("freedom", false),
        ];
        for (s, is_my) in samples {
            if ui.button(RichText::new(s).size(11.5)).clicked() {
                app.dict_state.query = s.to_string();
                app.dict_state.last_searched = s.to_string();
                if is_my {
                    app.dict_state.is_reverse_mode = true;
                    let results = dict::reverse_search(s, 10);
                    app.dict_state.reverse_results = results.clone();
                    if let Some((_, item)) = results.first() {
                        app.dict_state.current_item = Some(item.clone());
                        app.dict_state.not_found = false;
                    }
                } else {
                    app.dict_state.is_reverse_mode = false;
                    app.dict_state.reverse_results.clear();
                    if let Some(item) = dict::lookup(s) {
                        app.dict_state.current_item = Some(item);
                        app.dict_state.not_found = false;
                    }
                }
            }
        }
    });
}
