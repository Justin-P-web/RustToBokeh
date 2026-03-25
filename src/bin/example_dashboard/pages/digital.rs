use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;

pub fn page_web_analytics() -> Result<Page, ChartError> {
    PageBuilder::new("web-analytics", "Website Analytics", "Web", 2)
        .category("Digital")
        .chart(
            C::line(
                "Visitor Traffic",
                "website_traffic",
                Line::builder().x("month").y_cols(&["visitors"]).y_label("Visitors").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Signups Over Time",
                "website_traffic",
                Line::builder().x("month").y_cols(&["signups"]).y_label("Signups").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Conversions Over Time",
                "website_traffic",
                Line::builder().x("month").y_cols(&["conversions"]).y_label("Conversions").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_growth_indicators() -> Result<Page, ChartError> {
    PageBuilder::new("growth-indicators", "Growth Indicators", "Growth", 2)
        .category("Digital")
        .chart(
            C::line(
                "Revenue & Profit Growth",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Visitor Growth",
                "website_traffic",
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Products",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_marketing_roi() -> Result<Page, ChartError> {
    PageBuilder::new("marketing-roi", "Marketing ROI", "Marketing", 2)
        .category("Digital")
        .chart(
            C::bar(
                "Channel Spend by Quarter",
                "marketing_channels",
                Bar::builder().x("quarter").group("channel").value("spend").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Website Conversions",
                "website_traffic",
                Line::builder().x("month").y_cols(&["signups", "conversions"]).y_label("Count").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}
