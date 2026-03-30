use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Para = ParagraphSpec;

pub fn page_density_demo() -> Result<Page, ChartError> {
    PageBuilder::new("density-demo", "Density Plots", "Density", 2)
        .category("Reference")
        .paragraph(
            Para::new(
                "Density plots reveal the full shape of a distribution across categories. \
                 This page demonstrates the automatic sina/violin selection:\n\n\
                 The top chart uses the raw salary dataset (≈10 data points per department). \
                 Because each category is sparsely populated the renderer chooses a \
                 sina plot — each observation is drawn as a scatter marker whose \
                 horizontal jitter is proportional to the local kernel-density estimate, \
                 so you can see every individual data point while still reading the \
                 distribution envelope.\n\n\
                 The bottom chart uses a denser performance-score dataset (50 observations \
                 per department). The higher point count triggers the violin variant — a \
                 mirrored KDE polygon is drawn for each category with a white median line \
                 overlaid, giving a smooth picture of the overall distribution shape. \
                 The threshold between modes defaults to 30 points per category and is \
                 configurable via DensityConfig::point_threshold().",
            )
            .title("Sina vs Violin — Automatic Mode Selection")
            .at(0, 0, 2)
            .build(),
        )
        // Sina mode: salary_raw has ~10 pts per department → below default threshold of 30
        .chart(
            C::density(
                "Salary by Department",
                "salary_raw",
                DensityConfig::builder()
                    .category("department")
                    .value("salary_k")
                    .y_label("Salary (k USD)")
                    .palette(PaletteSpec::Named("Set2".into()))
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        // Violin mode: density_scores has 50 pts per department → above default threshold of 30
        .chart(
            C::density(
                "Performance Score by Department",
                "density_scores",
                DensityConfig::builder()
                    .category("dept")
                    .value("score")
                    .y_label("Performance Score")
                    .palette(PaletteSpec::Named("Category10".into()))
                    .alpha(0.7)
                    .build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .build()
}
