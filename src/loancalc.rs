//! Shared loan calculation types used by both the CLI and GUI binaries.

/// Amortization schedule data
#[derive(Debug, Clone)]
pub struct AmortizationSchedule {
    pub years: Vec<f64>,
    pub months: Vec<u32>,
    pub balance: Vec<f64>,
    pub equity: Vec<f64>,
    pub principal_paid: Vec<f64>,
    pub interest_paid: Vec<f64>,
    pub total_paid: Vec<f64>,
    pub home_value: Vec<f64>,
    pub monthly_payment: f64,
}

/// Loan calculator
pub struct LoanCalculator {
    pub home_price: f64,
    down_payment: f64,
    loan_amount: f64,
    interest_rate: f64,
    pub loan_term_years: u32,
    appreciation_rate: f64,
}

impl LoanCalculator {
    pub fn new(
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
    pub fn calculate_schedule(&self, extra_monthly: f64) -> AmortizationSchedule {
        let monthly_rate = self.interest_rate / 12.0;
        let num_payments = self.loan_term_years as usize * 12;

        // Calculate monthly payment
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

            // Annual appreciation
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
