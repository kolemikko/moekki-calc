use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Day {
    pub name: String,
    pub servings: Servings,
    pub total_day_rate: f64,
    pub breakfast_day_rate: f64,
    pub breakfast_attendance_count: usize,
    pub lunch_day_rate: f64,
    pub lunch_attendance_count: usize,
    pub dinner_day_rate: f64,
    pub dinner_attendance_count: usize,
    pub snacks_day_rate: f64,
    pub snacks_attendance_count: usize,
}

impl Day {
    pub fn new(name: String) -> Self {
        Self {
            name,
            servings: Servings::new(),
            total_day_rate: 0.0,
            breakfast_day_rate: 0.0,
            breakfast_attendance_count: 0,
            lunch_day_rate: 0.0,
            lunch_attendance_count: 0,
            dinner_day_rate: 0.0,
            dinner_attendance_count: 0,
            snacks_day_rate: 0.0,
            snacks_attendance_count: 0,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Servings {
    pub breakfast: bool,
    pub lunch: bool,
    pub dinner: bool,
    pub snacks: bool,
}

impl Servings {
    pub fn new() -> Self {
        Self {
            breakfast: true,
            lunch: true,
            dinner: true,
            snacks: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Expense {
    pub name: String,
    pub price: f64,
    pub serving_type: Servings,
    pub specific_day: bool,
    pub target_day: String,
}

impl Expense {
    pub fn new(name: String, price: f64) -> Self {
        Self {
            name,
            price,
            serving_type: Servings::default(),
            specific_day: false,
            target_day: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Attendance {
    pub day_name: String,
    pub present: bool,
    pub servings: Servings,
}

impl Attendance {
    pub fn new(day_name: String) -> Self {
        Self {
            day_name,
            present: false,
            servings: Servings::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub attendance: Vec<Attendance>,
    pub cost: f64,
}

impl Person {
    pub fn new(name: String, days: &[Day]) -> Self {
        let mut attendance: Vec<Attendance> = Vec::new();
        for d in days.iter() {
            attendance.push(Attendance {
                day_name: d.name.clone(),
                present: false,
                servings: Servings::new(),
            });
        }
        Self {
            name,
            attendance,
            cost: 0.0,
        }
    }
}
