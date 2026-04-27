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
            --c-bg:        light-dark(oklch(98% 0.004 250), oklch(15% 0.015 255));
            --c-surface:   light-dark(oklch(100% 0 0),     oklch(19% 0.018 255));
            --c-surface-2: light-dark(oklch(96% 0.005 250),oklch(23% 0.018 255));
            --c-fg:        light-dark(oklch(20% 0.02 255), oklch(92% 0.01 250));
            --c-fg-dim:    light-dark(oklch(45% 0.015 255),oklch(62% 0.012 255));
            --c-fg-muted:  light-dark(oklch(60% 0.01 250), oklch(45% 0.012 255));
            --c-border:    light-dark(oklch(88% 0.008 255),oklch(28% 0.018 258));
            --c-border-soft: light-dark(oklch(93% 0.005 250), oklch(24% 0.015 258));

            /* Sidebar is always dark (lab-instrument convention). */
            --c-nav-bg:        oklch(13% 0.02 258);
            --c-nav-fg:        oklch(95% 0.005 250);
            --c-nav-fg-dim:    oklch(58% 0.015 255);
            --c-nav-fg-muted:  oklch(38% 0.018 258);
            --c-nav-border:    oklch(22% 0.022 258);
            --c-nav-hover-bg:  oklch(20% 0.022 258);
            --c-nav-active-bg: oklch(26% 0.025 258);

            /* One sharp accent (steel blue, calm). */
            --c-accent: oklch(62% 0.13 245);
            --c-accent-soft: light-dark(oklch(95% 0.04 245), oklch(30% 0.07 245));

            /* ── Dimensions ─────────────────────────────────────────────── */
            --r-sm: 2px;
            --r-md: 3px;
            --sidebar-w: 200px;
            --page-max:  1400px;
            --nav-h:     38px;
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
            background: oklch(20% 0.02 258); border: 1px solid var(--c-nav-border);
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
            color: var(--c-fg); margin: 0 0 var(--s-3);
            padding-bottom: var(--s-3); border-bottom: 1px solid var(--c-border);
            letter-spacing: -0.005em;
        }
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
