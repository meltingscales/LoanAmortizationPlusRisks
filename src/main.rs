//! Home Loan Amortization Calculator with Risk Analysis - Rust Edition
//! Shows home equity vs bank profit over time with disaster/cushion scenarios.
//!
//! Run: cargo run --release

use itertools::Itertools;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Configuration loaded from config.toml
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    loan: LoanConfig,
    scenarios: ScenarioConfig,
    scenario_params: ScenarioParams,
    display: DisplayConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoanConfig {
    home_price: f64,
    down_payment_percent: f64,
    interest_rate: f64,
    loan_term_years: u32,
    start_year: u32,
    appreciation_rate: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScenarioConfig {
    #[serde(default = "default_true")]
    show_base: bool,
    #[serde(default = "default_true")]
    show_high_rate: bool,
    #[serde(default = "default_true")]
    show_low_down: bool,
    #[serde(default = "default_true")]
    show_extra_principal: bool,
    #[serde(default = "default_true")]
    show_disasters: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScenarioParams {
    high_rate_percent: f64,
    low_down_percent: f64,
    extra_principal_monthly: f64,
    disaster_appreciation_rate: f64,
    disaster_cost_total: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct DisplayConfig {
    chart_width: u32,
    chart_height: u32,
    output_file: String,
    #[serde(default = "default_true")]
    show_grid: bool,
}

fn default_true() -> bool { true }

/// Amortization schedule data
#[derive(Debug, Clone)]
struct AmortizationSchedule {
    years: Vec<f64>,
    equity: Vec<f64>,
    interest_paid: Vec<f64>,
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

    /// Calculate amortization schedule
    fn calculate_schedule(&self, extra_monthly: f64) -> AmortizationSchedule {
        let monthly_rate = self.interest_rate / 12.0;
        let num_payments = self.loan_term_years as usize * 12;

        // Calculate monthly payment
        let monthly_payment = if monthly_rate > 0.0 {
            self.loan_amount * (monthly_rate * (1.0 + monthly_rate).powi(num_payments as i32))
                / ((1.0 + monthly_rate).powi(num_payments as i32) - 1.0)
        } else {
            self.loan_amount / num_payments as f64
        } + extra_monthly;

        let mut balance = self.loan_amount;
        let mut home_value = self.home_price;
        let mut cumulative_principal = self.down_payment;
        let mut cumulative_interest = 0.0;

        let mut years = Vec::with_capacity(num_payments);
        let mut equity = Vec::with_capacity(num_payments);
        let mut interest_paid = Vec::with_capacity(num_payments);

        for month in 1..=num_payments {
            let interest_payment = balance * monthly_rate;
            let principal_payment = (monthly_payment - interest_payment).min(balance);

            balance -= principal_payment;
            cumulative_principal += principal_payment;
            cumulative_interest += interest_payment;

            // Annual appreciation
            if month % 12 == 0 {
                home_value *= 1.0 + self.appreciation_rate;
            }

            let current_equity = cumulative_principal + (home_value - self.home_price);

            years.push(month as f64 / 12.0);
            equity.push(current_equity);
            interest_paid.push(cumulative_interest);
        }

        AmortizationSchedule {
            years,
            equity,
            interest_paid,
        }
    }
}

/// Calculate all scenarios based on config
fn calculate_scenarios(config: &Config) -> HashMap<String, AmortizationSchedule> {
    let mut scenarios = HashMap::new();

    let cfg = &config.loan;
    let base_calc = LoanCalculator::new(
        cfg.home_price,
        cfg.down_payment_percent,
        cfg.interest_rate,
        cfg.loan_term_years,
        cfg.appreciation_rate,
    );

    // Base case
    if config.scenarios.show_base {
        scenarios.insert(
            "Base Case".to_string(),
            base_calc.calculate_schedule(0.0),
        );
    }

    // High interest rate
    if config.scenarios.show_high_rate {
        let calc = LoanCalculator::new(
            cfg.home_price,
            cfg.down_payment_percent,
            config.scenario_params.high_rate_percent,
            cfg.loan_term_years,
            cfg.appreciation_rate,
        );
        scenarios.insert(
            format!("High Rate ({}%)", config.scenario_params.high_rate_percent),
            calc.calculate_schedule(0.0),
        );
    }

    // Low down payment
    if config.scenarios.show_low_down {
        let calc = LoanCalculator::new(
            cfg.home_price,
            config.scenario_params.low_down_percent,
            cfg.interest_rate,
            cfg.loan_term_years,
            cfg.appreciation_rate,
        );
        scenarios.insert(
            format!("Low Down ({}%)", config.scenario_params.low_down_percent),
            calc.calculate_schedule(0.0),
        );
    }

    // Extra principal payments
    if config.scenarios.show_extra_principal {
        scenarios.insert(
            format!("Extra Principal (+${}/mo)", config.scenario_params.extra_principal_monthly),
            base_calc.calculate_schedule(config.scenario_params.extra_principal_monthly),
        );
    }

    // Disasters
    if config.scenarios.show_disasters {
        let calc = LoanCalculator::new(
            cfg.home_price,
            cfg.down_payment_percent,
            cfg.interest_rate,
            cfg.loan_term_years,
            config.scenario_params.disaster_appreciation_rate,
        );
        let mut schedule = calc.calculate_schedule(0.0);
        // Apply disaster costs
        for e in schedule.equity.iter_mut() {
            *e -= config.scenario_params.disaster_cost_total;
        }
        scenarios.insert("With Disasters".to_string(), schedule);
    }

    scenarios
}

/// Generate comparison chart using plotters
fn generate_chart(config: &Config, scenarios: &HashMap<String, AmortizationSchedule>) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = &config.display.output_file;
    let root = BitMapBackend::new(output_path, (config.display.chart_width, config.display.chart_height)).into_drawing_area();
    root.fill(&WHITE)?;

    // Colors
    let equity_color = RGBColor(46, 204, 113);  // Green
    let bank_color = RGBColor(231, 76, 60);     // Red
    let text_color = BLACK;

    let n_scenarios = scenarios.len();
    let n_cols = 3.min(n_scenarios);
    let n_rows = (n_scenarios + n_cols - 1) / n_cols + 1; // +1 for summary

    let areas = root.split_evenly((n_rows, n_cols));

    // Plot each scenario
    let mut scenario_data: Vec<_> = scenarios.iter().collect();
    scenario_data.sort_by_key(|(k, _)| *k);

    for (idx, (name, schedule)) in scenario_data.iter().enumerate() {
        if idx >= areas.len() - 1 { break; } // Reserve last row for summary

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

        let mut chart = ChartBuilder::on(area)
            .margin_left(60)
            .margin_right(20)
            .margin_top(40)
            .margin_bottom(40)
            .caption(
                &format!("{}\nBank Share: {:.1}%", name, bank_share),
                ("sans-serif", 20).into_font().with_color(&text_color),
            )
            .x_label_area_size(40)
            .y_label_area_size(70)
            .build_cartesian_2d(0f64..30f64, 0f64..y_max)?;

        chart.configure_mesh()
            .x_desc("Years")
            .y_desc("Amount ($)")
            .x_label_style(("sans-serif", 12).into_font())
            .y_label_style(("sans-serif", 12).into_font())
            .y_label_formatter(&|x| format!("${:.0}K", x / 1000.0))
            .draw()?;

        // Draw equity area
        let equity_points: Vec<(f64, f64)> = schedule.years.iter()
            .zip(schedule.equity.iter())
            .map(|(x, y)| (*x, *y))
            .collect();

        chart.draw_series(AreaSeries::new(
            equity_points.clone(),
            0.0,
            equity_color.mix(0.3),
        ))?;

        chart.draw_series(LineSeries::new(
            equity_points,
            equity_color.stroke_width(2),
        ))?.label("Your Equity")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], equity_color.stroke_width(2)));

        // Draw bank profit area
        let bank_points: Vec<(f64, f64)> = schedule.years.iter()
            .zip(schedule.interest_paid.iter())
            .map(|(x, y)| (*x, *y))
            .collect();

        chart.draw_series(AreaSeries::new(
            bank_points.clone(),
            0.0,
            bank_color.mix(0.3),
        ))?;

        chart.draw_series(LineSeries::new(
            bank_points,
            bank_color.stroke_width(2),
        ))?.label("Bank's Profit")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], bank_color.stroke_width(2)));

        chart.configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK.stroke_width(1))
            .position(SeriesLabelPosition::UpperLeft)
            .draw()?;
    }

    // Note: Last area left blank - summary printed to console instead

    root.present()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_content)?;

    println!("🏠 Loan Amortization Calculator - Rust Edition");
    println!("═══════════════════════════════════════════════");
    println!("Home Price: ${}", config.loan.home_price);
    println!("Down Payment: {}%", config.loan.down_payment_percent);
    println!("Interest Rate: {}%", config.loan.interest_rate);
    println!("Term: {} years", config.loan.loan_term_years);
    println!();

    // Calculate scenarios
    let scenarios = calculate_scenarios(&config);

    println!("Calculating {} scenarios...", scenarios.len());
    for name in scenarios.keys().sorted() {
        println!("  - {}", name);
    }

    // Generate chart
    println!("\nGenerating chart...");
    generate_chart(&config, &scenarios)?;

    // Print summary
    println!("\n═══════════════════════════════════════════════");
    println!("SCENARIO COMPARISON (30-Year Projection)");
    println!("═══════════════════════════════════════════════");

    let mut sorted_data: Vec<_> = scenarios.iter().collect();
    sorted_data.sort_by_key(|(k, _)| *k);

    for (name, schedule) in &sorted_data {
        let equity = schedule.equity.last().copied().unwrap_or(0.0);
        let bank = schedule.interest_paid.last().copied().unwrap_or(0.0);
        let share = if equity + bank > 0.0 { bank / (equity + bank) * 100.0 } else { 0.0 };
        println!("{}: Equity=${:.0}K Bank=${:.0}K ({:.1}%)",
            name, equity / 1000.0, bank / 1000.0, share);
    }

    println!("\nHome: ${} Down: {}% Rate: {}% Term: {}yr",
        config.loan.home_price,
        config.loan.down_payment_percent,
        config.loan.interest_rate,
        config.loan.loan_term_years
    );

    println!("\n✅ Done!");
    println!("Chart saved to: {}", config.display.output_file);

    Ok(())
}
