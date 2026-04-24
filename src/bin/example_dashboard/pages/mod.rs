mod commercial;
mod executive;
mod operations;
mod people;
mod reference;

pub use commercial::{page_market_position, page_product_analysis, page_regional_breakdown};
pub use executive::page_executive_summary;
pub use operations::{
    page_cost_optimization, page_forecast_targets, page_operations_dashboard,
    page_project_portfolio,
};
pub use people::{page_customer_insights, page_team_metrics, page_workforce_planning};
pub use reference::{
    page_box_plot_demo, page_bubble_demo, page_chart_customization, page_density_demo,
    page_histogram_demo, page_module_showcase, page_pie_donut_charts, page_range_tool_demo,
    page_time_series_events,
};
