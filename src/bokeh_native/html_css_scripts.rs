// ── Embedded CSS ──────────────────────────────────────────────────────────────

pub const INLINE_CSS: &str = r#"<style>
        :root {
            color-scheme: light dark;

            /* ── Spacing ────────────────────────────────────────────────── */
            --s-1: 0.25rem; --s-2: 0.5rem;  --s-3: 0.75rem; --s-4: 1rem;
            --s-5: 1.5rem;  --s-6: 2rem;    --s-7: 3rem;    --s-8: 4rem;

            /* ── Type scale ─────────────────────────────────────────────── */
            --fs-cap:  0.68rem;
            --fs-xs:   0.72rem;
            --fs-sm:   0.78rem;
            --fs-base: 0.83rem;
            --fs-md:   0.92rem;
            --fs-lg:   1rem;
            --fs-xl:   1.05rem;
            --fs-stat: 1.25rem;

            /* ── Fonts ──────────────────────────────────────────────────── */
            /* Humanist body — system stack, avoids Plex/Inter/Space-Grotesk. */
            --ff-body: -apple-system, BlinkMacSystemFont, "Segoe UI", "Optima", "Lucida Sans Unicode", system-ui, sans-serif;
            /* Tabular numeric mono — JetBrains/IBM 3270 stand-ins via system mono. */
            --ff-mono: "JetBrains Mono", ui-monospace, "SF Mono", "Cascadia Mono", "Consolas", "Liberation Mono", monospace;

            /* ── Palette (OKLCH, light-dark pairs, no hex) ──────────────── */
            --c-bg:        light-dark(oklch(96% 0.008 250), oklch(11% 0.02 258));
            --c-surface:   light-dark(oklch(100% 0 0),     oklch(20% 0.022 258));
            --c-surface-2: light-dark(oklch(93% 0.012 250),oklch(26% 0.025 258));
            --c-fg:        light-dark(oklch(20% 0.02 255), oklch(94% 0.012 250));
            --c-fg-dim:    light-dark(oklch(42% 0.018 255),oklch(68% 0.014 255));
            --c-fg-muted:  light-dark(oklch(58% 0.012 250),oklch(52% 0.014 255));
            --c-border:    light-dark(oklch(84% 0.012 255),oklch(34% 0.025 258));
            --c-border-soft: light-dark(oklch(90% 0.008 250),oklch(28% 0.02 258));

            /* Sidebar is always dark (lab-instrument convention). */
            --c-nav-bg:        light-dark(oklch(99% 0.004 250),  oklch(8% 0.025 258));
            --c-nav-fg:        light-dark(oklch(22% 0.025 255),  oklch(95% 0.008 250));
            --c-nav-fg-dim:    light-dark(oklch(48% 0.018 255),  oklch(64% 0.018 255));
            --c-nav-fg-muted:  light-dark(oklch(64% 0.012 250),  oklch(44% 0.02 258));
            --c-nav-border:    light-dark(oklch(88% 0.012 255),  oklch(28% 0.025 258));
            --c-nav-hover-bg:  light-dark(oklch(95% 0.008 250),  oklch(22% 0.028 258));
            --c-nav-active-bg: light-dark(oklch(91% 0.012 250),  oklch(28% 0.032 258));

            /* One sharp accent (steel blue, calm). */
            --c-accent: oklch(62% 0.13 245);
            --c-accent-soft: light-dark(oklch(95% 0.04 245), oklch(30% 0.07 245));
            --c-accent-fg: light-dark(white, oklch(15% 0.02 258));

            /* ── Dimensions ─────────────────────────────────────────────── */
            --r-sm: 2px;
            --r-md: 3px;
            --sidebar-w: 200px;
            --page-max:  1400px;
            --nav-h:     38px;
        }

        /* ── Bokeh widget custom properties (cross shadow DOM) ────────── */
        /* Use * with !important so values apply directly to shadow host elements,
           beating Bokeh's :host {--color: #18191D} declarations. */
        * {
            --background-color: var(--c-surface) !important;
            --surface-background-color: var(--c-surface-2) !important;
            --color: var(--c-fg) !important;
            --border-color: var(--c-border) !important;
            --border: 1px solid var(--c-border) !important;
            --border-radius: var(--r-sm) !important;
            --placeholder-color: var(--c-fg-muted) !important;
            --input-focus-border-color: var(--c-accent) !important;
            --input-focus-halo-color: color-mix(in oklch, var(--c-accent) 35%, transparent) !important;
            --focus-border-color: var(--c-accent) !important;
            --focus-halo-color: color-mix(in oklch, var(--c-accent) 35%, transparent) !important;
            --disabled-background-color: var(--c-surface-2) !important;
            --disabled-color: var(--c-fg-muted) !important;
            --divider-color: var(--c-border-soft) !important;
            --hover-color: var(--c-surface-2) !important;
            --active-bg: var(--c-accent-soft) !important;
            --active-border: var(--c-accent) !important;
            --active-fg: var(--c-fg) !important;
            --inactive-bg: var(--c-surface-2) !important;
            --inactive-fg: var(--c-fg-muted) !important;
            --default: var(--c-surface) !important;
            --default-hover: var(--c-surface-2) !important;
            --default-active: var(--c-accent-soft) !important;
            --default-border: var(--c-border) !important;
            --default-hover-border: var(--c-accent) !important;
            --default-active-border: var(--c-accent) !important;
            --default-disabled: var(--c-surface-2) !important;
            --default-disabled-border: var(--c-border-soft) !important;
            --btn-color: var(--c-fg) !important;
            --shortcut-color: var(--c-fg-muted) !important;
            --highlight-color: var(--c-accent) !important;
            --inverted-color: var(--c-accent-fg) !important;
            --icon-color: var(--c-fg-dim) !important;
            --icon-color-disabled: var(--c-fg-muted) !important;
            --outline-color: var(--c-accent) !important;
            --bokeh-bg-color: var(--c-surface) !important;
            --bokeh-border-color: var(--c-border) !important;
            --bokeh-shadow-color: oklch(0% 0 0 / 0.18) !important;
        }

        /* ── Mode forcing ─────────────────────────────────────────────── */
        :root[data-mode="light"] { color-scheme: light; }
        :root[data-mode="dark"]  { color-scheme: dark; }

        /* ── Palette: classic (warm blue) ─────────────────────────────── */
        :root[data-theme="classic"] {
            --c-bg:        light-dark(oklch(98% 0.005 240), oklch(13% 0.018 240));
            --c-surface:   light-dark(oklch(100% 0 0),      oklch(21% 0.02 240));
            --c-surface-2: light-dark(oklch(95% 0.008 240), oklch(27% 0.022 240));
            --c-fg:        light-dark(oklch(22% 0.025 240), oklch(94% 0.012 240));
            --c-fg-dim:    light-dark(oklch(42% 0.02 240),  oklch(68% 0.015 240));
            --c-fg-muted:  light-dark(oklch(58% 0.015 240), oklch(52% 0.014 240));
            --c-border:    light-dark(oklch(85% 0.012 240), oklch(34% 0.025 240));
            --c-border-soft: light-dark(oklch(91% 0.008 240),oklch(28% 0.02 240));
            --c-accent:    oklch(58% 0.16 240);
            --c-accent-soft: light-dark(oklch(94% 0.05 240), oklch(32% 0.09 240));
            --c-accent-fg: light-dark(white, oklch(15% 0.02 240));
            --c-nav-bg:        light-dark(oklch(99% 0.004 240), oklch(9% 0.02 240));
            --c-nav-fg:        light-dark(oklch(22% 0.025 240), oklch(95% 0.008 240));
            --c-nav-fg-dim:    light-dark(oklch(48% 0.018 240), oklch(64% 0.018 240));
            --c-nav-fg-muted:  light-dark(oklch(64% 0.012 240), oklch(44% 0.02 240));
            --c-nav-border:    light-dark(oklch(88% 0.012 240), oklch(28% 0.025 240));
            --c-nav-hover-bg:  light-dark(oklch(95% 0.008 240), oklch(22% 0.028 240));
            --c-nav-active-bg: light-dark(oklch(91% 0.012 240), oklch(28% 0.032 240));
        }

        /* ── Palette: graphite (cool slate + teal) ────────────────────── */
        :root[data-theme="graphite"] {
            --c-bg:        light-dark(oklch(97% 0.004 230), oklch(12% 0.012 230));
            --c-surface:   light-dark(oklch(99.5% 0.002 230),oklch(20% 0.014 230));
            --c-surface-2: light-dark(oklch(94% 0.006 230), oklch(26% 0.016 230));
            --c-fg:        light-dark(oklch(24% 0.012 230), oklch(94% 0.008 230));
            --c-fg-dim:    light-dark(oklch(44% 0.01 230),  oklch(68% 0.01 230));
            --c-fg-muted:  light-dark(oklch(60% 0.008 230), oklch(52% 0.01 230));
            --c-border:    light-dark(oklch(86% 0.008 230), oklch(34% 0.014 230));
            --c-border-soft: light-dark(oklch(92% 0.005 230),oklch(28% 0.012 230));
            --c-accent:    oklch(60% 0.12 195);
            --c-accent-soft: light-dark(oklch(94% 0.04 195), oklch(32% 0.08 195));
            --c-accent-fg: light-dark(white, oklch(13% 0.012 230));
            --c-nav-bg:        light-dark(oklch(99% 0.003 230), oklch(8% 0.012 230));
            --c-nav-fg:        light-dark(oklch(24% 0.012 230), oklch(95% 0.006 230));
            --c-nav-fg-dim:    light-dark(oklch(48% 0.01 230),  oklch(64% 0.012 230));
            --c-nav-fg-muted:  light-dark(oklch(64% 0.008 230), oklch(44% 0.014 230));
            --c-nav-border:    light-dark(oklch(88% 0.008 230), oklch(28% 0.014 230));
            --c-nav-hover-bg:  light-dark(oklch(95% 0.005 230), oklch(22% 0.018 230));
            --c-nav-active-bg: light-dark(oklch(91% 0.008 230), oklch(28% 0.02 230));
        }

        /* ── Palette: forest (deep green + ochre) ─────────────────────── */
        :root[data-theme="forest"] {
            --c-bg:        light-dark(oklch(97% 0.012 130), oklch(13% 0.022 150));
            --c-surface:   light-dark(oklch(99.5% 0.005 130),oklch(20% 0.024 150));
            --c-surface-2: light-dark(oklch(93% 0.018 135), oklch(26% 0.026 150));
            --c-fg:        light-dark(oklch(24% 0.025 155), oklch(94% 0.018 130));
            --c-fg-dim:    light-dark(oklch(42% 0.02 155),  oklch(68% 0.018 130));
            --c-fg-muted:  light-dark(oklch(56% 0.015 155), oklch(52% 0.018 130));
            --c-border:    light-dark(oklch(85% 0.018 140), oklch(34% 0.025 150));
            --c-border-soft: light-dark(oklch(91% 0.012 140),oklch(28% 0.022 150));
            --c-accent:    oklch(60% 0.15 75);
            --c-accent-soft: light-dark(oklch(94% 0.05 75),  oklch(34% 0.1 75));
            --c-accent-fg: light-dark(white, oklch(15% 0.022 150));
            --c-nav-bg:        light-dark(oklch(99% 0.005 130), oklch(9% 0.025 150));
            --c-nav-fg:        light-dark(oklch(24% 0.025 155), oklch(95% 0.012 130));
            --c-nav-fg-dim:    light-dark(oklch(48% 0.02 155),  oklch(64% 0.018 130));
            --c-nav-fg-muted:  light-dark(oklch(64% 0.014 155), oklch(44% 0.022 150));
            --c-nav-border:    light-dark(oklch(88% 0.014 140), oklch(28% 0.024 150));
            --c-nav-hover-bg:  light-dark(oklch(95% 0.012 135), oklch(22% 0.028 150));
            --c-nav-active-bg: light-dark(oklch(91% 0.018 135), oklch(28% 0.032 150));
        }

        /* ── Palette: oxide (rust + verdigris) ────────────────────────── */
        :root[data-theme="oxide"] {
            --c-bg:        light-dark(oklch(97% 0.012 40),  oklch(13% 0.022 30));
            --c-surface:   light-dark(oklch(99.5% 0.005 40),oklch(20% 0.024 30));
            --c-surface-2: light-dark(oklch(93% 0.018 38),  oklch(26% 0.028 30));
            --c-fg:        light-dark(oklch(24% 0.025 30),  oklch(94% 0.018 40));
            --c-fg-dim:    light-dark(oklch(42% 0.02 30),   oklch(68% 0.018 40));
            --c-fg-muted:  light-dark(oklch(56% 0.015 30),  oklch(52% 0.018 40));
            --c-border:    light-dark(oklch(85% 0.018 35),  oklch(34% 0.026 30));
            --c-border-soft: light-dark(oklch(91% 0.012 35),oklch(28% 0.024 30));
            --c-accent:    oklch(58% 0.15 30);
            --c-accent-soft: light-dark(oklch(94% 0.05 30), oklch(34% 0.1 30));
            --c-accent-fg: light-dark(white, oklch(15% 0.022 30));
            --c-nav-bg:        light-dark(oklch(99% 0.005 40),  oklch(9% 0.025 30));
            --c-nav-fg:        light-dark(oklch(24% 0.025 30),  oklch(95% 0.012 40));
            --c-nav-fg-dim:    light-dark(oklch(48% 0.02 30),   oklch(64% 0.018 40));
            --c-nav-fg-muted:  light-dark(oklch(64% 0.014 30),  oklch(44% 0.022 30));
            --c-nav-border:    light-dark(oklch(88% 0.014 35),  oklch(28% 0.024 30));
            --c-nav-hover-bg:  light-dark(oklch(95% 0.012 38),  oklch(22% 0.028 30));
            --c-nav-active-bg: light-dark(oklch(91% 0.018 38),  oklch(28% 0.032 30));
        }

        * { box-sizing: border-box; }
        html, body { margin: 0; padding: 0; }
        body {
            font-family: var(--ff-body);
            font-size: var(--fs-base);
            background: var(--c-bg);
            color: var(--c-fg);
            -webkit-font-smoothing: antialiased;
        }
        body { font-feature-settings: "tnum" 1, "lnum" 1; }
        a { color: var(--c-accent); text-decoration: none; }
        a:hover { text-decoration: underline; }

        /* ── Horizontal nav ────────────────────────────────────────────── */
        .nav-horizontal {
            background: var(--c-surface);
            border-bottom: 1px solid var(--c-border);
            position: sticky; top: 0; z-index: 100;
        }
        .nav-horizontal .nav-header { display: flex; align-items: stretch; padding: 0 var(--s-4); }
        .nav-horizontal .nav-report-title {
            font-family: var(--ff-mono);
            font-size: var(--fs-sm); font-weight: 500;
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-fg);
            padding: 0 var(--s-4) 0 0; margin-right: var(--s-2);
            border-right: 1px solid var(--c-border);
            display: flex; align-items: center; flex-shrink: 0;
        }
        .nav-horizontal .nav-tabs-scroll {
            display: flex; align-items: stretch; flex: 1;
            overflow-x: auto; scrollbar-width: none; -ms-overflow-style: none;
        }
        .nav-horizontal .nav-tabs-scroll::-webkit-scrollbar { display: none; }
        .nav-horizontal .nav-tab {
            display: flex; align-items: center; gap: var(--s-2);
            padding: 0 var(--s-3); height: var(--nav-h);
            font-size: var(--fs-sm); color: var(--c-fg-dim);
            white-space: nowrap; border-bottom: 2px solid transparent;
            transition: color 0.1s, border-color 0.1s, background 0.1s;
        }
        .nav-horizontal .nav-tab:hover { color: var(--c-fg); background: var(--c-surface-2); text-decoration: none; }
        .nav-horizontal .nav-tab.active { color: var(--c-fg); border-bottom-color: var(--c-accent); font-weight: 500; }
        .nav-horizontal .nav-dd { position: relative; display: flex; align-items: stretch; flex-shrink: 0; }
        .nav-horizontal .nav-dd-trigger {
            display: flex; align-items: center; gap: var(--s-1);
            padding: 0 var(--s-3); height: var(--nav-h); border: none; background: none;
            font-family: inherit; font-size: var(--fs-sm); color: var(--c-fg-dim);
            white-space: nowrap; cursor: pointer; border-bottom: 2px solid transparent;
            transition: color 0.1s, border-color 0.1s, background 0.1s;
        }
        .nav-horizontal .nav-dd-trigger .caret { font-size: var(--fs-xs); opacity: 0.55; }
        .nav-horizontal .nav-dd:hover > .nav-dd-trigger,
        .nav-horizontal .nav-dd.open > .nav-dd-trigger { color: var(--c-fg); background: var(--c-surface-2); }
        .nav-horizontal .nav-dd.has-active > .nav-dd-trigger { color: var(--c-fg); border-bottom-color: var(--c-accent); }
        .nav-horizontal .nav-dd-menu,
        .nav-horizontal .nav-dd-sub-menu {
            display: none; position: fixed; background: var(--c-surface);
            border: 1px solid var(--c-border); border-radius: var(--r-md);
            min-width: 190px; z-index: 1000; padding: var(--s-1) 0;
            box-shadow: 0 4px 16px oklch(0% 0 0 / 0.18);
        }
        .nav-horizontal .nav-dd-item {
            display: flex; align-items: center; gap: var(--s-2);
            padding: var(--s-2) var(--s-3); font-size: var(--fs-sm); color: var(--c-fg-dim);
            white-space: nowrap; transition: background 0.1s, color 0.1s;
        }
        .nav-horizontal .nav-dd-item:hover { background: var(--c-surface-2); color: var(--c-fg); text-decoration: none; }
        .nav-horizontal .nav-dd-item.active { background: var(--c-surface-2); color: var(--c-fg); font-weight: 500; }
        .nav-horizontal .nav-dd-divider { border: none; border-top: 1px solid var(--c-border-soft); margin: var(--s-1) 0; }
        .nav-horizontal .nav-dd-sub { position: relative; }
        .nav-horizontal .nav-dd-sub-trigger {
            display: flex; justify-content: space-between; align-items: center; width: 100%;
            padding: var(--s-2) var(--s-3); border: none; background: none; font-family: inherit;
            font-size: var(--fs-sm); color: var(--c-fg-dim); white-space: nowrap; cursor: pointer;
            transition: background 0.1s, color 0.1s; text-align: left;
        }
        .nav-horizontal .nav-dd-sub-trigger .caret { font-size: var(--fs-xs); opacity: 0.55; }
        .nav-horizontal .nav-dd-sub:hover > .nav-dd-sub-trigger,
        .nav-horizontal .nav-dd-sub.open > .nav-dd-sub-trigger { background: var(--c-surface-2); color: var(--c-fg); }
        .nav-horizontal .nav-dd-sub.has-active > .nav-dd-sub-trigger { color: var(--c-fg); }

        /* ── Vertical nav (dark sidebar) ───────────────────────────────── */
        .nav-vertical {
            position: fixed; left: 0; top: 0;
            width: var(--sidebar-w); height: 100vh; overflow-y: auto;
            background: var(--c-nav-bg); color: var(--c-nav-fg);
            border-right: 1px solid var(--c-nav-border);
            z-index: 100; padding-bottom: var(--s-4);
            scrollbar-width: thin; scrollbar-color: var(--c-nav-border) transparent;
        }
        .nav-vertical .nav-report-title {
            display: flex; align-items: center; gap: var(--s-2);
            font-family: var(--ff-mono); font-size: var(--fs-sm); font-weight: 500;
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-nav-fg);
            padding: var(--s-3) var(--s-4); border-bottom: 1px solid var(--c-nav-border);
        }
        .nav-vertical .nav-report-title::before {
            content: ""; width: 7px; height: 7px; border-radius: 50%;
            background: var(--c-accent); flex-shrink: 0;
        }
        .nav-vertical .nav-report-title a { color: inherit; }
        .nav-vertical .nav-search {
            padding: var(--s-2) var(--s-3); border-bottom: 1px solid var(--c-nav-border);
        }
        .nav-vertical .nav-search-input {
            width: 100%; box-sizing: border-box;
            background: light-dark(oklch(96% 0.006 250), oklch(12% 0.022 258)); border: 1px solid var(--c-nav-border);
            border-radius: var(--r-sm); color: var(--c-nav-fg);
            padding: var(--s-1) var(--s-2) var(--s-1) 26px;
            font-family: var(--ff-mono); font-size: var(--fs-xs);
            outline: none;
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%237a8295' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cline x1='21' y1='21' x2='16.65' y2='16.65'/%3E%3C/svg%3E");
            background-repeat: no-repeat; background-position: var(--s-2) center; background-size: 12px 12px;
        }
        .nav-vertical .nav-search-input::placeholder { color: var(--c-nav-fg-muted); }
        .nav-vertical .nav-search-input:focus { border-color: var(--c-accent); }
        .nav-vertical .nav-uncategorized { padding: var(--s-2) 0; }
        .nav-vertical details > summary {
            list-style: none; cursor: pointer; user-select: none;
            font-family: var(--ff-mono); font-size: var(--fs-cap); font-weight: 500;
            text-transform: uppercase; letter-spacing: 0.12em;
            color: var(--c-nav-fg-muted);
            padding: var(--s-3) var(--s-4) var(--s-1);
            display: flex; align-items: center; justify-content: space-between;
        }
        .nav-vertical details > summary::-webkit-details-marker { display: none; }
        .nav-vertical details > summary::after {
            content: "+"; font-size: 0.95em; opacity: 0.55; transition: transform 0.15s;
        }
        .nav-vertical details[open] > summary::after { content: "−"; }
        .nav-vertical .nav-indent { padding-left: 0; }
        .nav-vertical a {
            display: flex; align-items: center; gap: var(--s-2);
            padding: var(--s-1) var(--s-4); margin: 0;
            font-size: var(--fs-sm); color: var(--c-nav-fg-dim);
            line-height: 1.6;
            transition: background 0.1s, color 0.1s;
        }
        .nav-vertical a:hover { color: var(--c-nav-fg); background: var(--c-nav-hover-bg); text-decoration: none; }
        .nav-vertical a.active { color: var(--c-nav-fg); background: var(--c-nav-active-bg); font-weight: 500; box-shadow: inset 2px 0 0 var(--c-accent); }
        .nav-vertical .nav-dot {
            width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
            display: inline-block;
        }

        /* ── Page layout ───────────────────────────────────────────────── */
        .layout-horizontal .page-content {
            max-width: var(--page-max); margin: 0 auto;
            padding: var(--s-4) var(--s-5) var(--s-7);
        }
        .layout-vertical .page-content {
            margin-left: var(--sidebar-w);
            padding: var(--s-4) var(--s-5) var(--s-7);
        }

        h1 {
            font-family: var(--ff-body);
            font-size: var(--fs-xl); font-weight: 600;
            color: var(--c-fg); margin: 0;
            letter-spacing: -0.005em;
        }
        .page-header-bar {
            display: flex; align-items: flex-end; justify-content: space-between;
            gap: var(--s-4); margin: 0 0 var(--s-3);
            padding-bottom: var(--s-3); border-bottom: 1px solid var(--c-border);
        }
        .export-btn {
            display: inline-flex; align-items: center; gap: var(--s-2);
            font-family: var(--ff-mono); font-size: var(--fs-xs);
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-fg-dim); background: var(--c-surface);
            border: 1px solid var(--c-border); border-radius: var(--r-sm);
            padding: var(--s-2) var(--s-3); cursor: pointer; flex-shrink: 0;
            transition: color 0.1s, background 0.1s, border-color 0.1s;
        }
        .export-btn:hover {
            color: var(--c-fg); background: var(--c-surface-2);
            border-color: var(--c-accent);
        }
        .export-btn:active { background: var(--c-accent-soft); }
        .export-btn-icon { font-size: var(--fs-sm); line-height: 1; }
        .subtitle {
            font-family: var(--ff-mono); font-size: var(--fs-xs);
            text-transform: uppercase; letter-spacing: 0.1em;
            color: var(--c-fg-muted); margin: 0 0 var(--s-4);
        }

        /* ── Grid layout ───────────────────────────────────────────────── */
        .grid-layout { display: grid; gap: var(--s-3); margin-bottom: var(--s-4); }
        .chart-container {
            background: var(--c-surface);
            border: 1px solid var(--c-border); border-radius: var(--r-md);
            padding: var(--s-3) var(--s-3) var(--s-2);
            min-width: 0;
        }
        /* Stat grid + paragraph drop the card chrome */
        .chart-container:has(.paragraph-module) { background: transparent; border: none; padding: var(--s-2) 0; }
        .chart-container:has(.stat-grid)        { background: transparent; border: none; padding: 0; }

        .chart-title {
            font-family: var(--ff-mono); font-size: var(--fs-cap);
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-fg-dim); font-weight: 500;
            margin: 0 0 var(--s-2);
            padding-bottom: var(--s-1);
            border-bottom: 1px solid var(--c-border-soft);
        }
        .module-title {
            font-family: var(--ff-mono); font-size: var(--fs-cap);
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-fg-dim); font-weight: 500;
            margin: 0 0 var(--s-2);
            padding-bottom: var(--s-1);
            border-bottom: 1px solid var(--c-border-soft);
        }

        /* ── Stat grid module ──────────────────────────────────────────── */
        .stat-grid {
            display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
            gap: var(--s-2);
        }
        .stat-card {
            background: var(--c-surface);
            border: 1px solid var(--c-border); border-radius: var(--r-md);
            padding: var(--s-2) var(--s-3);
        }
        .stat-label {
            font-family: var(--ff-mono); font-size: var(--fs-cap);
            text-transform: uppercase; letter-spacing: 0.08em;
            color: var(--c-fg-muted); margin-bottom: var(--s-1);
            font-weight: 500;
        }
        .stat-value {
            font-family: var(--ff-mono); font-size: var(--fs-stat);
            color: var(--c-fg); font-weight: 500; line-height: 1.05;
            font-variant-numeric: tabular-nums;
        }
        .stat-suffix {
            font-size: var(--fs-base); color: var(--c-fg-muted); margin-left: 0.2em;
            font-weight: 400;
        }

        /* ── Filter bar ────────────────────────────────────────────────── */
        .filter-bar {
            display: flex; flex-wrap: wrap; gap: var(--s-3);
            padding: var(--s-2) var(--s-3); margin-bottom: var(--s-3);
            background: var(--c-surface);
            border: 1px solid var(--c-border); border-radius: var(--r-md);
            border-left: 2px solid var(--c-accent);
            align-items: center;
        }
        .filter-bar-label {
            font-family: var(--ff-mono); font-size: var(--fs-cap);
            text-transform: uppercase; letter-spacing: 0.1em;
            color: var(--c-fg-dim); font-weight: 500;
            white-space: nowrap;
        }
        .filter-widget { flex: 1; min-width: 180px; }
        .switch-label {
            display: flex; align-items: center; gap: var(--s-2);
            font-size: var(--fs-sm); color: var(--c-fg);
        }

        /* ── Paragraph module ──────────────────────────────────────────── */
        .paragraph-module { height: 100%; }
        .paragraph-module p {
            color: var(--c-fg); line-height: 1.55;
            margin: 0 0 var(--s-2); font-size: var(--fs-base);
        }
        .paragraph-module p:last-child { margin-bottom: 0; }

        /* ── Table module ──────────────────────────────────────────────── */
        .table-module { overflow: hidden; }
        .table-wrapper { overflow-x: auto; max-height: 420px; overflow-y: auto; }
        .table-module table {
            width: 100%; border-collapse: collapse;
            font-family: var(--ff-mono); font-size: var(--fs-xs);
            font-variant-numeric: tabular-nums;
        }
        .table-module thead th {
            background: var(--c-surface); color: var(--c-fg-muted);
            padding: var(--s-2) var(--s-3); text-align: left;
            font-size: var(--fs-cap); text-transform: uppercase; letter-spacing: 0.08em;
            font-weight: 500;
            position: sticky; top: 0;
            border-bottom: 1px solid var(--c-border);
            white-space: nowrap;
        }
        .table-module tbody td {
            padding: var(--s-2) var(--s-3);
            color: var(--c-fg); border-bottom: 1px solid var(--c-border-soft);
        }
        .table-module tbody tr:last-child td { border-bottom: none; }
        .table-module tbody tr:hover td { background: var(--c-surface-2); }

        /* ── Range tool overview wrapper ───────────────────────────────── */
        .range-overview {
            background: var(--c-surface);
            border: 1px dashed var(--c-border); border-radius: var(--r-md);
            padding: var(--s-2) var(--s-3);
            margin-bottom: var(--s-3);
        }

        /* ── Theme switcher ────────────────────────────────────────────── */
        .theme-switcher { position: relative; flex-shrink: 0; }
        .nav-horizontal .theme-switcher { margin-left: auto; display: flex; align-items: stretch; border-left: 1px solid var(--c-border); }
        .nav-vertical .theme-switcher { padding: var(--s-3) var(--s-3); border-top: 1px solid var(--c-nav-border); margin-top: auto; position: sticky; bottom: 0; background: var(--c-nav-bg); }
        .theme-switcher-trigger {
            display: flex; align-items: center; gap: var(--s-2);
            border: none; background: none; cursor: pointer;
            font-family: inherit; font-size: var(--fs-sm);
            padding: 0 var(--s-3); transition: color 0.1s, background 0.1s;
        }
        .nav-horizontal .theme-switcher-trigger {
            height: var(--nav-h); color: var(--c-fg-dim);
            border-bottom: 2px solid transparent;
        }
        .nav-horizontal .theme-switcher-trigger:hover { color: var(--c-fg); background: var(--c-surface-2); }
        .nav-vertical .theme-switcher-trigger {
            width: 100%; padding: var(--s-2) var(--s-3);
            color: var(--c-nav-fg-dim);
            background: var(--c-nav-hover-bg);
            border: 1px solid var(--c-nav-border); border-radius: var(--r-sm);
        }
        .nav-vertical .theme-switcher-trigger:hover { color: var(--c-nav-fg); background: var(--c-nav-active-bg); }
        .theme-swatch {
            width: 12px; height: 12px; border-radius: 50%;
            background: var(--c-accent);
            border: 1px solid var(--c-border); flex-shrink: 0;
        }
        .theme-switcher-trigger .caret { font-size: var(--fs-xs); opacity: 0.55; }
        .theme-switcher-menu {
            display: none; position: fixed;
            background: var(--c-surface); color: var(--c-fg);
            border: 1px solid var(--c-border); border-radius: var(--r-md);
            min-width: 200px; z-index: 1100; padding: var(--s-1) 0;
            box-shadow: 0 4px 16px oklch(0% 0 0 / 0.22);
        }
        .theme-switcher-menu.open { display: block; }
        .theme-section {
            font-family: var(--ff-mono); font-size: var(--fs-cap);
            text-transform: uppercase; letter-spacing: 0.1em;
            color: var(--c-fg-muted); font-weight: 500;
            padding: var(--s-2) var(--s-3) var(--s-1);
        }
        .theme-option {
            display: flex; align-items: center; gap: var(--s-2);
            width: 100%; border: none; background: none; cursor: pointer;
            padding: var(--s-2) var(--s-3); text-align: left;
            font-family: inherit; font-size: var(--fs-sm); color: var(--c-fg-dim);
            transition: background 0.1s, color 0.1s;
        }
        .theme-option:hover { background: var(--c-surface-2); color: var(--c-fg); }
        .theme-option.active { color: var(--c-fg); font-weight: 500; }
        .theme-option.active::after { content: "●"; margin-left: auto; color: var(--c-accent); font-size: 0.8em; }
        .theme-option-swatch { width: 10px; height: 10px; border-radius: 50%; border: 1px solid var(--c-border); flex-shrink: 0; }
        .theme-divider { border: none; border-top: 1px solid var(--c-border-soft); margin: var(--s-1) 0; }

        /* ── Bokeh widget overrides (filters, sliders, dropdowns) ──────── */
        .bk-input,
        select.bk-input,
        textarea.bk-input {
            background-color: var(--c-surface) !important;
            color: var(--c-fg) !important;
            border: 1px solid var(--c-border) !important;
            border-radius: var(--r-sm) !important;
        }
        .bk-input::placeholder { color: var(--c-fg-muted) !important; }
        .bk-input:focus,
        .bk-input:focus-visible {
            border-color: var(--c-accent) !important;
            outline: none !important;
            box-shadow: 0 0 0 1px var(--c-accent) !important;
        }
        .bk-input:disabled { color: var(--c-fg-muted) !important; opacity: 0.6 !important; }
        select.bk-input option { background-color: var(--c-surface); color: var(--c-fg); }

        .bk-input-group > label,
        .bk-input-group .bk-input-group-text {
            color: var(--c-fg-dim) !important;
        }

        .bk-btn,
        .bk-btn-default,
        .bk-btn-group .bk-btn {
            background-color: var(--c-surface) !important;
            color: var(--c-fg) !important;
            border: 1px solid var(--c-border) !important;
            border-radius: var(--r-sm) !important;
        }
        .bk-btn:hover, .bk-btn-default:hover {
            background-color: var(--c-surface-2) !important;
            border-color: var(--c-accent) !important;
        }
        .bk-btn.bk-active,
        .bk-btn-default.bk-active {
            background-color: var(--c-accent-soft) !important;
            border-color: var(--c-accent) !important;
            color: var(--c-fg) !important;
        }

        .bk-toolbar { background-color: transparent !important; }
        .bk-toolbar .bk-tool-icon-color { color: var(--c-fg-dim) !important; }
        .bk-toolbar .bk-tool-icon-color:hover { color: var(--c-fg) !important; }

        .bk-tooltip,
        .bk-tooltip-content,
        div.bk-tooltip {
            background-color: var(--c-surface) !important;
            color: var(--c-fg) !important;
            border: 1px solid var(--c-border) !important;
            border-radius: var(--r-sm) !important;
            box-shadow: 0 4px 12px oklch(0% 0 0 / 0.18) !important;
        }

        .noUi-target {
            background: var(--c-surface-2) !important;
            border: 1px solid var(--c-border) !important;
            box-shadow: none !important;
        }
        .noUi-connect { background: var(--c-accent) !important; }
        .noUi-handle {
            background: var(--c-surface) !important;
            border: 1px solid var(--c-border) !important;
            box-shadow: 0 1px 3px oklch(0% 0 0 / 0.2) !important;
        }
        .noUi-handle::before, .noUi-handle::after { background: var(--c-fg-muted) !important; }
        .noUi-tooltip {
            background: var(--c-surface) !important;
            color: var(--c-fg) !important;
            border: 1px solid var(--c-border) !important;
        }
        .noUi-marker, .noUi-marker-large, .noUi-marker-sub { background: var(--c-border) !important; }
        .noUi-value, .noUi-value-sub { color: var(--c-fg-muted) !important; }
        .noUi-pips { color: var(--c-fg-muted) !important; }

        .choices__inner {
            background-color: var(--c-surface) !important;
            border: 1px solid var(--c-border) !important;
            color: var(--c-fg) !important;
        }
        .choices__list--dropdown,
        .choices__list[aria-expanded] {
            background-color: var(--c-surface) !important;
            border: 1px solid var(--c-border) !important;
            color: var(--c-fg) !important;
        }
        .choices__item { color: var(--c-fg) !important; }
        .choices__item--choice { color: var(--c-fg) !important; }
        .choices__item--choice.is-highlighted,
        .choices__item--choice:hover {
            background-color: var(--c-surface-2) !important;
            color: var(--c-fg) !important;
        }
        .choices__item--selectable.is-highlighted { background-color: var(--c-surface-2) !important; }
        .choices__list--multiple .choices__item {
            background-color: var(--c-accent) !important;
            border-color: var(--c-accent) !important;
            color: var(--c-accent-fg) !important;
        }
        .choices__input { background-color: transparent !important; color: var(--c-fg) !important; }
        .choices[data-type*="select"]::after { border-color: var(--c-fg-muted) transparent transparent !important; }

        .bk-switch { background: var(--c-surface-2) !important; border: 1px solid var(--c-border) !important; }
        .bk-switch.bk-active { background: var(--c-accent) !important; border-color: var(--c-accent) !important; }
        .bk-switch .bk-switch-bar { background: var(--c-fg-muted) !important; }
        .bk-switch.bk-active .bk-switch-bar { background: var(--c-accent-fg) !important; }

        /* ── Tech badges hidden (lab aesthetic — no marketing chrome) ─── */
        .tech-badge { display: none; }

        /* ── Footer ────────────────────────────────────────────────────── */
        footer {
            margin-top: var(--s-5);
            font-family: var(--ff-mono); font-size: var(--fs-xs);
            text-transform: uppercase; letter-spacing: 0.1em;
            color: var(--c-fg-muted); text-align: center;
        }

        /* ── Responsive ────────────────────────────────────────────────── */
        @media (max-width: 900px) {
            :root { --sidebar-w: 168px; }
        }
        @media (max-width: 600px) {
            .layout-vertical .page-content { margin-left: 0; }
            .nav-vertical { display: none; }
        }
    </style>"#;

// ── Theme switcher head pre-paint (FOUC avoidance) ───────────────────────────

pub const THEME_HEAD_SCRIPT: &str = r#"<script>
(function(){
    function getParam(h, k){
        if (!h) return null;
        var ps = h.replace(/^#/,'').split('&');
        for (var i=0;i<ps.length;i++){
            var kv = ps[i].split('=');
            if (kv[0] === k) return decodeURIComponent(kv[1] || '');
        }
        return null;
    }
    var t = getParam(window.location.hash, 'theme');
    var m = getParam(window.location.hash, 'mode');
    try {
        if (!t) t = localStorage.getItem('rtb-theme');
        if (!m) m = localStorage.getItem('rtb-mode');
    } catch(e){}
    if (t) document.documentElement.setAttribute('data-theme', t);
    if (m && m !== 'auto') document.documentElement.setAttribute('data-mode', m);
})();
</script>"#;

// ── Export button (top-right of every page header) ───────────────────────────

/// Inline HTML for the export button rendered at the top-right of each page's
/// content header. The button triggers [`EXPORT_SCRIPT`], which downloads the
/// `report.typ` source previously embedded as `window.__rtb_report_typst`.
pub const EXPORT_BUTTON_HTML: &str = r#"<button class="export-btn" id="export-typst-btn" type="button" aria-label="Export report as Typst source">
                <span class="export-btn-icon">⬇</span><span>Export .typ</span>
            </button>"#;

/// JS handler for the export button. Reads the inline Typst source from
/// `window.__rtb_report_typst`, builds a Blob, and triggers a download of
/// `report.typ`. Compile to PDF afterward with `typst compile report.typ`.
pub const EXPORT_SCRIPT: &str = r#"<script>
(function () {
    var btn = document.getElementById('export-typst-btn');
    if (!btn) return;
    btn.addEventListener('click', function () {
        var src = window.__rtb_report_typst || '';
        if (!src) {
            alert('No Typst source available for this report.');
            return;
        }
        var blob = new Blob([src], { type: 'text/plain;charset=utf-8' });
        var url = URL.createObjectURL(blob);
        var a = document.createElement('a');
        a.href = url;
        a.download = 'report.typ';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        setTimeout(function () { URL.revokeObjectURL(url); }, 0);
    });
})();
</script>"#;

// ── Theme switcher HTML (drop into nav containers) ───────────────────────────

pub const THEME_SWITCHER_HTML: &str = r##"<div class="theme-switcher">
    <button class="theme-switcher-trigger" id="theme-switcher-trigger" aria-label="Theme">
        <span class="theme-swatch"></span><span class="theme-switcher-label">Theme</span><span class="caret">▾</span>
    </button>
    <div class="theme-switcher-menu" id="theme-switcher-menu">
        <div class="theme-section">Mode</div>
        <button class="theme-option" data-mode="auto">Auto (system)</button>
        <button class="theme-option" data-mode="light">Light</button>
        <button class="theme-option" data-mode="dark">Dark</button>
        <hr class="theme-divider">
        <div class="theme-section">Palette</div>
        <button class="theme-option" data-theme="lab"><span class="theme-option-swatch" style="background:oklch(62% 0.13 245)"></span>Lab</button>
        <button class="theme-option" data-theme="classic"><span class="theme-option-swatch" style="background:oklch(58% 0.16 240)"></span>Classic</button>
        <button class="theme-option" data-theme="graphite"><span class="theme-option-swatch" style="background:oklch(60% 0.12 195)"></span>Graphite</button>
        <button class="theme-option" data-theme="forest"><span class="theme-option-swatch" style="background:oklch(60% 0.15 75)"></span>Forest</button>
        <button class="theme-option" data-theme="oxide"><span class="theme-option-swatch" style="background:oklch(58% 0.15 30)"></span>Oxide</button>
    </div>
</div>"##;

// ── Theme switcher logic + Bokeh runtime theming ─────────────────────────────

pub const THEME_SCRIPT: &str = r#"<script>
(function () {
    var trigger = document.getElementById('theme-switcher-trigger');
    var menu = document.getElementById('theme-switcher-menu');
    if (!trigger || !menu) return;
    var html = document.documentElement;

    function getHashParam(k){
        var h = window.location.hash;
        if (!h) return null;
        var ps = h.replace(/^#/,'').split('&');
        for (var i=0;i<ps.length;i++){
            var kv = ps[i].split('=');
            if (kv[0] === k) return decodeURIComponent(kv[1] || '');
        }
        return null;
    }
    function tryLS(fn){ try { return fn(); } catch(e){ return null; } }
    function currentMode()  { return getHashParam('mode')  || tryLS(function(){return localStorage.getItem('rtb-mode');})  || 'auto'; }
    function currentTheme() { return getHashParam('theme') || tryLS(function(){return localStorage.getItem('rtb-theme');}) || 'lab'; }

    function buildHash(theme, mode) {
        var p = [];
        if (theme) p.push('theme=' + encodeURIComponent(theme));
        if (mode && mode !== 'auto') p.push('mode=' + encodeURIComponent(mode));
        return p.length ? '#' + p.join('&') : '';
    }
    function syncLinksAndUrl(theme, mode) {
        var newHash = buildHash(theme, mode);
        if (history.replaceState) {
            history.replaceState(null, '', window.location.pathname + window.location.search + newHash);
        } else {
            window.location.hash = newHash.slice(1);
        }
        document.querySelectorAll('a[href]').forEach(function(a){
            var raw = a.getAttribute('href');
            if (!raw || raw.charAt(0) === '#') return;
            if (/^[a-z]+:\/\//i.test(raw) || raw.indexOf('mailto:') === 0) return;
            var base = raw.split('#')[0];
            if (!/\.html?$/i.test(base) && !/\.html?\?/i.test(base)) return;
            a.setAttribute('href', base + newHash);
        });
    }

    function syncActive() {
        var m = currentMode(), t = currentTheme();
        menu.querySelectorAll('[data-mode]').forEach(function(b){ b.classList.toggle('active', b.dataset.mode === m); });
        menu.querySelectorAll('[data-theme]').forEach(function(b){ b.classList.toggle('active', b.dataset.theme === t); });
    }

    function applyMode(mode) {
        tryLS(function(){ localStorage.setItem('rtb-mode', mode); });
        if (mode === 'auto') html.removeAttribute('data-mode');
        else html.setAttribute('data-mode', mode);
        syncLinksAndUrl(currentTheme(), mode);
        syncActive();
        applyBokehTheme();
    }
    function applyTheme(theme) {
        tryLS(function(){ localStorage.setItem('rtb-theme', theme); });
        html.setAttribute('data-theme', theme);
        syncLinksAndUrl(theme, currentMode());
        syncActive();
        applyBokehTheme();
    }

    if (!html.getAttribute('data-theme')) html.setAttribute('data-theme', currentTheme());
    syncLinksAndUrl(currentTheme(), currentMode());
    syncActive();

    function showMenu() {
        var r = trigger.getBoundingClientRect();
        menu.classList.add('open');
        menu.style.top = r.bottom + 'px';
        var vw = window.innerWidth;
        var mw = menu.offsetWidth;
        menu.style.left = Math.max(4, Math.min(r.left, vw - mw - 4)) + 'px';
    }
    function hideMenu() { menu.classList.remove('open'); }
    trigger.addEventListener('click', function(e){
        e.stopPropagation();
        if (menu.classList.contains('open')) hideMenu(); else showMenu();
    });
    document.addEventListener('click', function(e){
        if (!menu.contains(e.target) && e.target !== trigger) hideMenu();
    });
    menu.addEventListener('click', function(e){
        var b = e.target.closest('button.theme-option');
        if (!b) return;
        if (b.dataset.mode)  applyMode(b.dataset.mode);
        if (b.dataset.theme) applyTheme(b.dataset.theme);
    });

    if (window.matchMedia) {
        var mq = window.matchMedia('(prefers-color-scheme: dark)');
        var onChange = function(){ if (currentMode() === 'auto') applyBokehTheme(); };
        if (mq.addEventListener) mq.addEventListener('change', onChange);
        else if (mq.addListener) mq.addListener(onChange);
    }
})();

var SHADOW_THEME_CSS = `
    .noUi-target { background: var(--c-surface-2) !important; border: 1px solid var(--c-border) !important; box-shadow: none !important; }
    .noUi-connect, .noUi-connects > .noUi-connect { background: var(--c-accent) !important; }
    .noUi-handle { background: var(--c-surface) !important; border: 1px solid var(--c-border) !important; box-shadow: 0 1px 3px rgba(0,0,0,0.2) !important; }
    .noUi-handle:hover, .noUi-active { border-color: var(--c-accent) !important; }
    .noUi-handle::before, .noUi-handle::after { background: var(--c-fg-muted) !important; }
    .noUi-tooltip { background: var(--c-surface) !important; color: var(--c-fg) !important; border: 1px solid var(--c-border) !important; }
    .noUi-marker, .noUi-marker-large, .noUi-marker-sub { background: var(--c-border) !important; }
    .noUi-value, .noUi-value-sub { color: var(--c-fg-muted) !important; }
    .noUi-pips, .noUi-pips * { color: var(--c-fg-muted) !important; }

    .choices__inner { background-color: var(--c-surface) !important; border: 1px solid var(--c-border) !important; color: var(--c-fg) !important; }
    .choices__list--dropdown, .choices__list[aria-expanded] { background-color: var(--c-surface) !important; border: 1px solid var(--c-border) !important; color: var(--c-fg) !important; }
    .choices__item, .choices__item--choice { color: var(--c-fg) !important; }
    .choices__item--choice.is-highlighted, .choices__item--choice:hover, .choices__item--selectable.is-highlighted { background-color: var(--c-surface-2) !important; color: var(--c-fg) !important; }
    .choices__list--multiple .choices__item { background-color: var(--c-accent) !important; border-color: var(--c-accent) !important; color: var(--c-accent-fg) !important; }
    .choices__input { background-color: transparent !important; color: var(--c-fg) !important; border-color: var(--c-border) !important; }
    .choices[data-type*="select"]::after { border-color: var(--c-fg-muted) transparent transparent !important; }

    .bk-input, select.bk-input, textarea.bk-input { color: var(--c-fg) !important; background-color: var(--c-surface) !important; border-color: var(--c-border) !important; }
    .bk-input::placeholder { color: var(--c-fg-muted) !important; }
    select.bk-input option { background-color: var(--c-surface) !important; color: var(--c-fg) !important; }
    .bk-tooltip, div.bk-tooltip { background-color: var(--c-surface) !important; color: var(--c-fg) !important; border: 1px solid var(--c-border) !important; }
    .bk-btn, .bk-btn-default { color: var(--c-fg) !important; background-color: var(--c-surface) !important; border-color: var(--c-border) !important; }
`;

function injectShadowStyles(root) {
    var elements = (root || document).querySelectorAll('*');
    for (var i = 0; i < elements.length; i++) {
        var el = elements[i];
        if (el.shadowRoot && !el.__rtbStyled) {
            el.__rtbStyled = true;
            var s = document.createElement('style');
            s.textContent = SHADOW_THEME_CSS;
            el.shadowRoot.appendChild(s);
            injectShadowStyles(el.shadowRoot);
        }
    }
}

function applyBokehTheme() {
    if (!window.Bokeh || !Bokeh.documents || !Bokeh.documents.length) {
        return setTimeout(applyBokehTheme, 80);
    }
    injectShadowStyles();
    var cs = getComputedStyle(document.documentElement);
    function v(name){ return cs.getPropertyValue(name).trim() || null; }
    var T = {
        bg:      v('--c-surface'),
        border:  v('--c-border'),
        fg:      v('--c-fg'),
        fgDim:   v('--c-fg-dim'),
        fgMuted: v('--c-fg-muted'),
        grid:    v('--c-border-soft'),
        accent:  v('--c-accent'),
    };
    if (!T.bg) return;

    Bokeh.documents.forEach(function (doc) {
        var models = doc.all_models;
        var iter = (models && typeof models.forEach === 'function')
            ? function(fn){ models.forEach(fn); }
            : function(fn){ Object.keys(models).forEach(function(k){ fn(models[k], k); }); };
        iter(function (m) {
            if (!m || !m.type || typeof m.setv !== 'function') return;
            var t = m.type;
            try {
                if (t === 'Plot' || t === 'Figure') {
                    m.setv({
                        background_fill_color: T.bg,
                        border_fill_color: T.bg,
                        outline_line_color: T.border,
                    });
                } else if (/Axis$/.test(t)) {
                    m.setv({
                        axis_label_text_color: T.fgDim,
                        major_label_text_color: T.fgDim,
                        axis_line_color: T.border,
                        major_tick_line_color: T.border,
                        minor_tick_line_color: T.border,
                    });
                } else if (t === 'Grid') {
                    m.setv({ grid_line_color: T.grid, minor_grid_line_color: T.grid });
                } else if (t === 'Legend') {
                    m.setv({
                        background_fill_color: T.bg,
                        border_line_color: T.border,
                        label_text_color: T.fg,
                        title_text_color: T.fgDim,
                        inactive_fill_color: T.bg,
                    });
                } else if (t === 'Title') {
                    m.setv({ text_color: T.fg });
                } else if (t === 'ColorBar') {
                    m.setv({
                        background_fill_color: T.bg,
                        major_label_text_color: T.fgDim,
                        title_text_color: T.fg,
                        major_tick_line_color: T.border,
                        minor_tick_line_color: T.border,
                        bar_line_color: T.border,
                    });
                }
            } catch(e) {}
        });
    });
}
window.addEventListener('load', applyBokehTheme);
</script>"#;

pub const NAV_DROPDOWN_SCRIPT: &str = r#"    <script>
    (function () {
        function showMenu(menu, x, y) {
            clearTimeout(menu._ht);
            menu.style.left = x + 'px';
            menu.style.top  = y + 'px';
            menu.style.display = 'block';
            var vw = window.innerWidth;
            var mw = menu.offsetWidth;
            if (x + mw > vw) menu.style.left = Math.max(0, vw - mw) + 'px';
        }
        function hideMenu(menu) { menu._ht = setTimeout(function () { menu.style.display = 'none'; }, 150); }
        function keepOpen(menu) { clearTimeout(menu._ht); }
        function wire(trigger, menu, openRight) {
            trigger.addEventListener('mouseenter', function () {
                var r = trigger.getBoundingClientRect();
                showMenu(menu, openRight ? r.right : r.left, openRight ? r.top : r.bottom);
            });
            trigger.addEventListener('mouseleave', function () { hideMenu(menu); });
            menu.addEventListener('mouseenter', function () { keepOpen(menu); });
            menu.addEventListener('mouseleave', function () { hideMenu(menu); });
        }
        document.querySelectorAll('.nav-horizontal .nav-dd').forEach(function (dd) {
            var t = dd.querySelector(':scope > .nav-dd-trigger');
            var m = dd.querySelector(':scope > .nav-dd-menu');
            if (!t || !m) return;
            wire(t, m, false);
            t.addEventListener('click', function (e) {
                e.stopPropagation();
                if (m.style.display === 'block') { m.style.display = 'none'; } else { var r = t.getBoundingClientRect(); showMenu(m, r.left, r.bottom); }
            });
        });
        document.querySelectorAll('.nav-horizontal .nav-dd-sub').forEach(function (sub) {
            var t = sub.querySelector(':scope > .nav-dd-sub-trigger');
            var m = sub.querySelector(':scope > .nav-dd-sub-menu');
            if (!t || !m) return;
            wire(t, m, true);
        });
        document.addEventListener('click', function () {
            document.querySelectorAll('.nav-horizontal .nav-dd-menu, .nav-horizontal .nav-dd-sub-menu').forEach(function (m) { m.style.display = 'none'; });
        });
    })();
    (function () {
        var input = document.getElementById('nav-search-input');
        if (!input) return;
        var sidebar = document.querySelector('.nav-vertical');
        if (!sidebar) return;
        sidebar.querySelectorAll('details').forEach(function (d) {
            if (d.open) d.setAttribute('data-was-open', '');
        });
        input.addEventListener('input', function () {
            var q = this.value.trim().toLowerCase();
            var links = sidebar.querySelectorAll('a[href]');
            var details = sidebar.querySelectorAll('details');
            if (!q) {
                links.forEach(function (a) { a.style.display = ''; });
                details.forEach(function (d) { d.style.display = ''; d.open = d.hasAttribute('data-was-open'); });
                return;
            }
            links.forEach(function (a) { a.style.display = 'none'; });
            details.forEach(function (d) { d.style.display = 'none'; });
            links.forEach(function (a) {
                if (a.textContent.trim().toLowerCase().indexOf(q) !== -1) {
                    a.style.display = '';
                    var el = a.parentElement;
                    while (el && el !== sidebar) {
                        if (el.tagName === 'DETAILS') { el.style.display = ''; el.open = true; }
                        el = el.parentElement;
                    }
                }
            });
        });
    })();
    </script>"#;
