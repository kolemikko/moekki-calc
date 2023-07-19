use crate::types::{Attendance, Day, Expense, Person};
use currency_rs::{Currency, CurrencyOpts};
use egui::{
    epaint::{Color32, Stroke},
    RichText, Rounding, Vec2,
};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct MoekkiCalcApp {
    #[serde(skip)]
    currency_opts_eur: CurrencyOpts,

    expenses: Vec<Expense>,
    #[serde(skip)]
    new_expense_name: String,
    #[serde(skip)]
    new_expense_price: f64,
    #[serde(skip)]
    expenses_to_remove: Vec<usize>,

    total_breakfast_cost: f64,
    total_lunch_cost: f64,
    total_dinner_cost: f64,
    total_snacks_cost: f64,
    total_cost: f64,

    days: Vec<Day>,
    #[serde(skip)]
    days_to_remove: Vec<usize>,
    people: Vec<Person>,
    #[serde(skip)]
    new_person_name: String,
    #[serde(skip)]
    people_to_remove: Vec<usize>,

    #[serde(skip)]
    update_attendances: bool,
    #[serde(skip)]
    update_costs: bool,
}

impl Default for MoekkiCalcApp {
    fn default() -> Self {
        Self {
            currency_opts_eur: CurrencyOpts::new()
                .set_pattern("#!")
                .set_symbol("€")
                .set_separator(","),
            expenses: Vec::new(),
            new_expense_name: String::new(),
            new_expense_price: 0.0,
            total_breakfast_cost: 0.0,
            total_lunch_cost: 0.0,
            total_dinner_cost: 0.0,
            total_snacks_cost: 0.0,
            total_cost: 0.0,
            days: Vec::new(),
            days_to_remove: Vec::new(),
            expenses_to_remove: Vec::new(),
            people: Vec::new(),
            new_person_name: String::new(),
            people_to_remove: Vec::new(),
            update_attendances: false,
            update_costs: false,
        }
    }
}

impl MoekkiCalcApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn update_costs(&mut self) {
        let mut total = 0.0;
        let mut total_breakfast = 0.0;
        let mut total_lunch = 0.0;
        let mut total_dinner = 0.0;
        let mut total_snacks = 0.0;
        for e in self.expenses.iter() {
            total += e.price;
            let mut divided = 0.0;
            if e.serving_type.breakfast {
                divided += 1.0;
            }
            if e.serving_type.lunch {
                divided += 1.0;
            }
            if e.serving_type.dinner {
                divided += 1.0;
            }
            if e.serving_type.snacks {
                divided += 1.0;
            }

            let divided_price = e.price / divided;
            if e.serving_type.breakfast {
                total_breakfast += divided_price;
            }
            if e.serving_type.lunch {
                total_lunch += divided_price;
            }
            if e.serving_type.dinner {
                total_dinner += divided_price;
            }
            if e.serving_type.snacks {
                total_snacks += divided_price;
            }
        }

        self.total_cost = total;
        self.total_breakfast_cost = total_breakfast;
        self.total_lunch_cost = total_lunch;
        self.total_dinner_cost = total_dinner;
        self.total_snacks_cost = total_snacks;

        let breakfast_divided = self.days.iter().filter(|x| x.servings.breakfast).count();

        let lunch_divided = self.days.iter().filter(|x| x.servings.lunch).count();

        let dinner_divided = self.days.iter().filter(|x| x.servings.dinner).count();

        let snacks_divided = self.days.iter().filter(|x| x.servings.snacks).count();

        for d in self.days.iter_mut() {
            if d.servings.breakfast {
                d.breakfast_day_rate = self.total_breakfast_cost / breakfast_divided as f64;
            } else {
                d.breakfast_day_rate = 0.0;
            }
            if d.servings.lunch {
                d.lunch_day_rate = self.total_lunch_cost / lunch_divided as f64;
            } else {
                d.lunch_day_rate = 0.0;
            }
            if d.servings.dinner {
                d.dinner_day_rate = self.total_dinner_cost / dinner_divided as f64;
            } else {
                d.dinner_day_rate = 0.0;
            }
            if d.servings.snacks {
                d.snacks_day_rate = self.total_snacks_cost / snacks_divided as f64;
            } else {
                d.snacks_day_rate = 0.0;
            }
            d.total_day_rate =
                d.breakfast_day_rate + d.lunch_day_rate + d.dinner_day_rate + d.snacks_day_rate;
        }

        for p in self.people.iter_mut() {
            let mut total_cost = 0.0;
            for (idx, a) in p.attendance.iter().enumerate() {
                if a.present {
                    let day = self.days.get(idx).unwrap();
                    if a.servings.breakfast {
                        total_cost +=
                            day.breakfast_day_rate / day.breakfast_attendance_count as f64;
                    }
                    if a.servings.lunch {
                        total_cost += day.lunch_day_rate / day.lunch_attendance_count as f64;
                    }
                    if a.servings.dinner {
                        total_cost += day.dinner_day_rate / day.dinner_attendance_count as f64;
                    }
                    if a.servings.snacks {
                        total_cost += day.snacks_day_rate / day.snacks_attendance_count as f64;
                    }
                }
            }
            p.cost = total_cost;
        }
        self.update_costs = false;
    }

    fn update_attendances(&mut self) {
        for (idx, d) in self.days.iter_mut().enumerate() {
            if d.servings.breakfast {
                d.breakfast_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        a.present && a.servings.breakfast
                    })
                    .count();
            } else {
                for p in self.people.iter_mut() {
                    p.attendance.get_mut(idx).unwrap().servings.breakfast = false;
                }
            }
            if d.servings.lunch {
                d.lunch_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        a.present && a.servings.lunch
                    })
                    .count();
            } else {
                for p in self.people.iter_mut() {
                    p.attendance.get_mut(idx).unwrap().servings.lunch = false;
                }
            }
            if d.servings.dinner {
                d.dinner_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        a.present && a.servings.dinner
                    })
                    .count();
            } else {
                for p in self.people.iter_mut() {
                    p.attendance.get_mut(idx).unwrap().servings.dinner = false;
                }
            }
            if d.servings.snacks {
                d.snacks_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        a.present && a.servings.snacks
                    })
                    .count();
            } else {
                for p in self.people.iter_mut() {
                    p.attendance.get_mut(idx).unwrap().servings.snacks = false;
                }
            }
        }
        self.update_attendances = false;
    }

    fn update_removed(&mut self) {
        while let Some(idx) = self.expenses_to_remove.pop() {
            self.expenses.remove(idx);
        }
        while let Some(idx) = self.people_to_remove.pop() {
            self.people.remove(idx);
        }
        while let Some(idx) = self.days_to_remove.pop() {
            self.days.remove(idx);
            for p in self.people.iter_mut() {
                p.attendance.remove(idx);
            }
        }
        self.update_costs = true;
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top-panel")
            .frame(
                egui::Frame::none()
                    .stroke(Stroke::new(1.0, Color32::GRAY))
                    .inner_margin(egui::style::Margin::symmetric(10.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Moekki-Calc");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("Reset session").clicked() {
                            self.days.clear();
                            self.people.clear();
                            self.expenses.clear();
                        }
                    });
                });
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(10, 25, 30))
                    .inner_margin(egui::style::Margin::symmetric(80.0, 50.0)),
            )
            .show(ctx, |ui| {
                self.update_removed();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.render_days_frame(ui);
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        self.render_balances_frame(ui);
                    });
                });
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.render_people_frame(ui);
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        self.render_expenses_frame(ui);
                    });
                });

                if self.update_attendances {
                    self.update_attendances();
                }
                if self.update_costs {
                    self.update_costs();
                }
            });
    }

    fn render_days_frame(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .rounding(Rounding::same(20.0))
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .inner_margin(egui::style::Margin::symmetric(20.0, 20.0))
            .show(ui, |ui| {
                ui.heading("Trip definition");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Add day")).clicked() {
                        let day_name = format!("{}", self.days.len() + 1);
                        self.days.push(Day::new(day_name.clone()));
                        for p in self.people.iter_mut() {
                            p.attendance.push(Attendance::new(day_name.clone()));
                        }
                    }
                    if ui.add(egui::Button::new("Remove day")).clicked() {
                        self.days_to_remove.push(self.days.len() - 1);
                    }
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    for d in self.days.iter_mut() {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(format!("Day {}", &d.name)).strong());
                            let resp1 = ui.checkbox(&mut d.servings.breakfast, "Breakfast");
                            let resp2 = ui.checkbox(&mut d.servings.lunch, "Lunch");
                            let resp3 = ui.checkbox(&mut d.servings.dinner, "Dinner");
                            let resp4 = ui.checkbox(&mut d.servings.snacks, "Snacks");
                            if resp1.changed()
                                || resp2.changed()
                                || resp3.changed()
                                || resp4.changed
                            {
                                self.update_attendances = true;
                            }
                            ui.add_space(5.0);
                            if d.servings.breakfast {
                                ui.label(format!(
                                    "Breakfast: {}",
                                    Currency::new_string(
                                        &d.breakfast_day_rate.to_string(),
                                        Some(self.currency_opts_eur.clone())
                                    )
                                    .unwrap()
                                    .format()
                                ));
                            }
                            if d.servings.lunch {
                                ui.label(format!(
                                    "Lunch: {}",
                                    Currency::new_string(
                                        &d.lunch_day_rate.to_string(),
                                        Some(self.currency_opts_eur.clone())
                                    )
                                    .unwrap()
                                    .format()
                                ));
                            }
                            if d.servings.dinner {
                                ui.label(format!(
                                    "Dinner: {}",
                                    Currency::new_string(
                                        &d.dinner_day_rate.to_string(),
                                        Some(self.currency_opts_eur.clone())
                                    )
                                    .unwrap()
                                    .format()
                                ));
                            }
                            if d.servings.snacks {
                                ui.label(format!(
                                    "Snacks: {}",
                                    Currency::new_string(
                                        &d.snacks_day_rate.to_string(),
                                        Some(self.currency_opts_eur.clone())
                                    )
                                    .unwrap()
                                    .format()
                                ));
                            }
                            ui.label(format!(
                                "Total: {}",
                                Currency::new_string(
                                    &d.total_day_rate.to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                        });
                        ui.add_space(10.0);
                    }
                });
            });
    }

    fn render_balances_frame(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .rounding(Rounding::same(20.0))
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .inner_margin(egui::style::Margin::symmetric(20.0, 20.0))
            .show(ui, |ui| {
                ui.heading("Balances");
                ui.add_space(8.0);
                for p in self.people.iter() {
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            p.name,
                            Currency::new_string(
                                &p.cost.to_string(),
                                Some(self.currency_opts_eur.clone())
                            )
                            .unwrap()
                            .format()
                        ))
                        .strong(),
                    );
                    ui.add_space(5.0);
                }
                ui.add_space(10.0);
                let covered: f64 = self.people.iter().map(|x| x.cost).sum();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Expenses covered: {} / {}",
                        Currency::new_string(
                            &covered.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format(),
                        Currency::new_string(
                            &self.total_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    if covered < self.total_cost {
                        ui.label(RichText::new("!").color(Color32::RED).strong())
                            .on_hover_text("All expenses are not covered yet");
                    }
                });
            });
    }

    fn render_people_frame(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .rounding(Rounding::same(20.0))
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .inner_margin(egui::style::Margin::symmetric(20.0, 20.0))
            .show(ui, |ui| {
                ui.heading("People & Attendance");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add_sized(
                        Vec2::new(150.0, 10.0),
                        egui::TextEdit::singleline(&mut self.new_person_name),
                    );
                });
                ui.add_space(5.0);
                let allow_add_person = !self.new_person_name.is_empty();
                if ui
                    .add_enabled(allow_add_person, egui::Button::new("Add person"))
                    .clicked()
                {
                    self.people
                        .push(Person::new(self.new_person_name.clone(), &self.days));
                    self.new_person_name = String::new();
                    self.update_costs = true;
                }
                ui.add_space(20.0);

                egui::ScrollArea::vertical()
                    .id_source("people-scrollarea")
                    .min_scrolled_height(600.0)
                    .show(ui, |ui| {
                        for (idx, p) in self.people.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&p.name).strong());
                                if ui.add(egui::Button::new("x")).clicked() {
                                    self.people_to_remove.push(idx);
                                }
                            });
                            ui.horizontal(|ui| {
                                for d in p.attendance.iter_mut() {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Day {}", &d.day_name));
                                            let resp = ui.checkbox(&mut d.present, "Present");
                                            if resp.changed() {
                                                self.update_attendances = true;
                                            }
                                        });
                                        if d.present {
                                            let resp1 =
                                                ui.checkbox(&mut d.servings.breakfast, "Breakfast");
                                            let resp2 = ui.checkbox(&mut d.servings.lunch, "Lunch");
                                            let resp3 =
                                                ui.checkbox(&mut d.servings.dinner, "Dinner");
                                            let resp4 =
                                                ui.checkbox(&mut d.servings.snacks, "Snacks");
                                            if resp1.changed()
                                                || resp2.changed()
                                                || resp3.changed()
                                                || resp4.changed
                                            {
                                                self.update_attendances = true;
                                            }
                                        }
                                    });
                                    ui.add_space(10.0);
                                }
                            });
                            ui.add_space(25.0);
                        }
                    });
            });
    }

    fn render_expenses_frame(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .rounding(Rounding::same(20.0))
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .inner_margin(egui::style::Margin::symmetric(20.0, 20.0))
            .show(ui, |ui| {
                ui.heading("Expenses");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add_sized(
                        Vec2::new(150.0, 10.0),
                        egui::TextEdit::singleline(&mut self.new_expense_name),
                    );
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    ui.add(
                        egui::DragValue::new(&mut self.new_expense_price)
                            .speed(0.1)
                            .max_decimals(2)
                            .clamp_range(RangeInclusive::new(0.0, 1000.0)),
                    );
                    ui.label(self.currency_opts_eur.symbol());
                });
                ui.add_space(5.0);
                let allow_add_expense =
                    !self.new_expense_name.is_empty() && self.new_expense_price > 0.0;
                if ui
                    .add_enabled(allow_add_expense, egui::Button::new("Add expense"))
                    .clicked()
                {
                    self.expenses.push(Expense::new(
                        self.new_expense_name.clone(),
                        self.new_expense_price,
                    ));
                    self.new_expense_name = String::new();
                    self.new_expense_price = 0.0;
                    self.update_costs = true;
                }
                ui.add_space(10.0);
                egui::ScrollArea::vertical()
                    .id_source("expenses-scrollarea")
                    .min_scrolled_height(600.0)
                    .show(ui, |ui| {
                        for (idx, e) in self.expenses.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&e.name).strong());
                                ui.label(
                                    Currency::new_string(
                                        &e.price.to_string(),
                                        Some(self.currency_opts_eur.clone()),
                                    )
                                    .unwrap()
                                    .format(),
                                );
                                if ui.add(egui::Button::new("x")).clicked() {
                                    self.expenses_to_remove.push(idx);
                                }
                            });
                            ui.horizontal(|ui| {
                                let resp1 = ui.checkbox(&mut e.serving_type.breakfast, "Breakfast");
                                let resp2 = ui.checkbox(&mut e.serving_type.lunch, "Lunch");
                                let resp3 = ui.checkbox(&mut e.serving_type.dinner, "Dinner");
                                let resp4 = ui.checkbox(&mut e.serving_type.snacks, "Snacks");
                                // ui.checkbox(&mut e.specific_day, "Only for one day");
                                // if e.specific_day {
                                //     egui::ComboBox::from_id_source("target-day")
                                //         .selected_text(format!("{:?}", e.target_day))
                                //         .show_ui(ui, |ui| {
                                //             for d in self.days.iter() {
                                //                 ui.selectable_value(
                                //                     &mut e.target_day,
                                //                     d.name.clone(),
                                //                     d.name.clone(),
                                //                 );
                                //             }
                                //         });
                                // }
                                ui.add_space(10.0);
                                if !e.serving_type.breakfast
                                    && !e.serving_type.lunch
                                    && !e.serving_type.dinner
                                    && !e.serving_type.snacks
                                {
                                    ui.label(RichText::new("!").color(Color32::RED).strong())
                                        .on_hover_text(
                                            "Expense must be assigned to at least one serving",
                                        );
                                }
                                if resp1.changed()
                                    || resp2.changed()
                                    || resp3.changed()
                                    || resp4.changed
                                {
                                    self.update_costs = true;
                                }
                            });
                            ui.add_space(10.0);
                        }
                    });
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Total: {}",
                        Currency::new_string(
                            &self.total_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Breakfast: {}",
                        Currency::new_string(
                            &self.total_breakfast_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Lunch: {}",
                        Currency::new_string(
                            &self.total_lunch_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Dinner: {}",
                        Currency::new_string(
                            &self.total_dinner_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Snacks: {}",
                        Currency::new_string(
                            &self.total_snacks_cost.to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                });
                ui.add_space(20.0);
            });
    }
}

impl eframe::App for MoekkiCalcApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_panel(ctx);
        self.render_central_panel(ctx);
    }
}
