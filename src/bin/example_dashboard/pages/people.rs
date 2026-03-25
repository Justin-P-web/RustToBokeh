use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_team_metrics() -> Result<Page, ChartError> {
    PageBuilder::new("team-metrics", "Team & Workforce Metrics", "Team", 2)
        .category("People")
        .chart(
            C::bar(
                "Department Headcount by Year",
                "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Profit",
                "scatter_performance",
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Team Size").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::threshold(
            "scatter_performance",
            "satisfaction",
            "High Satisfaction Only (>4.2)",
            4.2,
            true,
        ))
        .build()
}

pub fn page_customer_insights() -> Result<Page, ChartError> {
    PageBuilder::new("customer-insights", "Customer Insights", "Customers", 2)
        .category("People")
        .chart(
            C::hbar(
                "Satisfaction Scores",
                "satisfaction",
                HB::builder().category("category").value("score").x_label("Score (1-5)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Customer Satisfaction",
                "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Profit vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("profit").y("satisfaction").x_label("Profit (k)").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::group(
            "scatter_performance",
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .build()
}

pub fn page_workforce_planning() -> Result<Page, ChartError> {
    PageBuilder::new("workforce-planning", "Workforce Planning", "Workforce", 2)
        .category("People")
        .chart(
            C::bar(
                "Headcount Growth",
                "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Revenue",
                "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Employees").y_label("Revenue (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Employees").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::hbar(
                "Budget by Department",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .filter(FilterSpec::top_n("scatter_performance", "revenue", "Top N by Revenue", 30, true))
        .filter(FilterSpec::threshold(
            "scatter_performance",
            "satisfaction",
            "High Satisfaction Only (>4.0)",
            4.0,
            true,
        ))
        .build()
}
