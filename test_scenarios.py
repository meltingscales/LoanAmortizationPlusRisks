#!/usr/bin/env python3
"""
Test scenarios for the Loan Amortization Calculator.
Run with: uv run python test_scenarios.py <scenario>
"""

import sys
import matplotlib
matplotlib.use('Agg')  # Non-interactive backend for tests
import matplotlib.pyplot as plt
from loan_calculator import LoanAmortizationCalculator, create_interactive_plot


# Scenario definitions
SCENARIOS = {
    'starter': {
        'name': 'Florida Starter Home',
        'home_price': 350000,
        'down_payment': 0.10,  # 10% down ($35K)
        'interest_rate': 0.065,
        'loan_term_years': 30,
        'appreciation_rate': 0.03,
        'description': '$350K home, 10% down, 6.5% rate'
    },
    'typical': {
        'name': "Tony's Typical Florida Home",
        'home_price': 450000,
        'down_payment': 0.20,  # 20% down ($90K)
        'interest_rate': 0.065,
        'loan_term_years': 30,
        'appreciation_rate': 0.03,
        'description': '$450K home, 20% down, 6.5% rate'
    },
    'luxury': {
        'name': 'Florida Luxury Home',
        'home_price': 650000,
        'down_payment': 0.20,  # 20% down ($130K)
        'interest_rate': 0.0625,
        'loan_term_years': 30,
        'appreciation_rate': 0.04,
        'description': '$650K home, 20% down, 6.25% rate (jumbo)'
    },
    'high_rate': {
        'name': 'High Interest Rate Scenario',
        'home_price': 450000,
        'down_payment': 0.20,
        'interest_rate': 0.08,  # 8% - tough market
        'loan_term_years': 30,
        'appreciation_rate': 0.03,
        'description': '$450K home, 20% down, 8% rate (inflation scenario)'
    },
    'low_down': {
        'name': 'Low Down Payment (FHA-style)',
        'home_price': 450000,
        'down_payment': 0.035,  # 3.5% down ($15,750)
        'interest_rate': 0.065,
        'loan_term_years': 30,
        'appreciation_rate': 0.03,
        'description': '$450K home, 3.5% down (FHA), 6.5% rate'
    },
    '15yr': {
        'name': '15-Year Fast Payoff',
        'home_price': 450000,
        'down_payment': 0.20,
        'interest_rate': 0.055,  # Lower rate for 15-yr
        'loan_term_years': 15,
        'appreciation_rate': 0.03,
        'description': '$450K home, 20% down, 5.5% rate, 15-year term'
    },
    'disaster': {
        'name': 'South Florida Disaster Risk',
        'home_price': 450000,
        'down_payment': 0.20,
        'interest_rate': 0.065,
        'loan_term_years': 30,
        'appreciation_rate': 0.02,  # Lower appreciation due to risk
        'description': '$450K home, high disaster risk area, 2% appreciation'
    },
}


def print_summary(calc, scenario_name):
    """Print a summary of the loan scenario."""
    schedule = calc.schedule
    final_equity = schedule['equity'].iloc[-1]
    final_bank_profit = schedule['total_interest_paid'].iloc[-1]
    total_paid = final_equity + final_bank_profit
    bank_share = (final_bank_profit / total_paid * 100) if total_paid > 0 else 0

    print(f"\n{'='*60}")
    print(f"  {scenario_name}")
    print(f"{'='*60}")
    print(f"  Home Price:        ${calc.home_price:,.0f}")
    print(f"  Down Payment:      ${calc.down_payment:,.0f} ({calc.down_payment/calc.home_price*100:.1f}%)")
    print(f"  Loan Amount:       ${calc.loan_amount:,.0f}")
    print(f"  Interest Rate:     {calc.interest_rate*100:.2f}%")
    print(f"  Loan Term:         {calc.loan_term_years} years")
    print(f"  Appreciation:      {calc.appreciation_rate*100:.1f}%/year")
    print(f"\n  ──────────────────────────────────────────────────")
    print(f"  FINAL 30-YEAR PROJECTION:")
    print(f"  Your Equity:       ${final_equity:,.0f}")
    print(f"  Bank's Profit:     ${final_bank_profit:,.0f}")
    print(f"  Bank Share:        {bank_share:.1f}%")
    print(f"  Total Cost:        ${total_paid:,.0f}")
    print(f"{'='*60}\n")


def create_comparison_chart(output_file='comparison.png'):
    """Create a side-by-side comparison of all scenarios."""
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Florida Home Loan Comparison: Different Scenarios\nYour Equity vs Bank Profit',
                 fontsize=16, fontweight='bold')

    scenarios_to_plot = ['starter', 'typical', 'high_rate', 'low_down']

    for idx, scenario_key in enumerate(scenarios_to_plot):
        ax = axes[idx // 2, idx % 2]
        params = SCENARIOS[scenario_key]

        calc = LoanAmortizationCalculator(**{k: v for k, v in params.items()
                                            if k not in ['name', 'description']})
        calc.calculate_amortization()

        years = calc.schedule['years']
        equity = calc.schedule['equity']
        bank_profit = calc.schedule['total_interest_paid']

        ax.plot(years, equity, 'g-', linewidth=2, label='Your Equity')
        ax.plot(years, bank_profit, 'r-', linewidth=2, label="Bank's Profit")
        ax.fill_between(years, 0, equity, alpha=0.3, color='green')
        ax.fill_between(years, 0, bank_profit, alpha=0.3, color='red')

        ax.set_title(params['description'], fontsize=11, fontweight='bold')
        ax.set_xlabel('Years')
        ax.set_ylabel('Amount ($)')
        ax.legend(fontsize=9)
        ax.grid(True, alpha=0.3)
        ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'${x/1000:.0f}K'))

    plt.tight_layout()
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    print(f"Comparison chart saved to: {output_file}")


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Available scenarios:")
        for key, params in SCENARIOS.items():
            print(f"  {key:12} - {params['description']}")
        print("\nOther commands:")
        print("  all          - Run all scenarios")
        print("  interactive  - Custom interactive input")
        print("  price <amt>  - Custom home price")
        print("  export <file> - Export comparison chart")
        sys.exit(1)

    command = sys.argv[1].lower()

    if command == 'all':
        print("\n" + "="*60)
        print("  RUNNING ALL SCENARIOS")
        print("="*60)

        for key, params in SCENARIOS.items():
            calc = LoanAmortizationCalculator(**{k: v for k, v in params.items()
                                                if k not in ['name', 'description']})
            calc.calculate_amortization()
            print_summary(calc, params['name'])

        print("\nGenerating comparison chart...")
        create_comparison_chart()

    elif command == 'interactive':
        print("\n--- Interactive Mode ---")
        price = float(input('Home price ($) [450000]: ') or 450000)
        down_pct = float(input('Down payment % (e.g. 20 for 20%) [20]: ') or 20) / 100
        rate = float(input('Interest rate % (e.g. 6.5) [6.5]: ') or 6.5) / 100
        term = int(input('Loan term years [30]: ') or 30)

        calc = LoanAmortizationCalculator(
            home_price=price,
            down_payment=down_pct,
            interest_rate=rate,
            loan_term_years=term,
            start_year=2026,
            appreciation_rate=0.03
        )
        calc.calculate_amortization()
        print_summary(calc, "Custom Scenario")

    elif command == 'price' and len(sys.argv) >= 3:
        price = float(sys.argv[2])
        calc = LoanAmortizationCalculator(
            home_price=price,
            down_payment=0.20,
            interest_rate=0.065,
            loan_term_years=30,
            start_year=2026,
            appreciation_rate=0.03
        )
        calc.calculate_amortization()
        print_summary(calc, f"Custom Price: ${price:,.0f}")

    elif command == 'export':
        output_file = sys.argv[2] if len(sys.argv) >= 3 else 'comparison.png'
        create_comparison_chart(output_file)

    elif command in SCENARIOS:
        params = SCENARIOS[command]
        calc = LoanAmortizationCalculator(**{k: v for k, v in params.items()
                                            if k not in ['name', 'description']})
        calc.calculate_amortization()
        print_summary(calc, params['name'])

    else:
        print(f"Unknown scenario: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
