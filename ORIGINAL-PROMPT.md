I'd like to help my friend Tony buy a house in Florida in 2026. I want to give him a graph with 1. home equity, 2. how much the bank makes, 3. what if toggles for disasters or cushions that shows color coded areas under. it should be similar to a solar panel payoff calculator but for home loans. use rust/plotters (chart rendering). The reason for the chart area is to actually see how much the bank takes home from you versus your own equity.

## Implementation Notes

**Stack:** Rust + plotters (bitmap chart rendering) + egui/eframe (interactive GUI)

**Two modes:**
- CLI (`cargo run --bin loan-calc`): reads `config.toml`, generates `loan_comparison.png` and CSV files
- GUI (`cargo run --bin loan-calc-gui --features gui`): interactive egui window with live sliders

**GUI features:**
- Sliders for home price, down payment %, interest rate, loan term, appreciation rate
- Sliders for chart resolution (px) and font size
- Checkboxes to toggle each scenario on/off; chart regenerates live
- Scrollable chart area with pan support
- Export CSV button + open-file buttons per scenario

**Scenarios:**
1. Base Case — standard loan
2. High Rate (8%) — disaster: rate shock
3. Low Down (3.5%) — low down payment, more bank profit
4. Extra Principal (+$200/mo) — cushion: pay down faster
5. With Disasters — lower appreciation + $40K storm/flood costs (Florida-specific)

**Chart:** color-coded filled areas — green for homeowner equity, red for cumulative bank interest — side-by-side per scenario, with bank share % in each panel title.
