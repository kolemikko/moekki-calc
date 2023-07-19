use currency_rs::{Currency, CurrencyOpts};
use egui::epaint::{Color32, Stroke};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MoekkiCalcApp {
    #[serde(skip)]
    currency_opts_eur: CurrencyOpts,
    #[serde(skip)]
    expenses: Vec<Expense>,
    #[serde(skip)]
    new_expense_name: String,
    #[serde(skip)]
    new_expense_price: f64,
    #[serde(skip)]
    expenses_to_remove: Vec<usize>,
    #[serde(skip)]
    total_breakfast_cost: Currency,
    #[serde(skip)]
    total_lunch_cost: Currency,
    #[serde(skip)]
    total_dinner_cost: Currency,
    #[serde(skip)]
    total_snacks_cost: Currency,
    #[serde(skip)]
    total_cost: Currency,

    #[serde(skip)]
    days: Vec<Day>,
    #[serde(skip)]
    days_to_remove: Vec<usize>,
    #[serde(skip)]
    people: Vec<Person>,
    #[serde(skip)]
    new_person_name: String,
    #[serde(skip)]
    people_to_remove: Vec<usize>,
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
            total_breakfast_cost: Currency::new_float(0.0, None),
            total_lunch_cost: Currency::new_float(0.0, None),
            total_dinner_cost: Currency::new_float(0.0, None),
            total_snacks_cost: Currency::new_float(0.0, None),
            total_cost: Currency::new_float(0.0, None),
            days: Vec::new(),
            days_to_remove: Vec::new(),
            expenses_to_remove: Vec::new(),
            people: Vec::new(),
            new_person_name: String::new(),
            people_to_remove: Vec::new(),
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
        self.total_cost = Currency::new_float(total, None);
        self.total_breakfast_cost = Currency::new_float(total_breakfast, None);
        self.total_lunch_cost = Currency::new_float(total_lunch, None);
        self.total_dinner_cost = Currency::new_float(total_dinner, None);
        self.total_snacks_cost = Currency::new_float(total_snacks, None);

        let breakfast_divided = self
            .days
            .iter()
            .filter(|x| x.servings.breakfast == true)
            .count();

        let lunch_divided = self
            .days
            .iter()
            .filter(|x| x.servings.lunch == true)
            .count();

        let dinner_divided = self
            .days
            .iter()
            .filter(|x| x.servings.dinner == true)
            .count();

        let snacks_divided = self
            .days
            .iter()
            .filter(|x| x.servings.snacks == true)
            .count();

        for d in self.days.iter_mut() {
            if d.servings.breakfast {
                d.breakfast_day_rate = Currency::new_float(self.total_breakfast_cost.value(), None)
                    .divide(breakfast_divided as f64);
            } else {
                d.breakfast_day_rate = Currency::new_float(0.0, None);
            }
            if d.servings.lunch {
                d.lunch_day_rate = Currency::new_float(self.total_lunch_cost.value(), None)
                    .divide(lunch_divided as f64);
            } else {
                d.lunch_day_rate = Currency::new_float(0.0, None);
            }
            if d.servings.dinner {
                d.dinner_day_rate = Currency::new_float(self.total_dinner_cost.value(), None)
                    .divide(dinner_divided as f64);
            } else {
                d.dinner_day_rate = Currency::new_float(0.0, None);
            }
            if d.servings.snacks {
                d.snacks_day_rate = Currency::new_float(self.total_snacks_cost.value(), None)
                    .divide(snacks_divided as f64);
            } else {
                d.snacks_day_rate = Currency::new_float(0.0, None);
            }
            d.total_day_rate = Currency::new_float(d.breakfast_day_rate.value(), None)
                .add(d.lunch_day_rate.value())
                .add(d.dinner_day_rate.value())
                .add(d.snacks_day_rate.value());
        }

        for p in self.people.iter_mut() {
            let mut total_cost = 0.0;
            for (idx, a) in p.attendance.iter().enumerate() {
                if a.present {
                    let day = self.days.get(idx).unwrap();
                    if a.servings.breakfast {
                        total_cost +=
                            day.breakfast_day_rate.value() / day.breakfast_attendance_count as f64;
                    }
                    if a.servings.lunch {
                        total_cost +=
                            day.lunch_day_rate.value() / day.lunch_attendance_count as f64;
                    }
                    if a.servings.dinner {
                        total_cost +=
                            day.dinner_day_rate.value() / day.dinner_attendance_count as f64;
                    }
                    if a.servings.snacks {
                        total_cost +=
                            day.snacks_day_rate.value() / day.snacks_attendance_count as f64;
                    }
                }
            }
            p.cost = Currency::new_float(total_cost, None);
        }
    }

    fn update_attendances(&mut self) {
        for (idx, d) in self.days.iter_mut().enumerate() {
            if d.servings.breakfast {
                d.breakfast_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        if a.present == true && a.servings.breakfast == true {
                            true
                        } else {
                            false
                        }
                    })
                    .count();
            }
            if d.servings.lunch {
                d.lunch_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        if a.present == true && a.servings.lunch == true {
                            true
                        } else {
                            false
                        }
                    })
                    .count();
            }
            if d.servings.dinner {
                d.dinner_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        if a.present == true && a.servings.dinner == true {
                            true
                        } else {
                            false
                        }
                    })
                    .count();
            }
            if d.servings.snacks {
                d.snacks_attendance_count = self
                    .people
                    .iter()
                    .filter(|x| {
                        let a = x.attendance.get(idx).unwrap();
                        if a.present == true && a.servings.snacks == true {
                            true
                        } else {
                            false
                        }
                    })
                    .count();
            }
        }
    }

    fn update_removed(&mut self) {
        while self.expenses_to_remove.len() > 0 {
            let idx = self.expenses_to_remove.pop().unwrap();
            self.expenses.remove(idx);
        }
        while self.people_to_remove.len() > 0 {
            let idx = self.people_to_remove.pop().unwrap();
            self.people.remove(idx);
        }
        while self.days_to_remove.len() > 0 {
            let idx = self.days_to_remove.pop().unwrap();
            self.days.remove(idx);
            for p in self.people.iter_mut() {
                p.attendance.remove(idx);
            }
        }
        self.update_costs();
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top-panel")
            .frame(
                egui::Frame::none()
                    .stroke(Stroke::new(1.0, Color32::GRAY))
                    .inner_margin(egui::style::Margin::symmetric(10.0, 10.0)), // .fill(Color32::DARK_GRAY),
            )
            .show(ctx, |ui| {
                ui.heading("Moekki-Calc");
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none().inner_margin(egui::style::Margin::symmetric(30.0, 30.0)), // .fill(Color32::DARK_GRAY),
            )
            .show(ctx, |ui| {
                self.update_removed();

                ui.add_space(20.0);
                ui.heading("Days");
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
                ui.horizontal(|ui| {
                    for d in self.days.iter_mut() {
                        ui.vertical(|ui| {
                            ui.label(format!("Day {}", &d.name));
                            ui.checkbox(&mut d.servings.breakfast, "Breakfast");
                            ui.checkbox(&mut d.servings.lunch, "Lunch");
                            ui.checkbox(&mut d.servings.dinner, "Dinner");
                            ui.checkbox(&mut d.servings.snacks, "Snacks");

                            ui.label(format!(
                                "Breakfast: {}",
                                Currency::new_string(
                                    &d.breakfast_day_rate.value().to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                            ui.label(format!(
                                "Lunch: {}",
                                Currency::new_string(
                                    &d.lunch_day_rate.value().to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                            ui.label(format!(
                                "Dinner: {}",
                                Currency::new_string(
                                    &d.dinner_day_rate.value().to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                            ui.label(format!(
                                "Snacks: {}",
                                Currency::new_string(
                                    &d.snacks_day_rate.value().to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                            ui.label(format!(
                                "Total: {}",
                                Currency::new_string(
                                    &d.total_day_rate.value().to_string(),
                                    Some(self.currency_opts_eur.clone())
                                )
                                .unwrap()
                                .format()
                            ));
                        });
                    }
                });
                ui.separator();
                ui.add_space(20.0);

                ui.heading("Expenses");
                ui.label("Name of new expense:");
                ui.add(egui::TextEdit::singleline(&mut self.new_expense_name));
                ui.add(
                    egui::DragValue::new(&mut self.new_expense_price)
                        .speed(0.1)
                        .max_decimals(2)
                        .clamp_range(RangeInclusive::new(0.0, 1000.0)),
                );
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
                    self.update_costs();
                }
                let mut update_costs = false;
                for (idx, e) in self.expenses.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&e.name);
                        ui.label(format!(
                            "Price: {}",
                            Currency::new_string(
                                &e.price.to_string(),
                                Some(self.currency_opts_eur.clone())
                            )
                            .unwrap()
                            .format()
                        ));
                        let resp1 = ui.checkbox(&mut e.serving_type.breakfast, "Breakfast");
                        let resp2 = ui.checkbox(&mut e.serving_type.lunch, "Lunch");
                        let resp3 = ui.checkbox(&mut e.serving_type.dinner, "Dinner");
                        let resp4 = ui.checkbox(&mut e.serving_type.snacks, "Snacks");
                        ui.checkbox(&mut e.specific_day, "Only for one day");
                        if e.specific_day {
                            egui::ComboBox::from_id_source("target-day")
                                .selected_text(format!("{:?}", e.target_day))
                                .show_ui(ui, |ui| {
                                    for d in self.days.iter() {
                                        ui.selectable_value(
                                            &mut e.target_day,
                                            d.name.clone(),
                                            d.name.clone(),
                                        );
                                    }
                                });
                        }
                        if ui.add(egui::Button::new("x")).clicked() {
                            self.expenses_to_remove.push(idx);
                        }
                        if resp1.changed() || resp2.changed() || resp3.changed() || resp4.changed {
                            update_costs = true;
                        }
                    });
                }
                if update_costs {
                    self.update_costs();
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Total: {}",
                        Currency::new_string(
                            &self.total_cost.value().to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Breakfast: {}",
                        Currency::new_string(
                            &self.total_breakfast_cost.value().to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Lunch: {}",
                        Currency::new_string(
                            &self.total_lunch_cost.value().to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Dinner: {}",
                        Currency::new_string(
                            &self.total_dinner_cost.value().to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                    ui.label(format!(
                        "Snacks: {}",
                        Currency::new_string(
                            &self.total_snacks_cost.value().to_string(),
                            Some(self.currency_opts_eur.clone())
                        )
                        .unwrap()
                        .format()
                    ));
                });
                ui.separator();
                ui.add_space(20.0);

                ui.heading("People");
                ui.label("Name of new person:");
                ui.add(egui::TextEdit::singleline(&mut self.new_person_name));
                let allow_add_person = !self.new_person_name.is_empty();
                if ui
                    .add_enabled(allow_add_person, egui::Button::new("Add person"))
                    .clicked()
                {
                    self.people
                        .push(Person::new(self.new_person_name.clone(), &self.days));
                    self.new_person_name = String::new();
                    self.update_costs();
                }
                ui.add_space(20.0);

                let mut update_attendances = false;
                for (idx, p) in self.people.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Name: {}, Cost: {}",
                            p.name,
                            Currency::new_string(
                                &p.cost.value().to_string(),
                                Some(self.currency_opts_eur.clone())
                            )
                            .unwrap()
                            .format()
                        ));
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
                                        update_attendances = true;
                                    }
                                });
                                if d.present {
                                    let resp1 = ui.checkbox(&mut d.servings.breakfast, "Breakfast");
                                    let resp2 = ui.checkbox(&mut d.servings.lunch, "Lunch");
                                    let resp3 = ui.checkbox(&mut d.servings.dinner, "Dinner");
                                    let resp4 = ui.checkbox(&mut d.servings.snacks, "Snacks");
                                    if resp1.changed()
                                        || resp2.changed()
                                        || resp3.changed()
                                        || resp4.changed
                                    {
                                        update_attendances = true;
                                    }
                                }
                            });
                            ui.add_space(10.0);
                        }
                    });
                    ui.add_space(10.0);
                }
                if update_attendances {
                    self.update_attendances();
                }
            });
    }
}

impl eframe::App for MoekkiCalcApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.render_top_panel(ctx);
        self.render_central_panel(ctx);
    }
}

use std::{future::Future, ops::RangeInclusive};

use crate::types::{Attendance, Day, Expense, Person};
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
