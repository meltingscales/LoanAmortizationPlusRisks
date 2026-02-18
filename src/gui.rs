//! Interactive GUI for Loan Amortization Calculator
//! Run with: cargo run --bin loan-calc-gui --features gui

#![cfg(feature = "gui")]

use eframe::egui;
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
}

impl Default for LoanParams {
    fn default() -> Self {
        Self {
            home_price: 450_000.0,
            down_payment_percent: 20.0,
            interest_rate: 6.5,
            loan_term_years: 30,
            appreciation_rate: 3.0,
            chart_width: 2400,
            chart_height: 1600,
            font_size: 28,
        }
    }
}

/// The main GUI application
struct LoanCalcGui {
    params: LoanParams,
    chart_texture: Option<egui::TextureHandle>,
    scenarios: HashMap<String, AmortizationSchedule>,
    show_scenarios: [bool; 5],
    scenario_names: Vec<String>,
    regenerate_chart: bool,
}

impl LoanCalcGui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            params: LoanParams::default(),
            chart_texture: None,
            scenarios: HashMap::new(),
            show_scenarios: [true, true, true, true, true],
            scenario_names: vec![
                "Base Case".to_string(),
                "High Rate".to_string(),
                "Low Down".to_string(),
                "Extra Principal".to_string(),
                "With Disasters".to_string(),
            ],
            regenerate_chart: true,
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
    }

    fn generate_chart(&mut self, ctx: &egui::Context) {
        if self.scenarios.is_empty() {
            return;
        }

        let chart_width = self.params.chart_width;
        let chart_height = self.params.chart_height;
        let caption_font_size = self.params.font_size;
        let label_font_size = (self.params.font_size as f32 * 0.75) as u32;

        // Create in-memory chart
        let mut buffer = vec![0u8; (chart_width * chart_height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (chart_width, chart_height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let equity_color = RGBColor(46, 204, 113);
            let bank_color = RGBColor(231, 76, 60);
            let text_color = BLACK;

            let n_scenarios = self.scenarios.len();
            let n_cols = 3.min(n_scenarios);
            let n_rows = (n_scenarios + n_cols - 1) / n_cols;

            let areas = root.split_evenly((n_rows, n_cols));

            let mut scenario_data: Vec<_> = self.scenarios.iter().collect();
            scenario_data.sort_by_key(|(k, _)| *k);

            for (idx, (name, schedule)) in scenario_data.iter().enumerate() {
                if idx >= areas.len() { break; }

                let area = &areas[idx];
                let final_equity = schedule.equity.last().copied().unwrap_or(0.0);
                let final_bank = schedule.interest_paid.last().copied().unwrap_or(0.0);
                let bank_share = if final_equity + final_bank > 0.0 {
                    final_bank / (final_equity + final_bank) * 100.0
                } else {
                    0.0
                };

                let max_value = final_equity.max(final_bank);
                let y_max = (max_value * 1.1).ceil() as f64;

                let title = format!(
                    "{}\nBank Share: {:.1}%\nHome: ${} | Down: {}% | Rate: {}%",
                    name, bank_share,
                    self.params.home_price as u64,
                    self.params.down_payment_percent,
                    self.params.interest_rate
                );

                let mut chart = ChartBuilder::on(area)
                    .margin_left(60)
                    .margin_right(20)
                    .margin_top(50)
                    .margin_bottom(40)
                    .caption(&title, ("sans-serif", caption_font_size).into_font().with_color(&text_color))
                    .x_label_area_size(40)
                    .y_label_area_size(70)
                    .build_cartesian_2d(0f64..30f64, 0f64..y_max).unwrap();

                chart.configure_mesh()
                    .x_desc("Years")
                    .y_desc("Amount ($)")
                    .x_label_style(("sans-serif", label_font_size).into_font())
                    .y_label_style(("sans-serif", label_font_size).into_font())
                    .y_label_formatter(&|x| format!("${:.0}K", x / 1000.0))
                    .draw().unwrap();

                // Draw equity
                let equity_points: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.equity.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();

                chart.draw_series(AreaSeries::new(
                    equity_points.clone(),
                    0.0,
                    equity_color.mix(0.3),
                )).unwrap();

                chart.draw_series(LineSeries::new(
                    equity_points,
                    equity_color.stroke_width(2),
                )).unwrap().label("Your Equity")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], equity_color.stroke_width(2)));

                // Draw bank profit
                let bank_points: Vec<(f64, f64)> = schedule.years.iter()
                    .zip(schedule.interest_paid.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect();

                chart.draw_series(AreaSeries::new(
                    bank_points.clone(),
                    0.0,
                    bank_color.mix(0.3),
                )).unwrap();

                chart.draw_series(LineSeries::new(
                    bank_points,
                    bank_color.stroke_width(2),
                )).unwrap().label("Bank's Profit")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], bank_color.stroke_width(2)));

                chart.configure_series_labels()
                    .background_style(WHITE.mix(0.8))
                    .border_style(BLACK.stroke_width(1))
                    .position(SeriesLabelPosition::UpperLeft)
                    .draw().unwrap();
            }
        }

        // Convert to image texture
        self.chart_texture = Some(ctx.load_texture(
            "chart",
            egui::ColorImage::from_rgb([chart_width as usize, chart_height as usize], &buffer),
            egui::TextureOptions::LINEAR,
        ));

        self.regenerate_chart = false;
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
                    });

                    ui.add_space(10.0);

                    // Summary stats
                    ui.group(|ui| {
                        ui.heading("Summary");

                        let total_scenarios = self.show_scenarios.iter().filter(|&&x| x).count();
                        ui.label(format!("Showing {} scenarios", total_scenarios));

                        if !self.scenarios.is_empty() {
                            ui.add_space(5.0);

                            if let Some(base) = self.scenarios.get("Base Case") {
                                let equity = base.equity.last().copied().unwrap_or(0.0) / 1000.0;
                                let bank = base.interest_paid.last().copied().unwrap_or(0.0) / 1000.0;
                                let share = if equity + bank > 0.0 { bank / (equity + bank) * 100.0 } else { 0.0 };

                                ui.label(format!("Equity: ${:.0}K", equity));
                                ui.label(format!("Bank: ${:.0}K", bank));
                                ui.label(format!("Bank Share: {:.1}%", share));
                            }
                        }

                        ui.add_space(10.0);

                        if ui.button("📥 Export CSV").clicked() {
                            self.export_csv();
                        }
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

        // Chart panel — CentralPanel fills all remaining space, giving ScrollArea full height
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Equity vs Bank Profit");

            if self.regenerate_chart || self.chart_texture.is_none() {
                self.calculate_all_scenarios();
                self.generate_chart(ctx);
            }

            if let Some(texture) = &self.chart_texture {
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
