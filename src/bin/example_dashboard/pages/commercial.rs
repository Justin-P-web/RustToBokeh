use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_product_analysis() -> Result<Page, ChartError> {
    PageBuilder::new("product-analysis", "Product Analysis", "Products", 2)
        .category("Commercial")
        .chart(
            C::bar(
                "Quarterly Product Revenue",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit by Team",
                "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::select(
            "scatter_performance",
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .filter(FilterSpec::range("scatter_performance", "revenue", "Revenue Range", 40.0, 320.0, 10.0))
        .build()
}

pub fn page_regional_breakdown() -> Result<Page, ChartError> {
    PageBuilder::new("regional-breakdown", "Regional Sales Breakdown", "Regions", 2)
        .category("Commercial")
        .chart(
            C::bar(
                "Sales by Region & Channel",
                "regional_sales",
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Revenue",
                "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Team Size").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_market_position() -> Result<Page, ChartError> {
    PageBuilder::new("market-position", "Market Position", "Market", 2)
        .category("Commercial")
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("Share %").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Project Completion",
                "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Revenue vs Costs (Quarterly)",
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .build()
}
