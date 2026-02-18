#!/usr/bin/env python3
"""
Home Loan Amortization Calculator with Risk Analysis
Shows home equity vs bank profit over time with disaster/cushion scenarios.
"""

import numpy as np
import pandas as pd

# Use interactive backend for widget support
import matplotlib
try:
    matplotlib.use('Qt5Agg')
except ImportError:
    try:
        matplotlib.use('TkAgg')
    except ImportError:
        matplotlib.use('module://ipympl.backend_nbagg')

import matplotlib.pyplot as plt
from matplotlib.widgets import CheckButtons, Slider
from matplotlib.patches import Patch
from matplotlib.colors import LinearSegmentedColormap


class LoanAmortizationCalculator:
    """Calculator for mortgage amortization with risk scenarios."""

    # Florida-specific disaster costs (approximate)
    DISASTER_COSTS = {
        'hurricane': 15000,      # Average damage per event
        'flood': 25000,          # Average flood damage
        'sinkhole': 100000,      # Major sinkhole repair
        'market_crash': 0.20,    # 20% home value drop
    }

    # Cushion options
    CUSHIONS = {
        'emergency_fund': 25000,     # 6 months expenses
        'insurance_premium': 500,     # Annual hurricane insurance
        'extra_principal': 200,       # Extra monthly payment
    }

    def __init__(self, home_price, down_payment, interest_rate, loan_term_years,
                 start_year=2026, appreciation_rate=0.03):
        """
        Initialize loan parameters.

        Args:
            home_price: Purchase price of the home
            down_payment: Down payment amount (or percentage if < 1)
            interest_rate: Annual interest rate (e.g., 0.065 for 6.5%)
            loan_term_years: Loan term in years (typically 30)
            start_year: Year of purchase
            appreciation_rate: Annual home appreciation rate
        """
        self.home_price = home_price
        self.down_payment = down_payment if down_payment >= 1 else down_payment * home_price
        self.loan_amount = home_price - self.down_payment
        self.interest_rate = interest_rate
        self.loan_term_years = loan_term_years
        self.start_year = start_year
        self.appreciation_rate = appreciation_rate

        self.schedule = None
        self.scenarios = {}
        self.active_disasters = set()
        self.active_cushions = set()

    def calculate_amortization(self):
        """Calculate the base amortization schedule."""
        monthly_rate = self.interest_rate / 12
        num_payments = self.loan_term_years * 12

        # Monthly payment (P&I only)
        if monthly_rate > 0:
            monthly_payment = self.loan_amount * (monthly_rate * (1 + monthly_rate)**num_payments) / \
                              ((1 + monthly_rate)**num_payments - 1)
        else:
            monthly_payment = self.loan_amount / num_payments

        # Build schedule
        months = []
        payment_numbers = []
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
            principal_payment = monthly_payment - interest_payment

            balance -= principal_payment
            cumulative_principal += principal_payment
            cumulative_interest += interest_payment

            # Update home value (annual appreciation applied monthly)
            if month % 12 == 0:
                home_value *= (1 + self.appreciation_rate)

            equity = cumulative_principal + (home_value - self.home_price)

            months.append(month)
            payment_numbers.append(month)
            balances.append(max(0, balance))
            principal_paid.append(cumulative_principal)
            interest_paid.append(cumulative_interest)
            home_values.append(home_value)
            equities.append(equity)

        self.schedule = pd.DataFrame({
            'month': months,
            'payment_number': payment_numbers,
            'balance': balances,
            'total_principal_paid': principal_paid,
            'total_interest_paid': interest_paid,
            'home_value': home_values,
            'equity': equities,
            'years': [m / 12 for m in months]
        })

        return self.schedule

    def apply_disaster(self, disaster_type, year=None):
        """
        Apply a disaster scenario at a specific year.

        Args:
            disaster_type: Type of disaster ('hurricane', 'flood', 'sinkhole', 'market_crash')
            year: Year when disaster occurs (relative to start)
        """
        if disaster_type not in self.DISASTER_COSTS:
            raise ValueError(f"Unknown disaster type: {disaster_type}")

        schedule = self.schedule.copy()
        cost = self.DISASTER_COSTS[disaster_type]

        if disaster_type == 'market_crash':
            # Percentage drop in home value
            crash_month = int((year if year else 10) * 12)
            mask = schedule['month'] >= crash_month
            schedule.loc[mask, 'home_value'] *= (1 - cost)
            schedule.loc[mask, 'equity'] = schedule.loc[mask, 'total_principal_paid'] + \
                                           (schedule.loc[mask, 'home_value'] - self.home_price)
        else:
            # Fixed cost reducing equity
            disaster_month = int((year if year else 10) * 12)
            mask = schedule['month'] >= disaster_month
            schedule.loc[mask, 'equity'] -= cost

        scenario_name = f"{disaster_type}_{year if year else 'year10'}"
        self.scenarios[scenario_name] = schedule
        return schedule

    def apply_cushion(self, cushion_type):
        """
        Apply a cushion scenario.

        Args:
            cushion_type: Type of cushion ('emergency_fund', 'insurance_premium', 'extra_principal')
        """
        if cushion_type not in self.CUSHIONS:
            raise ValueError(f"Unknown cushion type: {cushion_type}")

        schedule = self.schedule.copy()

        if cushion_type == 'extra_principal':
            # Recalculate with extra principal payment
            monthly_rate = self.interest_rate / 12
            extra = self.CUSHIONS[cushion_type]
            num_payments = self.loan_term_years * 12

            balance = self.loan_amount
            cumulative_principal = self.down_payment
            cumulative_interest = 0
            home_value = self.home_price

            months = []
            balances = []
            principal_paid = []
            interest_paid = []
            home_values = []
            equities = []

            for month in range(1, num_payments + 1):
                base_payment = self.schedule.loc[month - 1, 'total_principal_paid'] - \
                               (self.schedule.loc[month - 1, 'total_principal_paid'] -
                                (0 if month == 1 else self.schedule.loc[month - 2, 'total_principal_paid']))

                # Get original monthly payment
                if monthly_rate > 0:
                    regular_payment = self.loan_amount * (monthly_rate * (1 + monthly_rate)**num_payments) / \
                                      ((1 + monthly_rate)**num_payments - 1)
                else:
                    regular_payment = self.loan_amount / num_payments

                interest_payment = balance * monthly_rate
                principal_payment = (regular_payment - interest_payment) + extra

                if balance > 0:
                    balance -= min(principal_payment, balance)
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

            schedule = pd.DataFrame({
                'month': months,
                'payment_number': months,
                'balance': balances,
                'total_principal_paid': principal_paid,
                'total_interest_paid': interest_paid,
                'home_value': home_values,
                'equity': equities,
                'years': [m / 12 for m in months]
            })
        else:
            # Add cushion value to equity (savings buffer)
            cushion_value = self.CUSHIONS[cushion_type]
            if cushion_type == 'insurance_premium':
                # Cumulative cost of insurance over time
                schedule['equity'] -= schedule['years'] * cushion_value
            else:
                schedule['equity'] += cushion_value

        self.scenarios[cushion_type] = schedule
        return schedule

    def get_combined_schedule(self):
        """Get schedule with all active scenarios applied."""
        if not self.active_disasters and not self.active_cushions:
            return self.schedule

        result = self.schedule.copy()

        # Apply disasters (reduce equity)
        for disaster in self.active_disasters:
            if disaster == 'hurricane':
                result['equity'] -= self.DISASTER_COSTS['hurricane']
            elif disaster == 'flood':
                result['equity'] -= self.DISASTER_COSTS['flood']
            elif disaster == 'sinkhole':
                result['equity'] -= self.DISASTER_COSTS['sinkhole']
            elif disaster == 'market_crash':
                # Apply 20% drop after year 5
                crash_month = int(5 * 12)
                mask = result['month'] >= crash_month
                result.loc[mask, 'home_value'] *= 0.80
                result.loc[mask, 'equity'] = result.loc[mask, 'total_principal_paid'] + \
                                             (result.loc[mask, 'home_value'] - self.home_price)

        # Apply cushions
        for cushion in self.active_cushions:
            if cushion == 'emergency_fund':
                result['equity'] += self.CUSHIONS['emergency_fund']
            elif cushion == 'insurance_premium':
                result['equity'] -= result['years'] * self.CUSHIONS['insurance_premium']
            elif cushion == 'extra_principal':
                # Use pre-calculated extra principal scenario
                if 'extra_principal' in self.scenarios:
                    result = self.scenarios['extra_principal'].copy()
                    # Re-apply disasters
                    for disaster in self.active_disasters:
                        if disaster == 'hurricane':
                            result['equity'] -= self.DISASTER_COSTS['hurricane']
                        elif disaster == 'flood':
                            result['equity'] -= self.DISASTER_COSTS['flood']
                        elif disaster == 'sinkhole':
                            result['equity'] -= self.DISASTER_COSTS['sinkhole']

        return result


def create_interactive_plot(calculator):
    """Create interactive plot with toggles for disasters and cushions."""
    # Calculate base schedule
    calculator.calculate_amortization()
    calculator.apply_cushion('extra_principal')  # Pre-calculate for performance

    # Create figure with subplots
    fig = plt.figure(figsize=(16, 10))
    fig.suptitle('Home Loan Amortization: Your Equity vs Bank Profit\nFlorida 2026',
                 fontsize=16, fontweight='bold')

    # Main equity vs bank profit plot
    ax_main = plt.subplot2grid((3, 3), (0, 0), colspan=2, rowspan=2)

    # Bank profit area plot
    ax_bank = plt.subplot2grid((3, 3), (0, 2))

    # Equity breakdown plot
    ax_equity = plt.subplot2grid((3, 3), (1, 2))

    # Summary stats area (text)
    ax_stats = plt.subplot2grid((3, 3), (2, 0), colspan=3)
    ax_stats.axis('off')

    # Adjust layout for widgets
    plt.subplots_adjust(left=0.08, right=0.75, bottom=0.15, top=0.90, wspace=0.3, hspace=0.35)

    # Initial plot data
    schedule = calculator.get_combined_schedule()

    # Color scheme
    colors = {
        'equity': '#2ecc71',        # Green - your equity
        'bank_profit': '#e74c3c',   # Red - bank profit
        'principal': '#3498db',     # Blue - principal paid
        'home_value': '#9b59b6',    # Purple - home value
        'disaster': '#e67e22',      # Orange - disasters
        'cushion': '#1abc9c',       # Teal - cushions
    }

    def update_plot(val=None):
        """Update all plots based on active toggles."""
        schedule = calculator.get_combined_schedule()
        years = schedule['years']

        # Clear axes
        ax_main.clear()
        ax_bank.clear()
        ax_equity.clear()

        # Main plot: Equity vs Bank Profit
        ax_main.plot(years, schedule['equity'], color=colors['equity'],
                     linewidth=2.5, label='Your Equity')
        ax_main.plot(years, schedule['total_interest_paid'], color=colors['bank_profit'],
                     linewidth=2.5, label="Bank's Profit (Interest)")

        # Fill areas
        ax_main.fill_between(years, 0, schedule['equity'], alpha=0.3, color=colors['equity'])
        ax_main.fill_between(years, 0, schedule['total_interest_paid'], alpha=0.3, color=colors['bank_profit'])

        # Add disaster zones
        if calculator.active_disasters:
            disaster_color = colors['disaster']
            for disaster in calculator.active_disasters:
                if disaster == 'market_crash':
                    ax_main.axvspan(5, 30, alpha=0.15, color=disaster_color, label=f'Market Crash Impact')
                else:
                    ax_main.axhline(y=schedule['equity'].iloc[-1] - calculator.DISASTER_COSTS.get(disaster, 0),
                                    linestyle='--', alpha=0.5, color=disaster_color)

        ax_main.set_xlabel('Years', fontsize=11)
        ax_main.set_ylabel('Amount ($)', fontsize=11)
        ax_main.set_title('Cumulative: Your Equity vs Bank Profit', fontsize=12, fontweight='bold')
        ax_main.legend(loc='upper left')
        ax_main.grid(True, alpha=0.3)
        ax_main.set_xlim(0, 30)

        # Format y-axis as currency
        ax_main.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'${x/1000:.0f}K'))

        # Bank profit area plot
        cumulative_interest = schedule['total_interest_paid']
        ax_bank.fill_between(years, 0, cumulative_interest, color=colors['bank_profit'], alpha=0.6)
        ax_bank.set_title("What the Bank Earns", fontsize=10, fontweight='bold')
        ax_bank.set_ylabel('Total Interest ($)')
        ax_bank.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'${x/1000:.0f}K'))
        ax_bank.grid(True, alpha=0.3)
        ax_bank.set_xlim(0, 30)

        # Equity breakdown plot
        ax_equity.plot(years, schedule['total_principal_paid'], color=colors['principal'],
                      label='Principal Paid', linewidth=2)
        ax_equity.plot(years, schedule['equity'], color=colors['equity'],
                      label='Total Equity', linewidth=2)
        ax_equity.fill_between(years, schedule['total_principal_paid'], schedule['equity'],
                              color=colors['home_value'], alpha=0.3, label='Appreciation')
        ax_equity.set_title('Your Equity Breakdown', fontsize=10, fontweight='bold')
        ax_equity.legend(fontsize=8)
        ax_equity.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'${x/1000:.0f}K'))
        ax_equity.grid(True, alpha=0.3)
        ax_equity.set_xlim(0, 30)

        # Update summary stats
        ax_stats.clear()
        ax_stats.axis('off')

        final_equity = schedule['equity'].iloc[-1]
        final_bank_profit = schedule['total_interest_paid'].iloc[-1]
        total_paid = final_equity + final_bank_profit
        bank_share = (final_bank_profit / total_paid * 100) if total_paid > 0 else 0

        stats_text = f"""
        LOAN SUMMARY • Florida 2026
        ════════════════════════════════════════════════════════════════════
        Home Price: ${calculator.home_price:,.0f}  |  Down Payment: ${calculator.down_payment:,.0f}  |  Loan Amount: ${calculator.loan_amount:,.0f}
        Interest Rate: {calculator.interest_rate*100:.1f}%  |  Term: {calculator.loan_term_years} years  |  Appreciation: {calculator.appreciation_rate*100:.1f}%/yr

        ──────────────────────────────────────────────────────────────────────
        FINAL 30-YEAR PROJECTION:
        ▶ Your Home Equity:     ${final_equity:,.0f}
        ▶ Bank's Profit:        ${final_bank_profit:,.0f}
        ▶ Bank Share of Deal:   {bank_share:.1f}%

        ACTIVE SCENARIOS: {', '.join(calculator.active_disasters | calculator.active_cushions) if calculator.active_disasters or calculator.active_cushions else 'None'}
        ════════════════════════════════════════════════════════════════════
        """
        ax_stats.text(0.05, 0.5, stats_text, transform=ax_stats.transAxes,
                     fontsize=10, verticalalignment='center', family='monospace',
                     bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

        fig.canvas.draw_idle()

    # Create checkbox areas
    ax_disasters = plt.axes([0.77, 0.65, 0.20, 0.20])
    ax_cushions = plt.axes([0.77, 0.40, 0.20, 0.20])

    # Disaster toggles
    disaster_labels = ['Hurricane', 'Flood', 'Sinkhole', 'Market Crash']
    disaster_check = CheckButtons(ax_disasters, disaster_labels, [False] * len(disaster_labels))

    # Cushion toggles
    cushion_labels = ['Emergency Fund', 'Insurance', 'Extra Principal']
    cushion_check = CheckButtons(ax_cushions, cushion_labels, [False] * len(cushion_labels))

    # Styling
    ax_disasters.set_title("🌀 Disaster Scenarios", fontsize=10, fontweight='bold')
    ax_cushions.set_title("🛡️ Cushions & Mitigation", fontsize=10, fontweight='bold')

    def toggle_disaster(label):
        disaster_map = {
            'Hurricane': 'hurricane',
            'Flood': 'flood',
            'Sinkhole': 'sinkhole',
            'Market Crash': 'market_crash'
        }
        disaster = disaster_map[label]
        if disaster in calculator.active_disasters:
            calculator.active_disasters.remove(disaster)
        else:
            calculator.active_disasters.add(disaster)
        update_plot()

    def toggle_cushion(label):
        cushion_map = {
            'Emergency Fund': 'emergency_fund',
            'Insurance': 'insurance_premium',
            'Extra Principal': 'extra_principal'
        }
        cushion = cushion_map[label]
        if cushion in calculator.active_cushions:
            calculator.active_cushions.remove(cushion)
        else:
            calculator.active_cushions.add(cushion)
        update_plot()

    disaster_check.on_clicked(toggle_disaster)
    cushion_check.on_clicked(toggle_cushion)

    # Initial plot
    update_plot()

    return fig


def main():
    """Main entry point - example for Tony's Florida home purchase."""
    # Example parameters for Florida 2026
    # Adjust these based on Tony's specific situation
    calculator = LoanAmortizationCalculator(
        home_price=450000,           # $450K Florida home
        down_payment=0.20,           # 20% down ($90K)
        interest_rate=0.065,         # 6.5% interest rate
        loan_term_years=30,          # 30-year fixed
        start_year=2026,
        appreciation_rate=0.03       # 3% annual appreciation
    )

    # Create and show interactive plot
    fig = create_interactive_plot(calculator)
    plt.show()


if __name__ == '__main__':
    main()
