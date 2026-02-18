# Loan Amortization + Risks Calculator 🏠📊

A home loan amortization calculator with visualization showing **your equity vs. what the bank makes**, designed for Florida home buyers in 2026.

## Features

- 📈 **Visual equity buildup** over 30 years
- 💰 **Bank profit (interest)** visualization - see exactly how much the bank earns
- 🌀 **Disaster scenarios**: Hurricane, Flood, Sinkhole, Market Crash
- 🛡️ **Cushions & mitigation**: Emergency fund, Insurance, Extra principal payments
- 🎨 **Color-coded areas** showing who gets what

## Quick Start

```bash
# Install dependencies (requires uv)
just install

# Run the interactive calculator
just run
```

## Commands

### Main Commands
| Command | Description |
|---------|-------------|
| `just install` | Install dependencies with `uv` |
| `just run` | Run the interactive calculator with default settings |
| `just custom` | Interactive - enter your own parameters |
| `just export` | Save static comparison chart to PNG |

### Test Scenarios
| Command | Description |
|---------|-------------|
| `just test-starter` | $350K starter home, 10% down, 6.5% rate |
| `just test-typical` | $450K typical home, 20% down, 6.5% rate (Tony's baseline) |
| `just test-luxury` | $650K luxury home, 20% down, 6.25% jumbo rate |
| `just test-high-rate` | $450K home with 8% interest rate (tough market) |
| `just test-low-down` | $450K home, 3.5% down (FHA-style), 6.5% rate |
| `just test-15yr` | $450K home, 15-year loan at 5.5% (fast payoff) |
| `just test-disaster` | South Florida high-risk scenario (lower appreciation) |
| `just test-all` | Run all scenarios and generate comparison chart |

### Custom Commands
| Command | Description |
|---------|-------------|
| `just custom-price $500000` | Run with specific home price |
| `just clean` | Clean up generated files |

## Example Output

```
============================================================
  Tony's Typical Florida Home
============================================================
  Home Price:        $450,000
  Down Payment:      $90,000 (20.0%)
  Loan Amount:       $360,000
  Interest Rate:     6.50%
  Loan Term:         30 years
  Appreciation:      3.0%/year

  ──────────────────────────────────────────────────
  FINAL 30-YEAR PROJECTION:
  Your Equity:       $1,092,268
  Bank's Profit:     $459,160
  Bank Share:        29.6%
  Total Cost:        $1,551,428
============================================================
```

## Key Insights

- **15-year loans** dramatically reduce bank profit (19.5% vs 29.6%)
- **Low down payments** increase bank share significantly (33.6% vs 29.6%)
- **High interest rates** are the biggest factor in bank profit (35.1% at 8%)
- **Disaster risk areas** have lower appreciation, reducing your final equity
