//! Interactive GUI for Loan Amortization Calculator
//! Run with: cargo run --bin loan-calc-gui --features gui

#![cfg(feature = "gui")]

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use plotters::prelude::*;
use std::collections::HashMap;
use itertools::Itertools;

/// Amortization schedule data
#[derive(Debug, Clone)]
struct AmortizationSchedule {
    years: Vec<f64>,
    months: Vec<u32>,
    balance: Vec<f64>,
    equity: Vec<f64>,
    principal_paid: Vec<f64>,
    interest_paid: Vec<f64>,
    total_paid: Vec<f64>,
    home_value: Vec<f64>,
    monthly_payment: f64,
}

/// Loan calculator
struct LoanCalculator {
    home_price: f64,
    down_payment: f64,
    loan_amount: f64,
    interest_rate: f64,
    loan_term_years: u32,
    appreciation_rate: f64,
}

impl LoanCalculator {
    fn new(
        home_price: f64,
        down_payment_percent: f64,
        interest_rate: f64,
        loan_term_years: u32,
        appreciation_rate: f64,
    ) -> Self {
        let down_payment = home_price * down_payment_percent / 100.0;
        let loan_amount = home_price - down_payment;

        Self {
            home_price,
            down_payment,
            loan_amount,
            interest_rate: interest_rate / 100.0,
            loan_term_years,
            appreciation_rate: appreciation_rate / 100.0,
        }
    }

    fn calculate_schedule(&self, extra_monthly: f64) -> AmortizationSchedule {
        let monthly_rate = self.interest_rate / 12.0;
        let num_payments = self.loan_term_years as usize * 12;

        let base_monthly_payment = if monthly_rate > 0.0 {
            self.loan_amount * (monthly_rate * (1.0 + monthly_rate).powi(num_payments as i32))
                / ((1.0 + monthly_rate).powi(num_payments as i32) - 1.0)
        } else {
            self.loan_amount / num_payments as f64
        };
        let monthly_payment = base_monthly_payment + extra_monthly;

        let mut balance = self.loan_amount;
        let mut home_value = self.home_price;
        let mut cumulative_principal = self.down_payment;
        let mut cumulative_interest = 0.0;

        let mut years = Vec::with_capacity(num_payments);
        let mut months = Vec::with_capacity(num_payments);
        let mut balances = Vec::with_capacity(num_payments);
        let mut equity = Vec::with_capacity(num_payments);
        let mut principal_paid = Vec::with_capacity(num_payments);
        let mut interest_paid = Vec::with_capacity(num_payments);
        let mut total_paid = Vec::with_capacity(num_payments);
        let mut home_values = Vec::with_capacity(num_payments);

        for month in 1..=num_payments {
            let interest_payment = balance * monthly_rate;
            let principal_payment = (monthly_payment - interest_payment).min(balance);

            balance -= principal_payment;
            cumulative_principal += principal_payment;
            cumulative_interest += interest_payment;

            if month % 12 == 0 {
                home_value *= 1.0 + self.appreciation_rate;
            }

            let current_equity = cumulative_principal + (home_value - self.home_price);

            years.push(month as f64 / 12.0);
            months.push(month as u32);
            balances.push(balance);
            equity.push(current_equity);
            principal_paid.push(cumulative_principal);
            interest_paid.push(cumulative_interest);
            total_paid.push(cumulative_principal + cumulative_interest);
            home_values.push(home_value);
        }

        AmortizationSchedule {
            years,
            months,
            balance: balances,
            equity,
            principal_paid,
            interest_paid,
            total_paid,
            home_value: home_values,
            monthly_payment,
        }
    }
}

/// Loan parameters state for the GUI
#[derive(Debug, Clone)]
struct LoanParams {
    home_price: f32,
    down_payment_percent: f32,
    interest_rate: f32,
    loan_term_years: u32,
    appreciation_rate: f32,
    chart_width: u32,
    chart_height: u32,
    font_size: u32,
    // Additional monthly costs
    property_tax_rate: f32,
    insurance_rate: f32,
    pmi_rate: f32,
    monthly_hoa: f32,
    closing_cost_percent: f32,
    monthly_gross_income: f32,
    other_monthly_debts: f32,
    // Rent vs Buy
    monthly_rent: f32,
    rent_inflation: f32,
    stock_return: f32,
}

impl Default for LoanParams {
    fn default() -> Self {
        Self {
            home_price: 450_000.0,
            down_payment_percent: 20.0,
            interest_rate: 6.5,
            loan_term_years: 30,
            appreciation_rate: 3.0,
            chart_width: 800,
            chart_height: 600,
            font_size: 20,
            property_tax_rate: 1.1,
            insurance_rate: 1.5,
            pmi_rate: 0.85,
            monthly_hoa: 0.0,
            closing_cost_percent: 2.5,
            monthly_gross_income: 10_000.0,
            other_monthly_debts: 500.0,
            monthly_rent: 2_000.0,
            rent_inflation: 3.0,
            stock_return: 7.0,
        }
    }
}

/// The main GUI application
struct LoanCalcGui {
    params: LoanParams,
    chart_textures: HashMap<String, egui::TextureHandle>,
    scenarios: HashMap<String, AmortizationSchedule>,
    show_scenarios: [bool; 6],
    scenario_names: Vec<String>,
    selected_tab: String,
    regenerate_chart: bool,
    show_amort_table: bool,
    stacked_chart: bool,
}

impl LoanCalcGui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            params: LoanParams::default(),
            chart_textures: HashMap::new(),
            scenarios: HashMap::new(),
            show_scenarios: [true, true, true, true, true, true],
            scenario_names: vec![
                "Base Case".to_string(),
                "High Rate".to_string(),
                "Low Down".to_string(),
                "Extra Principal".to_string(),
                "With Disasters".to_string(),
                "Bi-weekly".to_string(),
            ],
            selected_tab: "Base Case".to_string(),
            regenerate_chart: true,
            show_amort_table: false,
            stacked_chart: false,
        }
    }

    fn calculate_all_scenarios(&mut self) {
        let base_calc = LoanCalculator::new(
            self.params.home_price as f64,
            self.params.down_payment_percent as f64,
            self.params.interest_rate as f64,
            self.params.loan_term_years,
            self.params.appreciation_rate as f64,
        );

        self.scenarios.clear();

        // Base case
        if self.show_scenarios[0] {
            self.scenarios.insert(
                self.scenario_names[0].clone(),
                base_calc.calculate_schedule(0.0),
            );
        }

        // High rate
        if self.show_scenarios[1] {
            let calc = LoanCalculator::new(
                self.params.home_price as f64,
                self.params.down_payment_percent as f64,
                8.0,
                self.params.loan_term_years,
                self.params.appreciation_rate as f64,
            );
            self.scenarios.insert(
                format!("High Rate (8%)"),
                calc.calculate_schedule(0.0),
            );
        }

        // Low down
        if self.show_scenarios[2] {
            let calc = LoanCalculator::new(
                self.params.home_price as f64,
                3.5,
                self.params.interest_rate as f64,
                self.params.loan_term_years,
                self.params.appreciation_rate as f64,
            );
            self.scenarios.insert(
                format!("Low Down (3.5%)"),
                calc.calculate_schedule(0.0),
            );
        }

        // Extra principal
        if self.show_scenarios[3] {
            self.scenarios.insert(
                format!("Extra Principal (+$200/mo)"),
                base_calc.calculate_schedule(200.0),
            );
        }

        // Disasters
        if self.show_scenarios[4] {
            let calc = LoanCalculator::new(
                self.params.home_price as f64,
                self.params.down_payment_percent as f64,
                self.params.interest_rate as f64,
                self.params.loan_term_years,
                2.0,
            );
            let mut schedule = calc.calculate_schedule(0.0);
            for e in schedule.equity.iter_mut() {
                *e -= 40_000.0;
            }
            self.scenarios.insert("With Disasters".to_string(), schedule);
        }

        // Bi-weekly payments: paying half the monthly payment every two weeks
        // = 26 half-payments/year = 13 full payments vs 12, equivalent to
        // one extra payment per year (extra_monthly = base_payment / 12).
        if self.show_scenarios[5] {
            let base_payment = base_calc.calculate_schedule(0.0).monthly_payment;
            self.scenarios.insert(
                "Bi-weekly Pmts".to_string(),
                base_calc.calculate_schedule(base_payment / 12.0),
            );
        }
    }

    fn generate_chart_for_tab(&mut self, ctx: &egui::Context, name: &str) {
        // Clone the schedule so we don't hold a borrow on self.scenarios
        let schedule = match self.scenarios.get(name).cloned() {
            Some(s) => s,
            None => return,
        };

        let chart_width = self.params.chart_width;
        let chart_height = self.params.chart_height;
        let caption_font_size = self.params.font_size;
        let label_font_size = (self.params.font_size as f32 * 0.75) as u32;
        let stacked = self.stacked_chart;

        // Capture cost params before any borrow
        let home_price = self.params.home_price as f64;
        let down_pct = self.params.down_payment_percent as f64;
        let property_tax_rate = self.params.property_tax_rate as f64;
        let insurance_rate = self.params.insurance_rate as f64;
        let pmi_rate = self.params.pmi_rate as f64;
        let monthly_hoa = self.params.monthly_hoa as f64;

        let equity_color = RGBColor(46, 204, 113);
        let bank_color = RGBColor(231, 76, 60);
        let cost_color = RGBColor(230, 126, 34); // orange
        let text_color = BLACK;

        // Cumulative ownership costs (tax + insurance + PMI + HOA).
        // PMI cancels once the loan balance drops to 80% LTV.
        // For the "Low Down" scenario the effective down payment is 3.5%.
        let effective_down = if name.contains("Low Down") { 3.5_f64 } else { down_pct };
        let mo_tax = home_price * property_tax_rate / 100.0 / 12.0;
        let mo_ins = home_price * insurance_rate / 100.0 / 12.0;
        let base_mo_pmi = if effective_down < 20.0 {
            home_price * (1.0 - effective_down / 100.0) * pmi_rate / 100.0 / 12.0
        } else { 0.0 };
        let mut cum_extra = 0.0;
        let extra_pts: Vec<(f64, f64)> = schedule.years.iter()
            .zip(schedule.balance.iter())
            .map(|(yr, bal)| {
                let pmi_this = if *bal / home_price > 0.80 { base_mo_pmi } else { 0.0 };
                cum_extra += mo_tax + mo_ins + pmi_this + monthly_hoa;
                (*yr, cum_extra)
            })
            .collect();

        // Mode-dependent title and y_max
        let (title, y_max) = if stacked {
            let final_hv = schedule.home_value.last().copied().unwrap_or(0.0);
            let final_equity = schedule.equity.last().copied().unwrap_or(0.0);
            let equity_pct = if final_hv > 0.0 { final_equity / final_hv * 100.0 } else { 0.0 };
            let t = format!(
                "{}\nOwnership: {:.1}% yours at year 30\nHome: ${} | Down: {}% | Rate: {}%",
                name, equity_pct,
                self.params.home_price as u64,
                self.params.down_payment_percent,
                self.params.interest_rate
            );
            (t, (final_hv * 1.1).ceil() as f64)
        } else {
            let final_equity = schedule.equity.last().copied().unwrap_or(0.0);
            let final_bank = schedule.interest_paid.last().copied().unwrap_or(0.0);
            let final_extra = extra_pts.last().map(|(_, y)| *y).unwrap_or(0.0);
            let bank_share = if final_equity + final_bank > 0.0 {
                final_bank / (final_equity + final_bank) * 100.0
            } else { 0.0 };
            let t = format!(
                "{}\nBank Share: {:.1}%\nHome: ${} | Down: {}% | Rate: {}%",
                name, bank_share,
                self.params.home_price as u64,
                self.params.down_payment_percent,
                self.params.interest_rate
            );
            (t, (final_equity.max(final_bank).max(final_extra) * 1.1).ceil() as f64)
        };

        let mut buffer = vec![0u8; (chart_width * chart_height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (chart_width, chart_height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(&root)
                .margin_left(60)
                .margin_right(40)
                .margin_top(60)
                .margin_bottom(50)
                .caption(&title, ("sans-serif", caption_font_size).into_font().with_color(&text_color))
                .x_label_area_size(50)
                .y_label_area_size(80)
                .build_cartesian_2d(0f64..30f64, 0f64..y_max).unwrap();

            chart.configure_mesh()
                .x_desc("Years")
                .y_desc("Amount ($)")
                .x_label_style(("sans-serif", label_font_size).into_font())
                .y_label_style(("sans-serif", label_font_size).into_font())
                .y_label_formatter(&|x| format!("${:.0}K", x / 1000.0))
                .draw().unwrap();

            if stacked {
                // Stacked view: equity (green, bottom) + loan balance (red, top) = home value.
                // Drawn with painter's algorithm: red fills 0→home_value first, then green
                // masks 0→equity, leaving red visible only in the equity→home_value band.
                let home_value_pts: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.home_value.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();
                let equity_pts: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.equity.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();

                // Red band (loan balance = home_value − equity)
                chart.draw_series(AreaSeries::new(
                    home_value_pts.clone(), 0.0, bank_color.mix(0.45),
                )).unwrap();
                chart.draw_series(LineSeries::new(
                    home_value_pts, bank_color.stroke_width(2),
                )).unwrap().label("Home Value (Bank + Yours)")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], bank_color.stroke_width(3)));

                // Green band (equity) masks bottom portion
                chart.draw_series(AreaSeries::new(
                    equity_pts.clone(), 0.0, equity_color.mix(0.75),
                )).unwrap();
                chart.draw_series(LineSeries::new(
                    equity_pts, equity_color.stroke_width(3),
                )).unwrap().label("Your Equity")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], equity_color.stroke_width(3)));
            } else {
                // Overlay view: equity vs cumulative interest vs ownership costs, all from zero
                let equity_points: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.equity.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();

                chart.draw_series(AreaSeries::new(
                    equity_points.clone(), 0.0, equity_color.mix(0.3),
                )).unwrap();
                chart.draw_series(LineSeries::new(
                    equity_points, equity_color.stroke_width(3),
                )).unwrap().label("Your Equity")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], equity_color.stroke_width(3)));

                let bank_points: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.interest_paid.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();

                chart.draw_series(AreaSeries::new(
                    bank_points.clone(), 0.0, bank_color.mix(0.3),
                )).unwrap();
                chart.draw_series(LineSeries::new(
                    bank_points, bank_color.stroke_width(3),
                )).unwrap().label("Bank's Profit")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], bank_color.stroke_width(3)));

                // Ownership costs (tax + insurance + PMI + HOA) — orange line, no fill
                chart.draw_series(LineSeries::new(
                    extra_pts, cost_color.stroke_width(2),
                )).unwrap().label("Tax + Ins + PMI + HOA")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cost_color.stroke_width(2)));
            }

            chart.configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK.stroke_width(1))
                .position(SeriesLabelPosition::UpperLeft)
                .draw().unwrap();
        }

        let texture = ctx.load_texture(
            name,
            egui::ColorImage::from_rgb([chart_width as usize, chart_height as usize], &buffer),
            egui::TextureOptions::LINEAR,
        );
        self.chart_textures.insert(name.to_string(), texture);
    }

    fn export_csv(&self) {
        use std::fs;

        for (name, schedule) in &self.scenarios {
            let filename = format!("loan_gui_{}.csv",
                name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "_"));

            let mut csv = String::from("Month,Year,Balance,Principal_Paid,Interest_Paid,Total_Paid,Equity,Home_Value,Bank_Share_Percent\n");

            for i in 0..schedule.months.len() {
                let month = schedule.months[i];
                let year = schedule.years[i];
                let balance = schedule.balance[i];
                let principal = schedule.principal_paid[i];
                let interest = schedule.interest_paid[i];
                let total = schedule.total_paid[i];
                let equity = schedule.equity[i];
                let home_value = schedule.home_value[i];

                let bank_share = if equity + total > 0.0 {
                    (interest / (equity + interest)) * 100.0
                } else {
                    0.0
                };

                csv.push_str(&format!("{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
                    month, year, balance, principal, interest, total, equity, home_value, bank_share));
            }

            let final_equity = schedule.equity.last().copied().unwrap_or(0.0);
            let final_bank = schedule.interest_paid.last().copied().unwrap_or(0.0);

            csv.push_str(&format!("\nSUMMARY\n"));
            csv.push_str(&format!("Home_Price,${}\n", self.params.home_price));
            csv.push_str(&format!("Down_Payment,${:.0}\n",
                self.params.home_price * self.params.down_payment_percent / 100.0));
            csv.push_str(&format!("Loan_Amount,${:.0}\n",
                self.params.home_price * (1.0 - self.params.down_payment_percent / 100.0)));
            csv.push_str(&format!("Interest_Rate,{}%\n", self.params.interest_rate));
            csv.push_str(&format!("Loan_Term,{} years\n", self.params.loan_term_years));
            csv.push_str(&format!("Monthly_Payment,${:.2}\n", schedule.monthly_payment));
            csv.push_str(&format!("Total_Equity,${:.2}\n", final_equity));
            csv.push_str(&format!("Total_Bank_Profit,${:.2}\n", final_bank));

            fs::write(&filename, csv).ok();
        }
    }
}

impl eframe::App for LoanCalcGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Recalculate scenarios and clear texture cache when params change
        if self.regenerate_chart {
            self.calculate_all_scenarios();
            self.chart_textures.clear();
            self.regenerate_chart = false;

            // If selected tab was removed (scenario unchecked), pick the first available
            if !self.scenarios.contains_key(&self.selected_tab) {
                if let Some(name) = self.scenarios.keys().sorted().next() {
                    self.selected_tab = name.clone();
                }
            }
        }

        // Left controls panel — SidePanel claims its space before CentralPanel
        egui::SidePanel::left("controls_panel")
            .resizable(true)
            .default_width(340.0)
            .min_width(260.0)
            .show(ctx, |ui| {
                ui.heading("🏠 Loan Calculator");
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {

                    ui.group(|ui| {
                        ui.heading("Loan Parameters");

                        ui.add_space(10.0);

                        // Home Price
                        ui.label("Home Price:");
                        if ui.add(egui::Slider::new(&mut self.params.home_price, 100_000.0..=1_000_000.0)
                            .step_by(5000.0)
                            .suffix(" $")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        // Down Payment %
                        ui.label("Down Payment:");
                        if ui.add(egui::Slider::new(&mut self.params.down_payment_percent, 0.0..=50.0)
                            .step_by(0.5)
                            .suffix("%")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        // Interest Rate
                        ui.label("Interest Rate:");
                        if ui.add(egui::Slider::new(&mut self.params.interest_rate, 2.0..=12.0)
                            .step_by(0.125)
                            .suffix("%")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        // Loan Term
                        ui.label("Loan Term:");
                        if ui.add(egui::Slider::new(&mut self.params.loan_term_years, 10..=30)
                            .step_by(5.0)
                            .suffix(" years")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        // Appreciation Rate
                        ui.label("Home Appreciation:");
                        if ui.add(egui::Slider::new(&mut self.params.appreciation_rate, 0.0..=10.0)
                            .step_by(0.5)
                            .suffix("%/year")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Chart Size
                        ui.label("Chart Size:");

                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label("W:");
                            if ui.add(egui::Slider::new(&mut self.params.chart_width, 800..=4000)
                                .step_by(100.0)
                                .suffix(" px")
                                .show_value(true)
                            ).changed() {
                                self.regenerate_chart = true;
                            }

                            ui.label("H:");
                            if ui.add(egui::Slider::new(&mut self.params.chart_height, 600..=3000)
                                .step_by(100.0)
                                .suffix(" px")
                                .show_value(true)
                            ).changed() {
                                self.regenerate_chart = true;
                            }
                        });

                        // Font Size
                        ui.label("Font Size:");
                        if ui.add(egui::Slider::new(&mut self.params.font_size, 12..=48)
                            .step_by(2.0)
                            .suffix(" px")
                            .show_value(true)
                        ).changed() {
                            self.regenerate_chart = true;
                        }

                        // Chart style toggle
                        ui.add_space(6.0);
                        ui.label("Chart Style:");
                        ui.horizontal(|ui| {
                            if ui.selectable_label(!self.stacked_chart, "Overlay").clicked()
                                && self.stacked_chart
                            {
                                self.stacked_chart = false;
                                self.chart_textures.clear();
                            }
                            if ui.selectable_label(self.stacked_chart, "Stacked").clicked()
                                && !self.stacked_chart
                            {
                                self.stacked_chart = true;
                                self.chart_textures.clear();
                            }
                        });
                        ui.label(egui::RichText::new(if self.stacked_chart {
                            "Equity vs loan balance = home value"
                        } else {
                            "Your equity vs interest paid to bank"
                        }).small().weak());
                    });

                    ui.add_space(10.0);

                    // Monthly costs beyond P&I
                    ui.group(|ui| {
                        ui.heading("Monthly Costs");
                        ui.add_space(5.0);

                        ui.label("Property Tax:");
                        if ui.add(egui::Slider::new(&mut self.params.property_tax_rate, 0.0..=3.0)
                            .step_by(0.1)
                            .suffix("% /yr")
                            .show_value(true)
                        ).changed() { self.chart_textures.clear(); }

                        ui.label("Home Insurance:");
                        if ui.add(egui::Slider::new(&mut self.params.insurance_rate, 0.0..=5.0)
                            .step_by(0.1)
                            .suffix("% /yr")
                            .show_value(true)
                        ).changed() { self.chart_textures.clear(); }

                        let pmi_label = if self.params.down_payment_percent < 20.0 {
                            "PMI:"
                        } else {
                            "PMI (N/A — down ≥20%):"
                        };
                        ui.label(pmi_label);
                        if ui.add(egui::Slider::new(&mut self.params.pmi_rate, 0.0..=2.0)
                            .step_by(0.05)
                            .suffix("% /yr")
                            .show_value(true)
                        ).changed() { self.chart_textures.clear(); }

                        ui.label("Monthly HOA:");
                        if ui.add(egui::Slider::new(&mut self.params.monthly_hoa, 0.0..=1000.0)
                            .step_by(25.0)
                            .prefix("$")
                            .show_value(true)
                        ).changed() { self.chart_textures.clear(); }

                        ui.label("Closing Costs:");
                        ui.add(egui::Slider::new(&mut self.params.closing_cost_percent, 0.5..=6.0)
                            .step_by(0.25)
                            .suffix("% of loan")
                            .show_value(true));
                    });

                    ui.add_space(10.0);

                    // DTI Qualifier
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading("DTI Qualifier");
                            ui.label("ℹ").on_hover_text(
                                "Debt-to-Income (DTI) ratio is the primary metric \
                                lenders use to approve or deny a mortgage.\n\n\
                                Front-end DTI (limit ~28%):\n\
                                Your total housing payment (principal, interest, \
                                taxes, insurance, PMI, HOA) divided by gross \
                                monthly income. Keeps housing costs from \
                                crowding out everything else in your budget.\n\n\
                                Back-end DTI (limit ~43%):\n\
                                All monthly debt obligations (housing + car loans \
                                + student loans + credit cards) divided by gross \
                                income. This is the number most lenders focus on \
                                for final approval.\n\n\
                                Exceeding either limit typically means denial or \
                                a higher rate. FHA loans allow up to 31% / 57% \
                                with compensating factors; conventional loans are \
                                stricter."
                            );
                        });
                        ui.add_space(5.0);

                        ui.label("Monthly Gross Income:");
                        ui.add(egui::Slider::new(&mut self.params.monthly_gross_income, 2_000.0..=30_000.0)
                            .step_by(250.0)
                            .prefix("$")
                            .show_value(true));

                        ui.label("Other Monthly Debts:");
                        ui.add(egui::Slider::new(&mut self.params.other_monthly_debts, 0.0..=5_000.0)
                            .step_by(50.0)
                            .prefix("$")
                            .show_value(true));

                        ui.add_space(6.0);

                        // Compute PITI from base case (or current params if not yet calculated)
                        let hp = self.params.home_price as f64;
                        let dp_pct = self.params.down_payment_percent as f64;
                        let loan_amt = hp * (1.0 - dp_pct / 100.0);
                        let gross = self.params.monthly_gross_income as f64;
                        let other_debts = self.params.other_monthly_debts as f64;

                        let mo_pi = self.scenarios.get("Base Case")
                            .or_else(|| self.scenarios.values().next())
                            .map(|s| s.monthly_payment)
                            .unwrap_or(0.0);
                        let mo_tax = hp * self.params.property_tax_rate as f64 / 100.0 / 12.0;
                        let mo_ins = hp * self.params.insurance_rate as f64 / 100.0 / 12.0;
                        let mo_pmi = if dp_pct < 20.0 {
                            loan_amt * self.params.pmi_rate as f64 / 100.0 / 12.0
                        } else { 0.0 };
                        let mo_hoa = self.params.monthly_hoa as f64;
                        let piti = mo_pi + mo_tax + mo_ins + mo_pmi + mo_hoa;

                        let front_dti = if gross > 0.0 { piti / gross * 100.0 } else { 0.0 };
                        let back_dti  = if gross > 0.0 { (piti + other_debts) / gross * 100.0 } else { 0.0 };

                        let dti_color = |dti: f64, limit: f64| -> egui::Color32 {
                            if dti <= limit * 0.85      { egui::Color32::from_rgb(46, 204, 113) }
                            else if dti <= limit        { egui::Color32::from_rgb(241, 196, 15) }
                            else                        { egui::Color32::from_rgb(231, 76, 60)  }
                        };

                        ui.label(egui::RichText::new("Front-end DTI (PITI ÷ income):").strong());
                        ui.label(egui::RichText::new(
                            format!("  ${:.0} ÷ ${:.0} = {:.1}%  (limit 28%)", piti, gross, front_dti)
                        ).color(dti_color(front_dti, 28.0)));

                        ui.add_space(3.0);
                        ui.label(egui::RichText::new("Back-end DTI (all debt ÷ income):").strong());
                        ui.label(egui::RichText::new(
                            format!("  ${:.0} ÷ ${:.0} = {:.1}%  (limit 43%)", piti + other_debts, gross, back_dti)
                        ).color(dti_color(back_dti, 43.0)));

                        ui.add_space(6.0);

                        // Max affordable home price: solve for H such that PITI(H) = gross * 0.28
                        // PITI(H) = H * (k_dp * amort + k_tax + k_ins + k_pmi) + mo_hoa
                        // => H = (max_piti - mo_hoa) / (k_dp * amort + k_tax + k_ins + k_pmi)
                        let r = self.params.interest_rate as f64 / 100.0 / 12.0;
                        let n = (self.params.loan_term_years * 12) as i32;
                        let amort = if r > 0.0 {
                            r * (1.0 + r).powi(n) / ((1.0 + r).powi(n) - 1.0)
                        } else { 1.0 / n as f64 };
                        let k_dp  = 1.0 - dp_pct / 100.0;
                        let k_tax = self.params.property_tax_rate as f64 / 100.0 / 12.0;
                        let k_ins = self.params.insurance_rate as f64 / 100.0 / 12.0;
                        let k_pmi = if dp_pct < 20.0 {
                            k_dp * self.params.pmi_rate as f64 / 100.0 / 12.0
                        } else { 0.0 };
                        let cost_per_dollar = k_dp * amort + k_tax + k_ins + k_pmi;

                        let max_piti_front = gross * 0.28;
                        let max_piti_back  = (gross * 0.43 - other_debts).max(0.0);
                        let max_piti = max_piti_front.min(max_piti_back);
                        let max_home = if cost_per_dollar > 0.0 {
                            (max_piti - mo_hoa) / cost_per_dollar
                        } else { 0.0 };

                        let afford_color = if hp <= max_home * 0.9 {
                            egui::Color32::from_rgb(46, 204, 113)
                        } else if hp <= max_home {
                            egui::Color32::from_rgb(241, 196, 15)
                        } else {
                            egui::Color32::from_rgb(231, 76, 60)
                        };

                        ui.label(egui::RichText::new("Max affordable home price:").strong());
                        ui.label(egui::RichText::new(
                            format!("  ${:.0}  (current: ${:.0})", max_home, hp)
                        ).color(afford_color));
                    });

                    ui.add_space(10.0);

                    // Rent vs Buy
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Rent vs Buy");
                            ui.label("ℹ").on_hover_text(
                                "Estimates when buying beats renting financially.\n\n\
                                The buyer starts with a deficit equal to cash-to-close \
                                (down payment + closing costs). Each month the buyer gains \
                                equity (principal + appreciation) but pays more than a renter \
                                (PITI vs rent). The renter's down-payment cash is assumed \
                                invested in stocks at the given return rate.\n\n\
                                Break-even = the year cumulative net advantage of buying \
                                crosses zero. Before that year, renting wins financially; \
                                after it, buying wins.\n\n\
                                This model omits maintenance costs (~1%/yr of home value) \
                                and mortgage interest tax deductions — both meaningful for \
                                a more complete picture."
                            );
                        });
                        ui.add_space(5.0);

                        ui.label("Current Monthly Rent:");
                        ui.add(egui::Slider::new(&mut self.params.monthly_rent, 500.0..=8_000.0)
                            .step_by(50.0)
                            .prefix("$")
                            .show_value(true));

                        ui.label("Rent Inflation:");
                        ui.add(egui::Slider::new(&mut self.params.rent_inflation, 0.0..=8.0)
                            .step_by(0.5)
                            .suffix("% /yr")
                            .show_value(true));

                        ui.label("Stock Return (opportunity cost):");
                        ui.add(egui::Slider::new(&mut self.params.stock_return, 0.0..=15.0)
                            .step_by(0.5)
                            .suffix("% /yr")
                            .show_value(true));

                        ui.add_space(6.0);

                        // Compute break-even using Base Case schedule
                        let hp      = self.params.home_price as f64;
                        let dp_pct  = self.params.down_payment_percent as f64;
                        let down    = hp * dp_pct / 100.0;
                        let loan    = hp * (1.0 - dp_pct / 100.0);
                        let closing = loan * self.params.closing_cost_percent as f64 / 100.0;
                        let cash_to_close = down + closing;

                        let mo_pi  = self.scenarios.get("Base Case")
                            .or_else(|| self.scenarios.values().next())
                            .map(|s| s.monthly_payment).unwrap_or(0.0);
                        let mo_tax = hp * self.params.property_tax_rate as f64 / 100.0 / 12.0;
                        let mo_ins = hp * self.params.insurance_rate as f64 / 100.0 / 12.0;
                        let mo_pmi = if dp_pct < 20.0 {
                            loan * self.params.pmi_rate as f64 / 100.0 / 12.0
                        } else { 0.0 };
                        let mo_hoa   = self.params.monthly_hoa as f64;
                        let piti     = mo_pi + mo_tax + mo_ins + mo_pmi + mo_hoa;

                        let mo_stock = self.params.stock_return as f64 / 100.0 / 12.0;
                        let mo_rent_inflate = (1.0 + self.params.rent_inflation as f64 / 100.0)
                            .powf(1.0 / 12.0) - 1.0;

                        let maybe_schedule = self.scenarios.get("Base Case")
                            .or_else(|| self.scenarios.values().next())
                            .cloned();

                        if let Some(schedule) = maybe_schedule {
                            let mut cum_net     = -cash_to_close;
                            let mut rent        = self.params.monthly_rent as f64;
                            let mut break_even  = None::<f64>;
                            let mut prev_equity = down;

                            for i in 0..schedule.months.len() {
                                let equity_gain     = schedule.equity[i] - prev_equity;
                                prev_equity         = schedule.equity[i];
                                // Opportunity cost: renter invests the cash-to-close and grows it
                                let opp_cost        = cash_to_close * mo_stock;
                                // Monthly net advantage of buying vs renting
                                cum_net += equity_gain + rent - piti - opp_cost;
                                if break_even.is_none() && cum_net >= 0.0 {
                                    break_even = Some(schedule.years[i]);
                                }
                                rent *= 1.0 + mo_rent_inflate;
                            }

                            let final_rent_paid: f64 = {
                                let mut r = self.params.monthly_rent as f64;
                                let mut total = 0.0;
                                for _ in 0..schedule.months.len() {
                                    total += r;
                                    r *= 1.0 + mo_rent_inflate;
                                }
                                total
                            };
                            let final_buying_paid = piti * schedule.months.len() as f64;
                            let final_equity = schedule.equity.last().copied().unwrap_or(0.0);

                            ui.label(format!("30yr rent paid:    ${:.0}", final_rent_paid));
                            ui.label(format!("30yr PITI paid:    ${:.0}", final_buying_paid));
                            ui.label(format!("30yr equity built: ${:.0}", final_equity));

                            ui.add_space(4.0);
                            match break_even {
                                Some(yr) => {
                                    let color = if yr <= 7.0 {
                                        egui::Color32::from_rgb(46, 204, 113)
                                    } else if yr <= 15.0 {
                                        egui::Color32::from_rgb(241, 196, 15)
                                    } else {
                                        egui::Color32::from_rgb(231, 76, 60)
                                    };
                                    ui.label(egui::RichText::new(
                                        format!("Break-even: year {:.1}", yr)
                                    ).strong().color(color));
                                }
                                None => {
                                    ui.label(egui::RichText::new(
                                        "No break-even within 30 years"
                                    ).strong().color(egui::Color32::from_rgb(231, 76, 60)));
                                }
                            }

                            let net_color = if cum_net >= 0.0 {
                                egui::Color32::from_rgb(46, 204, 113)
                            } else {
                                egui::Color32::from_rgb(231, 76, 60)
                            };
                            ui.label(egui::RichText::new(
                                format!("30yr net buying advantage: ${:.0}", cum_net)
                            ).color(net_color));
                        } else {
                            ui.label("Enable at least one scenario to see analysis.");
                        }
                    });

                    ui.add_space(10.0);

                    // Scenario toggles
                    ui.group(|ui| {
                        ui.heading("Scenarios");

                        if ui.checkbox(&mut self.show_scenarios[0], "Base Case").changed() { self.regenerate_chart = true; }
                        if ui.checkbox(&mut self.show_scenarios[1], "High Rate (8%)").changed() { self.regenerate_chart = true; }
                        if ui.checkbox(&mut self.show_scenarios[2], "Low Down (3.5%)").changed() { self.regenerate_chart = true; }
                        if ui.checkbox(&mut self.show_scenarios[3], "Extra Principal (+$200/mo)").changed() { self.regenerate_chart = true; }
                        if ui.checkbox(&mut self.show_scenarios[4], "With Disasters").changed() { self.regenerate_chart = true; }
                        if ui.checkbox(&mut self.show_scenarios[5], "Bi-weekly Payments").changed() { self.regenerate_chart = true; }
                    });

                    ui.add_space(10.0);

                    // Summary stats
                    ui.group(|ui| {
                        ui.heading("Summary");

                        let total_scenarios = self.show_scenarios.iter().filter(|&&x| x).count();
                        ui.label(format!("Showing {} scenarios", total_scenarios));

                        if !self.scenarios.is_empty() {
                            ui.add_space(5.0);

                            let base = self.scenarios.get("Base Case")
                                .or_else(|| self.scenarios.values().next());

                            if let Some(base) = base {
                                let equity = base.equity.last().copied().unwrap_or(0.0) / 1000.0;
                                let bank = base.interest_paid.last().copied().unwrap_or(0.0) / 1000.0;
                                let share = if equity + bank > 0.0 { bank / (equity + bank) * 100.0 } else { 0.0 };

                                ui.label(format!("30yr Equity: ${:.0}K", equity));
                                ui.label(format!("30yr Bank:   ${:.0}K  ({:.1}%)", bank, share));

                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("Monthly Breakdown (Base):").strong());

                                let hp = self.params.home_price as f64;
                                let dp_pct = self.params.down_payment_percent as f64;
                                let loan_amt = hp * (1.0 - dp_pct / 100.0);

                                let mo_pi  = base.monthly_payment;
                                let mo_tax = hp * self.params.property_tax_rate as f64 / 100.0 / 12.0;
                                let mo_ins = hp * self.params.insurance_rate as f64 / 100.0 / 12.0;
                                let mo_pmi = if dp_pct < 20.0 {
                                    loan_amt * self.params.pmi_rate as f64 / 100.0 / 12.0
                                } else { 0.0 };
                                let mo_hoa = self.params.monthly_hoa as f64;
                                let mo_total = mo_pi + mo_tax + mo_ins + mo_pmi + mo_hoa;

                                ui.label(format!("  P&I:   ${:.0}/mo", mo_pi));
                                ui.label(format!("  Tax:   ${:.0}/mo", mo_tax));
                                ui.label(format!("  Insur: ${:.0}/mo", mo_ins));
                                if mo_pmi > 0.0 {
                                    ui.label(format!("  PMI:   ${:.0}/mo", mo_pmi));
                                }
                                if mo_hoa > 0.0 {
                                    ui.label(format!("  HOA:   ${:.0}/mo", mo_hoa));
                                }
                                ui.label(egui::RichText::new(
                                    format!("  TOTAL: ${:.0}/mo", mo_total)
                                ).strong());

                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("Cash to Close:").strong());
                                let down_payment = hp * dp_pct / 100.0;
                                let closing_costs = loan_amt * self.params.closing_cost_percent as f64 / 100.0;
                                let cash_to_close = down_payment + closing_costs;
                                ui.label(format!("  Down:    ${:.0}", down_payment));
                                ui.label(format!("  Closing: ${:.0}", closing_costs));
                                ui.label(egui::RichText::new(
                                    format!("  TOTAL:   ${:.0}", cash_to_close)
                                ).strong());
                            }
                        }

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("📥 Export CSV").clicked() {
                                self.export_csv();
                            }
                            let tbl_label = if self.show_amort_table {
                                "📊 Hide Table"
                            } else {
                                "📊 Show Table"
                            };
                            if ui.button(tbl_label).clicked() {
                                self.show_amort_table = !self.show_amort_table;
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Open CSV buttons
                    ui.group(|ui| {
                        ui.heading("Open CSV Files");

                        for (name, _) in self.scenarios.iter().sorted_by_key(|a| a.0) {
                            let filename: String = format!("loan_gui_{}.csv",
                                name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "_"));

                            if ui.button(format!("📄 {}", name)).clicked() {
                                #[cfg(target_os = "linux")]
                                {
                                    use std::process::Command;
                                    let _ = Command::new("xdg-open").arg(&filename).spawn();
                                }
                                #[cfg(target_os = "macos")]
                                {
                                    use std::process::Command;
                                    let _ = Command::new("open").arg(&filename).spawn();
                                }
                                #[cfg(target_os = "windows")]
                                {
                                    use std::process::Command;
                                    let _ = Command::new("cmd").args(["/c", "start", "", &filename]).spawn();
                                }
                            }
                        }
                    });
                }); // end controls ScrollArea
            }); // end SidePanel

        // Amortization table panel — declared before CentralPanel so egui lays it out first
        if self.show_amort_table {
            egui::SidePanel::right("amort_table_panel")
                .resizable(true)
                .default_width(480.0)
                .min_width(380.0)
                .show(ctx, |ui| {
                    ui.heading("Amortization Schedule");
                    if let Some(sched) = self.scenarios.get(&self.selected_tab) {
                        ui.label(format!(
                            "{}  —  ${:.0}/mo P&I",
                            self.selected_tab, sched.monthly_payment
                        ));
                    }
                    ui.separator();

                    if let Some(schedule) = self.scenarios.get(&self.selected_tab).cloned() {
                        let loan_amount = self.params.home_price as f64
                            * (1.0 - self.params.down_payment_percent as f64 / 100.0);
                        let n = schedule.months.len();

                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(false)
                            .column(Column::exact(36.0))   // Mo
                            .column(Column::exact(76.0))   // Balance
                            .column(Column::exact(68.0))   // Principal
                            .column(Column::exact(68.0))   // Interest
                            .column(Column::exact(76.0))   // Cum. Int
                            .column(Column::exact(72.0))   // Equity
                            .column(Column::remainder())   // Home Val
                            .header(18.0, |mut header| {
                                header.col(|ui| { ui.strong("Mo"); });
                                header.col(|ui| { ui.strong("Balance"); });
                                header.col(|ui| { ui.strong("Princ."); });
                                header.col(|ui| { ui.strong("Int."); });
                                header.col(|ui| { ui.strong("Cum.Int"); });
                                header.col(|ui| { ui.strong("Equity"); });
                                header.col(|ui| { ui.strong("Home Val"); });
                            })
                            .body(|body| {
                                body.rows(16.0, n, |mut row| {
                                    let i = row.index();
                                    let prev_bal = if i == 0 { loan_amount } else { schedule.balance[i - 1] };
                                    let balance = schedule.balance[i];
                                    let mo_principal = prev_bal - balance;
                                    let mo_interest = schedule.interest_paid[i]
                                        - if i > 0 { schedule.interest_paid[i - 1] } else { 0.0 };

                                    row.col(|ui| { ui.label(format!("{}", schedule.months[i])); });
                                    row.col(|ui| { ui.label(format!("${:.0}", balance)); });
                                    row.col(|ui| { ui.label(format!("${:.0}", mo_principal)); });
                                    row.col(|ui| { ui.label(format!("${:.0}", mo_interest)); });
                                    row.col(|ui| { ui.label(format!("${:.0}", schedule.interest_paid[i])); });
                                    row.col(|ui| { ui.label(format!("${:.0}", schedule.equity[i])); });
                                    row.col(|ui| { ui.label(format!("${:.0}", schedule.home_value[i])); });
                                });
                            });
                    } else {
                        ui.label("No data — enable a scenario above.");
                    }
                });
        }

        // Chart panel — CentralPanel fills all remaining space, giving ScrollArea full height
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar — one tab per active scenario
            ui.horizontal(|ui: &mut egui::Ui| {
                let tab_names: Vec<String> = self.scenarios.keys().sorted().cloned().collect();
                for name in tab_names {
                    if ui.selectable_label(self.selected_tab == name, &name).clicked()
                        && self.selected_tab != name
                    {
                        self.selected_tab = name;
                    }
                }
            });
            ui.separator();

            // Generate chart for the selected tab if not cached yet
            if !self.chart_textures.contains_key(&self.selected_tab) {
                let tab = self.selected_tab.clone();
                self.generate_chart_for_tab(ctx, &tab);
            }

            if let Some(texture) = self.chart_textures.get(&self.selected_tab) {
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.image((texture.id(), texture.size_vec2()));
                    });
            } else {
                ui.label("Generating chart...");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 720.0])
            .with_min_inner_size([900.0, 560.0])
            .with_title("Loan Amortization Calculator - Interactive"),
        ..Default::default()
    };

    eframe::run_native(
        "Loan Calculator GUI",
        options,
        Box::new(|cc| Ok(Box::new(LoanCalcGui::new(cc)))),
    )
}
