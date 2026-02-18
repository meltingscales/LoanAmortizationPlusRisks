#!/usr/bin/env python3
"""
Home Loan Amortization Calculator - Side-by-Side View
Shows all scenarios at once without needing interactive widgets.
"""

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.patches import Patch
from matplotlib.colors import LinearSegmentedColormap
import matplotlib.gridspec as gridspec


class LoanAmortizationCalculator:
    """Calculator for mortgage amortization with risk scenarios."""

    # Florida-specific disaster costs (approximate)
    DISASTER_COSTS = {
        'hurricane': 15000,
        'flood': 25000,
        'sinkhole': 100000,
        'market_crash': 0.20,
    }

    CUSHIONS = {
        'emergency_fund': 25000,
        'insurance_premium': 500,
        'extra_principal': 200,
    }

    def __init__(self, home_price, down_payment, interest_rate, loan_term_years,
                 start_year=2026, appreciation_rate=0.03):
        self.home_price = home_price
        self.down_payment = down_payment if down_payment >= 1 else down_payment * home_price
        self.loan_amount = home_price - self.down_payment
        self.interest_rate = interest_rate
        self.loan_term_years = loan_term_years
        self.start_year = start_year
        self.appreciation_rate = appreciation_rate
        self.schedule = None

    def calculate_amortization(self, extra_monthly=0):
        """Calculate the amortization schedule."""
        monthly_rate = self.interest_rate / 12
        num_payments = self.loan_term_years * 12

        if monthly_rate > 0:
            base_payment = self.loan_amount * (monthly_rate * (1 + monthly_rate)**num_payments) / \
                           ((1 + monthly_rate)**num_payments - 1)
        else:
            base_payment = self.loan_amount / num_payments

        monthly_payment = base_payment + extra_monthly

        months = []
        balances = []
        principal_paid = []
        interest_paid = []
        home_values = []
        equities = []

        balance = self.loan_amount
        home_value = self.home_price
        cumulative_principal = self.down_payment
        cumulative_interest = 0

        for month in range(1, num_payments + 1):
            interest_payment = balance * monthly_rate
            principal_payment = min(monthly_payment - interest_payment, balance)

            balance -= principal_payment
            cumulative_principal += principal_payment
            cumulative_interest += interest_payment

            if month % 12 == 0:
                home_value *= (1 + self.appreciation_rate)

            equity = cumulative_principal + (home_value - self.home_price)

            months.append(month)
            balances.append(max(0, balance))
            principal_paid.append(cumulative_principal)
            interest_paid.append(cumulative_interest)
            home_values.append(home_value)
            equities.append(equity)

        self.schedule = pd.DataFrame({
            'month': months,
            'balance': balances,
            'total_principal_paid': principal_paid,
            'total_interest_paid': interest_paid,
            'home_value': home_values,
            'equity': equities,
            'years': [m / 12 for m in months]
        })

        return self.schedule


def create_scenario_comparison(calculator):
    """Create a side-by-side comparison of all scenarios."""
    scenarios = {}

    # Base scenario
    calculator.calculate_amortization()
    scenarios['Base Case'] = calculator.schedule.copy()

    # High interest rate
    calc_high = LoanAmortizationCalculator(
        calculator.home_price, calculator.down_payment,
        0.08, calculator.loan_term_years,
        calculator.start_year, calculator.appreciation_rate
    )
    calc_high.calculate_amortization()
    scenarios['High Rate (8%)'] = calc_high.schedule

    # With disasters
    calc_disaster = LoanAmortizationCalculator(
        calculator.home_price, calculator.down_payment,
        calculator.interest_rate, calculator.loan_term_years,
        calculator.start_year, 0.02  # Lower appreciation
    )
    calc_disaster.calculate_amortization()
    # Apply disaster costs
    disaster_schedule = calc_disaster.schedule.copy()
    disaster_schedule['equity'] -= 40000  # Hurricane + Flood costs
    scenarios['With Disasters'] = disaster_schedule

    # Extra principal payments
    calc_extra = LoanAmortizationCalculator(
        calculator.home_price, calculator.down_payment,
        calculator.interest_rate, calculator.loan_term_years,
        calculator.start_year, calculator.appreciation_rate
    )
    calc_extra.calculate_amortization(extra_monthly=200)
    scenarios['Extra Principal (+$200/mo)'] = calc_extra.schedule

    # Create figure
    fig = plt.figure(figsize=(18, 10))
    gs = gridspec.GridSpec(2, 3, figure=fig, hspace=0.3, wspace=0.25)

    fig.suptitle(f'Home Loan Amortization: Your Equity vs Bank Profit\n'
                 f'Florida {calculator.start_year} • ${calculator.home_price:,.0f} home',
                 fontsize=16, fontweight='bold')

    positions = [(0, 0), (0, 1), (0, 2), (1, 0)]
    colors = {
        'equity': '#2ecc71',
        'bank_profit': '#e74c3c',
    }

    for idx, (name, schedule) in enumerate(scenarios.items()):
        row, col = positions[idx]
        ax = fig.add_subplot(gs[row, col])

        years = schedule['years']
        equity = schedule['equity']
        bank_profit = schedule['total_interest_paid']

        ax.plot(years, equity, color=colors['equity'], linewidth=2.5, label='Your Equity')
        ax.plot(years, bank_profit, color=colors['bank_profit'], linewidth=2.5, label="Bank's Profit")
        ax.fill_between(years, 0, equity, alpha=0.3, color=colors['equity'])
        ax.fill_between(years, 0, bank_profit, alpha=0.3, color=colors['bank_profit'])

        final_equity = equity.iloc[-1]
        final_bank = bank_profit.iloc[-1]
        bank_share = final_bank / (final_equity + final_bank) * 100

        ax.set_title(f'{name}\nBank Share: {bank_share:.1f}%',
                     fontsize=11, fontweight='bold')
        ax.set_xlabel('Years')
        ax.set_ylabel('Amount ($)')
        ax.legend(loc='upper left', fontsize=9)
        ax.grid(True, alpha=0.3)
        ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'${x/1000:.0f}K'))
        ax.set_xlim(0, 30)

    # Summary table in bottom right
    ax_summary = fig.add_subplot(gs[1, 1:])
    ax_summary.axis('off')

    summary_text = "SCENARIO COMPARISON (30-Year Projection)\n"
    summary_text +="═" * 80 + "\n"
    summary_text += f"{'Scenario':<28} {'Your Equity':>15} {'Bank Profit':>15} {'Bank Share':>12}\n"
    summary_text += "─" * 80 + "\n"

    for name, schedule in scenarios.items():
        equity = schedule['equity'].iloc[-1]
        bank = schedule['total_interest_paid'].iloc[-1]
        share = bank / (equity + bank) * 100
        summary_text += f"{name:<28} ${equity:>13,.0f}  ${bank:>13,.0f}  {share:>10.1f}%\n"

    summary_text += "═" * 80 + "\n"
    summary_text += f"\nHome Price: ${calculator.home_price:,.0f}  |  "
    summary_text += f"Down Payment: ${calculator.down_payment:,.0f}  |  "
    summary_text += f"Loan Amount: ${calculator.loan_amount:,.0f}\n"
    summary_text += f"Interest Rate: {calculator.interest_rate*100:.1f}%  |  "
    summary_text += f"Term: {calculator.loan_term_years} years  |  "
    summary_text += f"Appreciation: {calculator.appreciation_rate*100:.1f}%/yr"

    ax_summary.text(0.05, 0.95, summary_text, transform=ax_summary.transAxes,
                    fontsize=10, verticalalignment='top', family='monospace',
                    bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    return fig


def main():
    """Main entry point."""
    calculator = LoanAmortizationCalculator(
        home_price=450000,
        down_payment=0.20,
        interest_rate=0.065,
        loan_term_years=30,
        start_year=2026,
        appreciation_rate=0.03
    )

    fig = create_scenario_comparison(calculator)
    plt.show()


if __name__ == '__main__':
    main()
