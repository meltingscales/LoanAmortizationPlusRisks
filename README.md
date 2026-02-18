# Loan Amortization + Risks Calculator 🏠📊

A home loan amortization calculator with visualization showing **your equity vs. what the bank makes**, written in Rust. Designed for Florida home buyers in 2026.

## Features

- 📈 **Visual equity buildup** over 30 years
- 💰 **Bank profit (interest)** visualization - see exactly how much the bank earns
- 🌀 **Disaster scenarios**: Hurricane, Flood, Sinkhole, Market Crash
- 🛡️ **Cushions & mitigation**: Emergency fund, Insurance, Extra principal payments
- 🎨 **Color-coded areas** showing who gets what
- ⚡ **Blazing fast** - native Rust performance

## Quick Start

```bash
# Build and run (optimized)
cargo run --release

# Or use just
just run-release
```

## Configuration

Edit `config.toml` to customize loan parameters:

```toml
[loan]
home_price = 450000
down_payment_percent = 20.0
interest_rate = 6.5
loan_term_years = 30
appreciation_rate = 3.0

[scenarios]
show_base = true
show_high_rate = true
show_low_down = true
show_extra_principal = true
show_disasters = true
```

## Commands

| Command | Description |
|---------|-------------|
| `cargo run --release` | Build and run optimized binary |
| `cargo run` | Build and run (debug, faster compile) |
| `just run-release` | Run via just (optimized) |
| `just build` | Build release binary |
| `just clean` | Clean build artifacts |

## Example Output

```
═══════════════════════════════════════════════
SCENARIO COMPARISON (30-Year Projection)
═══════════════════════════════════════════════
Base Case: Equity=$1092K Bank=$459K (29.6%)
Extra Principal (+$200/mo): Equity=$1092K Bank=$350K (24.3%)
High Rate (8%): Equity=$1092K Bank=$591K (35.1%)
Low Down (3.5%): Equity=$1092K Bank=$554K (33.6%)
With Disasters: Equity=$775K Bank=$459K (37.2%)
```

## Key Insights

- **15-year loans** dramatically reduce bank profit
- **Low down payments** increase bank share significantly
- **High interest rates** are the biggest factor in bank profit
- **Disaster risk areas** have lower appreciation, reducing your final equity

## Tech Stack

- **Rust** - Performance and reliability
- **plotters** - Chart generation
- **serde/toml** - Configuration parsing
- **itertools** - Data manipulation
