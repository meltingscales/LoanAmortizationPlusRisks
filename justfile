# Loan Amortization Calculator - Justfile
# Usage: `just <command>` or `just run`

default:
    @just --list

# Install dependencies with uv
install:
    uv sync

# Run the loan calculator (default settings)
run:
    uv run python loan_calculator.py

# =====================================
# Test Scripts - Example Scenarios
# =====================================

# Test 1: Typical Florida starter home ($350K)
test-starter:
    uv run python test_scenarios.py starter

# Test 2: Typical Florida family home ($450K - Tony's baseline)
test-typical:
    uv run python test_scenarios.py typical

# Test 3: Luxury Florida home ($650K)
test-luxury:
    uv run python test_scenarios.py luxury

# Test 4: High interest rate scenario (8%)
test-high-rate:
    uv run python test_scenarios.py high_rate

# Test 5: Low down payment (5% - FHA style)
test-low-down:
    uv run python test_scenarios.py low_down

# Test 6: 15-year loan (pay off faster)
test-15yr:
    uv run python test_scenarios.py 15yr

# Test 7: Disaster-heavy scenario (South Florida risks)
test-disaster:
    uv run python test_scenarios.py disaster

# Test 8: Run all test scenarios
test-all:
    uv run python test_scenarios.py all

# Run with specific home price (example: just custom-price 500000)
custom-price PRICE:
    uv run python test_scenarios.py price {{PRICE}}

# Run with custom parameters (interactive)
custom:
    uv run python test_scenarios.py interactive

# Export static chart (no interactivity)
export OUTPUT="equity_vs_bank.png":
    uv run python test_scenarios.py export {{OUTPUT}}

# Clean up generated files
clean:
    rm -rf .venv *.png __pycache__ .pytest_cache

# Update dependencies
update:
    uv lock --upgrade
