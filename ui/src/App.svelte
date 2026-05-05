<script>
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Chart, registerables } from 'chart.js/auto';

  Chart.register(...registerables);
  Chart.defaults.color = '#a98a7e';
  Chart.defaults.borderColor = '#2a2a2e';
  Chart.defaults.font.family = "'JetBrains Mono', monospace";
  Chart.defaults.font.size = 11;
  Chart.defaults.plugins.legend.labels.boxWidth = 12;
  Chart.defaults.plugins.legend.labels.padding = 16;
  Chart.defaults.animation.duration = 400;

  const CHART_COLORS = ['#e85d04', '#6b6459', '#3d3a35', '#8a8477', '#ffb596', '#c8c6c3'];

  let activeTab = $state('dashboard');
  let sessions = $state([]);
  let recommendations = $state([]);
  let tokenTrends = $state([]);
  let toolDist = $state([]);
  let costMetrics = $state(null);
  let dashboardStats = $state(null);
  let projects = $state([]);
  let loading = $state(true);
  let ingestResult = $state(null);

  let selectedProject = $state('');
  let selectedDays = $state(30);
  let isLight = $state(false);

  function toggleTheme() {
    isLight = !isLight;
    document.body.classList.toggle('light', isLight);
    const style = getComputedStyle(document.body);
    Chart.defaults.color = style.getPropertyValue('--outline').trim();
    Chart.defaults.borderColor = style.getPropertyValue('--border').trim();
  }

  let tokenTrendCanvas = $state();
  let costByModelCanvas = $state();
  let tokenAreaCanvas = $state();
  let toolBarCanvas = $state();
  let costTrendCanvas = $state();

  let tokenTrendChart = null;
  let costByModelChart = null;
  let tokenAreaChart = null;
  let toolBarChart = null;
  let costTrendChart = null;

  const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: 'dashboard' },
    { id: 'tokens', label: 'Tokens', icon: 'payments' },
    { id: 'tools', label: 'Tools', icon: 'construction' },
    { id: 'cost', label: 'Cost', icon: 'attach_money' },
    { id: 'sessions', label: 'Sessions', icon: 'history' },
    { id: 'recommendations', label: 'Tips', icon: 'lightbulb' },
  ];

  const dayOptions = [7, 14, 30, 90];

  let sectionTitle = $derived(navItems.find(n => n.id === activeTab)?.label?.toUpperCase() ?? '');
  let totalTokens = $derived(tokenTrends.reduce((s, t) => s + t.input_tokens + t.output_tokens, 0));
  let totalInputTokens = $derived(tokenTrends.reduce((s, t) => s + t.input_tokens, 0));
  let totalOutputTokens = $derived(tokenTrends.reduce((s, t) => s + t.output_tokens, 0));

  function sparklineHeights(trends, key, count = 10) {
    const dates = [...new Set(trends.map(t => t.date))].sort().slice(-count);
    if (dates.length === 0) return Array(count).fill(20);
    const vals = dates.map(d => trends.filter(t => t.date === d).reduce((s, t) => s + (t[key] || 0), 0));
    const max = Math.max(...vals, 1);
    return vals.map(v => Math.max(10, (v / max) * 100));
  }

  function sessionSparkline(count = 10) {
    const dates = [...new Set(sessions.map(s => s.started_at?.slice(0, 10)).filter(Boolean))].sort().slice(-count);
    if (dates.length === 0) return Array(count).fill(20);
    const vals = dates.map(d => sessions.filter(s => s.started_at?.slice(0, 10) === d).length);
    const max = Math.max(...vals, 1);
    return vals.map(v => Math.max(10, (v / max) * 100));
  }

  onMount(async () => {
    const projs = await invoke('get_projects').catch(() => []);
    projects = projs || [];
    await loadData();
  });

  $effect(() => {
    const tab = activeTab;
    const isLoading = loading;
    tick().then(() => {
      if (isLoading) return;
      if (tab === 'dashboard') renderDashboardCharts();
      else if (tab === 'tokens') renderTokenCharts();
      else if (tab === 'tools') renderToolCharts();
      else if (tab === 'cost') renderCostCharts();
    });
  });

  async function loadData() {
    loading = true;
    try {
      const projFilter = selectedProject || undefined;
      const results = await Promise.all([
        invoke('get_sessions', { limit: 100, days: selectedDays, project: projFilter }),
        invoke('get_recommendations'),
        invoke('get_token_trends', { days: selectedDays, project: projFilter }),
        invoke('get_tool_distribution', { days: selectedDays, project: projFilter }),
        invoke('get_cost_metrics', { days: selectedDays, project: projFilter }),
        invoke('get_dashboard_stats', { days: selectedDays, project: projFilter }),
      ]);
      sessions = results[0];
      recommendations = results[1];
      tokenTrends = results[2];
      toolDist = results[3];
      costMetrics = results[4];
      dashboardStats = results[5];
    } catch (e) {
      console.error('Failed to load data:', e);
    }
    loading = false;
  }

  function onFilterChange() { loadData(); }

  async function runIngestion() {
    try {
      ingestResult = await invoke('trigger_ingestion', { full: false });
      await loadData();
    } catch (e) {
      console.error('Ingestion failed:', e);
    }
  }

  function destroyChart(chart) {
    if (chart) chart.destroy();
    return null;
  }

  function renderDashboardCharts() {
    tokenTrendChart = destroyChart(tokenTrendChart);
    costByModelChart = destroyChart(costByModelChart);
    if (!tokenTrendCanvas || tokenTrends.length === 0) return;

    const dates = [...new Set(tokenTrends.map(t => t.date))].sort();
    const models = [...new Set(tokenTrends.map(t => t.model))];

    tokenTrendChart = new Chart(tokenTrendCanvas, {
      type: 'line',
      data: {
        labels: dates.map(d => d.slice(5)),
        datasets: models.flatMap((model, mi) => [
          {
            label: `${model} input`,
            data: dates.map(d => tokenTrends.find(t => t.date === d && t.model === model)?.input_tokens || 0),
            borderColor: CHART_COLORS[mi % CHART_COLORS.length],
            backgroundColor: CHART_COLORS[mi % CHART_COLORS.length] + '20',
            borderWidth: 1.5,
            pointRadius: 2,
            pointStyle: 'rect',
            tension: 0,
            fill: false,
          },
          {
            label: `${model} output`,
            data: dates.map(d => tokenTrends.find(t => t.date === d && t.model === model)?.output_tokens || 0),
            borderColor: CHART_COLORS[mi % CHART_COLORS.length],
            borderWidth: 1,
            borderDash: [4, 3],
            pointRadius: 1,
            tension: 0,
            fill: false,
          },
        ]),
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { position: 'bottom', labels: { font: { family: "'JetBrains Mono', monospace", size: 10 } } },
          tooltip: { callbacks: { label: ctx => `${ctx.dataset.label}: ${ctx.parsed.y.toLocaleString()}` } },
        },
        scales: {
          y: {
            ticks: { callback: v => v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e3 ? `${(v / 1e3).toFixed(0)}K` : v },
            grid: { color: '#2a2a2e' },
          },
          x: { grid: { display: false } },
        },
      },
    });

    if (!costByModelCanvas || !costMetrics?.by_model?.length) return;

    costByModelChart = new Chart(costByModelCanvas, {
      type: 'doughnut',
      data: {
        labels: costMetrics.by_model.map(m => m.model),
        datasets: [{
          data: costMetrics.by_model.map(m => m.total_cost_usd),
          backgroundColor: costMetrics.by_model.map((_, i) => CHART_COLORS[i % CHART_COLORS.length]),
          borderWidth: 0,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        cutout: '65%',
        plugins: {
          legend: { position: 'right', labels: { font: { family: "'JetBrains Mono', monospace", size: 10 } } },
          tooltip: { callbacks: { label: ctx => `${ctx.label}: $${ctx.parsed.toFixed(4)}` } },
        },
      },
    });
  }

  function renderTokenCharts() {
    tokenAreaChart = destroyChart(tokenAreaChart);
    if (!tokenAreaCanvas || tokenTrends.length === 0) return;

    const dates = [...new Set(tokenTrends.map(t => t.date))].sort();
    const models = [...new Set(tokenTrends.map(t => t.model))];

    tokenAreaChart = new Chart(tokenAreaCanvas, {
      type: 'line',
      data: {
        labels: dates.map(d => d.slice(5)),
        datasets: models.flatMap((model, mi) => [
          {
            label: `${model} input`,
            data: dates.map(d => tokenTrends.find(t => t.date === d && t.model === model)?.input_tokens || 0),
            borderColor: CHART_COLORS[mi % CHART_COLORS.length],
            backgroundColor: CHART_COLORS[mi % CHART_COLORS.length] + '25',
            borderWidth: 1.5,
            fill: true,
            tension: 0,
            pointRadius: 2,
            pointStyle: 'rect',
          },
          {
            label: `${model} output`,
            data: dates.map(d => tokenTrends.find(t => t.date === d && t.model === model)?.output_tokens || 0),
            borderColor: CHART_COLORS[(mi + 3) % CHART_COLORS.length],
            backgroundColor: CHART_COLORS[(mi + 3) % CHART_COLORS.length] + '25',
            borderWidth: 1.5,
            fill: true,
            tension: 0,
            pointRadius: 2,
            pointStyle: 'rect',
          },
        ]),
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { position: 'bottom', labels: { font: { family: "'JetBrains Mono', monospace", size: 10 } } },
          tooltip: { callbacks: { label: ctx => `${ctx.dataset.label}: ${ctx.parsed.y.toLocaleString()}` } },
        },
        scales: {
          y: {
            stacked: true,
            ticks: { callback: v => v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e3 ? `${(v / 1e3).toFixed(0)}K` : v },
            grid: { color: '#2a2a2e' },
          },
          x: { grid: { display: false } },
        },
      },
    });
  }

  function renderToolCharts() {
    toolBarChart = destroyChart(toolBarChart);
    if (!toolBarCanvas || toolDist.length === 0) return;

    toolBarChart = new Chart(toolBarCanvas, {
      type: 'bar',
      data: {
        labels: toolDist.map(t => t.tool_name),
        datasets: [
          {
            label: 'Calls',
            data: toolDist.map(t => t.call_count),
            backgroundColor: toolDist.map((_, i) => CHART_COLORS[i % CHART_COLORS.length] + 'cc'),
            borderColor: toolDist.map((_, i) => CHART_COLORS[i % CHART_COLORS.length]),
            borderWidth: 1,
            borderRadius: 0,
          },
          {
            label: 'Errors',
            data: toolDist.map(t => t.error_count),
            backgroundColor: '#dc2f0266',
            borderColor: '#dc2f02',
            borderWidth: 1,
            borderRadius: 0,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        indexAxis: 'y',
        plugins: { legend: { position: 'bottom', labels: { font: { family: "'JetBrains Mono', monospace", size: 10 } } } },
        scales: {
          x: { grid: { color: '#2a2a2e' }, ticks: { callback: v => v.toLocaleString() } },
          y: { grid: { display: false } },
        },
      },
    });
  }

  function renderCostCharts() {
    costTrendChart = destroyChart(costTrendChart);
    if (!costTrendCanvas || tokenTrends.length === 0) return;

    const dates = [...new Set(tokenTrends.map(t => t.date))].sort();
    const costByDate = dates.map(d => tokenTrends.filter(t => t.date === d).reduce((s, t) => s + t.total_cost_usd, 0));

    costTrendChart = new Chart(costTrendCanvas, {
      type: 'line',
      data: {
        labels: dates.map(d => d.slice(5)),
        datasets: [{
          label: 'Daily Cost ($)',
          data: costByDate,
          borderColor: '#e85d04',
          backgroundColor: 'rgba(232, 93, 4, 0.15)',
          borderWidth: 1.5,
          fill: true,
          tension: 0,
          pointRadius: 3,
          pointStyle: 'rect',
          pointBackgroundColor: '#e85d04',
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { display: false },
          tooltip: { callbacks: { label: ctx => `$${ctx.parsed.y.toFixed(4)}` } },
        },
        scales: {
          y: { ticks: { callback: v => `$${v.toFixed(2)}` }, grid: { color: '#2a2a2e' } },
          x: { grid: { display: false } },
        },
      },
    });
  }

  function formatCost(usd) {
    return `$${(usd ?? 0).toFixed(4)}`;
  }

  function formatDuration(ms) {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  function severityClass(sev) {
    return sev === 'Critical' ? 'critical' : sev === 'Warning' ? 'warning' : 'info';
  }
</script>

<div class="shell">
  <!-- Side Navigation Rail -->
  <aside class="side-nav">
    <div class="nav-logo">
      <span class="material-symbols-outlined logo-icon">speed</span>
      <span class="logo-text">CLAUDY</span>
    </div>
    <nav class="nav-items">
      {#each navItems as item}
        <button
          class="nav-btn"
          class:active={activeTab === item.id}
          onclick={() => activeTab = item.id}
        >
          <span class="material-symbols-outlined">{item.icon}</span>
          <span class="nav-label">{item.label}</span>
        </button>
      {/each}
    </nav>
    <div class="nav-bottom">
      <button class="nav-btn theme-toggle" onclick={toggleTheme} title="Toggle theme">
        <span class="material-symbols-outlined">{isLight ? 'dark_mode' : 'light_mode'}</span>
        <span class="nav-label">{isLight ? 'Dark' : 'Light'}</span>
      </button>
      <button class="nav-btn" onclick={runIngestion} title="Sync">
        <span class="material-symbols-outlined">sync</span>
        <span class="nav-label">Sync</span>
      </button>
    </div>
  </aside>

  <!-- Top App Bar -->
  <header class="top-bar">
    <div class="top-left">
      <span class="top-title">CLAUDY_{sectionTitle}</span>
      <div class="divider"></div>
      <div class="day-btns">
        {#each dayOptions as d}
          <button class:active={selectedDays === d} onclick={() => { selectedDays = d; onFilterChange(); }}>
            {d}d
          </button>
        {/each}
      </div>
      <div class="divider"></div>
      <select bind:value={selectedProject} onchange={onFilterChange}>
        <option value="">ALL PROJECTS</option>
        {#each projects as p}
          <option value={p.encoded_dir}>{p.display_name}</option>
        {/each}
      </select>
    </div>
    <div class="top-right">
      {#if ingestResult}
        <span class="ingest-badge">{ingestResult.files_ingested}/{ingestResult.files_scanned} files synced</span>
      {/if}
      <button class="icon-btn" onclick={runIngestion} title="Sync data">
        <span class="material-symbols-outlined">refresh</span>
      </button>
    </div>
  </header>

  <!-- Main Content -->
  <main class="content">
    {#if loading}
      <div class="loading">
        <div class="scan-line"></div>
        <span class="loading-text">LOADING DATA...</span>
      </div>
    {:else if activeTab === 'dashboard'}
      <div class="dashboard">
        <!-- Data Ribbon -->
        <div class="ribbon grid-line-container">
          <div class="ribbon-tile grid-line-item">
            <div class="tile-header">
              <span class="tile-label">TOTAL_TOKENS</span>
              <span class="tile-trend">+{selectedDays}d</span>
            </div>
            <div class="tile-value">{totalTokens >= 1e6 ? `${(totalTokens / 1e6).toFixed(1)}M` : totalTokens >= 1e3 ? `${(totalTokens / 1e3).toFixed(0)}K` : totalTokens.toLocaleString()}</div>
            <div class="sparkline">
              {#each sparklineHeights(tokenTrends, 'input_tokens') as h}
                <div class="bar" style="height: {h}%"></div>
              {/each}
            </div>
          </div>
          <div class="ribbon-tile grid-line-item">
            <div class="tile-header">
              <span class="tile-label">EST_COST</span>
              <span class="tile-trend">{formatCost(dashboardStats?.total_cost_usd ?? 0)}</span>
            </div>
            <div class="tile-value">{formatCost(dashboardStats?.total_cost_usd ?? 0)}</div>
            <div class="sparkline">
              {#each sparklineHeights(tokenTrends, 'total_cost_usd') as h}
                <div class="bar accent" style="height: {h}%"></div>
              {/each}
            </div>
          </div>
          <div class="ribbon-tile grid-line-item">
            <div class="tile-header">
              <span class="tile-label">TOOL_CALLS</span>
              <span class="tile-trend">{toolDist.reduce((s, t) => s + t.call_count, 0).toLocaleString()}</span>
            </div>
            <div class="tile-value">{toolDist.reduce((s, t) => s + t.call_count, 0).toLocaleString()}</div>
            <div class="sparkline">
              {#each sparklineHeights(tokenTrends, 'output_tokens') as h}
                <div class="bar" style="height: {h}%"></div>
              {/each}
            </div>
          </div>
          <div class="ribbon-tile grid-line-item">
            <div class="tile-header">
              <span class="tile-label">CACHE_HIT</span>
              <span class="tile-trend">{((dashboardStats?.cache_hit_ratio ?? 0) * 100).toFixed(1)}%</span>
            </div>
            <div class="tile-value">{((dashboardStats?.cache_hit_ratio ?? 0) * 100).toFixed(1)}%</div>
            <div class="sparkline">
              {#each sessionSparkline() as h}
                <div class="bar dim" style="height: {h}%"></div>
              {/each}
            </div>
          </div>
        </div>

        <!-- 60/40 Split -->
        <div class="split-row">
          <div class="split-left">
            <div class="panel">
              <div class="panel-header">
                <span class="panel-label">TOKEN_USAGE_TREND</span>
              </div>
              <div class="chart-wrap">
                <canvas bind:this={tokenTrendCanvas}></canvas>
              </div>
            </div>
          </div>
          <div class="split-right">
            <div class="panel">
              <div class="panel-header">
                <span class="panel-label">COST_BY_MODEL</span>
              </div>
              <div class="chart-wrap-sm">
                <canvas bind:this={costByModelCanvas}></canvas>
              </div>
            </div>
            {#if dashboardStats?.most_used_model}
              <div class="model-info">
                <span class="panel-label">PRIMARY_MODEL</span>
                <span class="model-name">{dashboardStats.most_used_model}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Session Ledger -->
        <section class="ledger-section">
          <div class="panel-header">
            <span class="panel-label">RECENT_SESSIONS</span>
            <span class="panel-count">{sessions.length} total</span>
          </div>
          <div class="ledger-wrap">
            <table class="ledger">
              <thead>
                <tr>
                  <th>SESSION_ID</th>
                  <th>PROJECT</th>
                  <th>MODEL</th>
                  <th>TURNS</th>
                  <th>DURATION</th>
                  <th>COST</th>
                  <th>STARTED</th>
                </tr>
              </thead>
              <tbody>
                {#each sessions.slice(0, 15) as s}
                  <tr>
                    <td class="mono">{s.session_uuid.slice(0, 12)}</td>
                    <td>{s.project_name || '-'}</td>
                    <td><span class="tag">{s.model || '-'}</span></td>
                    <td class="mono">{s.total_turns}</td>
                    <td class="mono">{formatDuration(s.total_duration_ms)}</td>
                    <td class="mono">{formatCost(s.total_cost_usd)}</td>
                    <td class="mono">{s.started_at?.slice(0, 16)?.replace('T', ' ') || '-'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>
      </div>

    {:else if activeTab === 'sessions'}
      <section class="page">
        <div class="page-header">
          <span class="panel-label">SESSION_HISTORY</span>
          <span class="panel-count">{sessions.length} records</span>
        </div>
        <div class="ledger-wrap full">
          <table class="ledger">
            <thead>
              <tr>
                <th>SESSION_ID</th>
                <th>PROJECT</th>
                <th>MODEL</th>
                <th>TURNS</th>
                <th>DURATION</th>
                <th>COST</th>
                <th>FIRST_MESSAGE</th>
                <th>STARTED</th>
              </tr>
            </thead>
            <tbody>
              {#each sessions as s}
                <tr>
                  <td class="mono">{s.session_uuid.slice(0, 12)}</td>
                  <td>{s.project_name || '-'}</td>
                  <td><span class="tag">{s.model || '-'}</span></td>
                  <td class="mono">{s.total_turns}</td>
                  <td class="mono">{formatDuration(s.total_duration_ms)}</td>
                  <td class="mono">{formatCost(s.total_cost_usd)}</td>
                  <td class="truncate">{s.first_message || '-'}</td>
                  <td class="mono">{s.started_at?.slice(0, 16)?.replace('T', ' ') || '-'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>

    {:else if activeTab === 'tokens'}
      <section class="page">
        <div class="page-header">
          <span class="panel-label">TOKEN_ANALYSIS</span>
          <span class="panel-count">{selectedDays}d period</span>
        </div>
        <div class="kpi-row grid-line-container">
          <div class="kpi-tile grid-line-item">
            <span class="tile-label">TOTAL_INPUT</span>
            <span class="kpi-value">{totalInputTokens.toLocaleString()}</span>
          </div>
          <div class="kpi-tile grid-line-item">
            <span class="tile-label">TOTAL_OUTPUT</span>
            <span class="kpi-value">{totalOutputTokens.toLocaleString()}</span>
          </div>
          <div class="kpi-tile grid-line-item">
            <span class="tile-label">TOTAL_COST</span>
            <span class="kpi-value">{formatCost(tokenTrends.reduce((s, t) => s + t.total_cost_usd, 0))}</span>
          </div>
          <div class="kpi-tile grid-line-item">
            <span class="tile-label">IN/OUT RATIO</span>
            <span class="kpi-value">{totalOutputTokens > 0 ? (totalInputTokens / totalOutputTokens).toFixed(1) + ':1' : '-'}</span>
          </div>
        </div>
        <div class="panel full-panel">
          <div class="panel-header">
            <span class="panel-label">INPUT_VS_OUTPUT_TOKENS</span>
          </div>
          <div class="chart-wrap-tall">
            <canvas bind:this={tokenAreaCanvas}></canvas>
          </div>
        </div>
        <div class="panel full-panel">
          <div class="panel-header">
            <span class="panel-label">DAILY_BREAKDOWN</span>
          </div>
          <div class="ledger-wrap">
            <table class="ledger">
              <thead>
                <tr>
                  <th>DATE</th>
                  <th>MODEL</th>
                  <th>INPUT</th>
                  <th>OUTPUT</th>
                  <th>COST</th>
                  <th>SESSIONS</th>
                </tr>
              </thead>
              <tbody>
                {#each tokenTrends as t}
                  <tr>
                    <td class="mono">{t.date}</td>
                    <td><span class="tag">{t.model}</span></td>
                    <td class="mono">{t.input_tokens.toLocaleString()}</td>
                    <td class="mono">{t.output_tokens.toLocaleString()}</td>
                    <td class="mono">{formatCost(t.total_cost_usd)}</td>
                    <td class="mono">{t.session_count}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      </section>

    {:else if activeTab === 'tools'}
      <section class="page">
        <div class="page-header">
          <span class="panel-label">TOOL_INSPECTOR</span>
          <span class="panel-count">{toolDist.length} tools tracked</span>
        </div>
        <div class="panel full-panel">
          <div class="panel-header">
            <span class="panel-label">CALL_COUNT_BY_TOOL</span>
          </div>
          <div class="chart-wrap-tall">
            <canvas bind:this={toolBarCanvas}></canvas>
          </div>
        </div>
        <div class="panel full-panel">
          <div class="panel-header">
            <span class="panel-label">TOOL_DETAILS</span>
          </div>
          <div class="tool-grid">
            {#each toolDist as tool}
              <div class="tool-card">
                <div class="tool-top">
                  <span class="tool-name">{tool.tool_name}</span>
                  <span class="tool-stat">{tool.call_count} calls ({tool.percentage.toFixed(1)}%)</span>
                </div>
                <div class="tool-meta">
                  <span>Avg: {formatDuration(tool.avg_duration_ms || 0)}</span>
                  <span class:error={tool.error_count > 0}>Errors: {tool.error_count}</span>
                </div>
                <div class="progress-track">
                  <div class="progress-fill" style="width: {tool.percentage}%"></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </section>

    {:else if activeTab === 'cost'}
      <section class="page">
        <div class="page-header">
          <span class="panel-label">COST_EFFICIENCY</span>
        </div>
        <div class="panel full-panel">
          <div class="panel-header">
            <span class="panel-label">DAILY_COST_TREND</span>
          </div>
          <div class="chart-wrap">
            <canvas bind:this={costTrendCanvas}></canvas>
          </div>
        </div>
        {#if costMetrics}
          <div class="kpi-row grid-line-container">
            <div class="kpi-tile grid-line-item">
              <span class="tile-label">TOTAL_COST_{selectedDays}d</span>
              <span class="kpi-value">{formatCost(costMetrics.total_cost_usd)}</span>
            </div>
            <div class="kpi-tile grid-line-item">
              <span class="tile-label">AVG_PER_SESSION</span>
              <span class="kpi-value">{formatCost(costMetrics.avg_cost_per_session)}</span>
            </div>
            <div class="kpi-tile grid-line-item">
              <span class="tile-label">AVG_PER_TURN</span>
              <span class="kpi-value">{formatCost(costMetrics.avg_cost_per_turn)}</span>
            </div>
            <div class="kpi-tile grid-line-item">
              <span class="tile-label">WEEKLY_AVG</span>
              <span class="kpi-value">{formatCost(costMetrics.weekly_avg_cost)}</span>
            </div>
          </div>

          {#if costMetrics.cache_savings_usd > 0}
            <div class="cache-card">
              <span class="cache-value">~{formatCost(costMetrics.cache_savings_usd)} saved</span>
              <span class="cache-label">ESTIMATED_CACHE_SAVINGS</span>
            </div>
          {/if}

          {#if costMetrics.most_expensive_session}
            <div class="panel">
              <div class="panel-header">
                <span class="panel-label">MOST_EXPENSIVE_SESSION</span>
              </div>
              <div class="expensive-row">
                <span class="tool-name">{costMetrics.most_expensive_session.project_name}</span>
                <span class="cost-highlight">{formatCost(costMetrics.most_expensive_session.cost_usd)}</span>
                <span class="meta">{costMetrics.most_expensive_session.turns} turns</span>
                <span class="tag">{costMetrics.most_expensive_session.model || '-'}</span>
                <span class="mono meta">{costMetrics.most_expensive_session.started_at?.slice(0, 10) || '-'}</span>
              </div>
            </div>
          {/if}

          {#if costMetrics.by_model.length > 0}
            <div class="panel full-panel">
              <div class="panel-header">
                <span class="panel-label">COST_BY_MODEL</span>
              </div>
              <div class="tool-grid">
                {#each costMetrics.by_model as m, i}
                  <div class="tool-card">
                    <div class="tool-top">
                      <span class="tool-name" style="color: {CHART_COLORS[i % CHART_COLORS.length]}">{m.model}</span>
                      <span class="tool-stat">{formatCost(m.total_cost_usd)} ({m.percentage.toFixed(1)}%)</span>
                    </div>
                    <div class="tool-meta">
                      <span>Sessions: {m.session_count}</span>
                      <span>Avg: {formatCost(m.avg_cost_per_session)}</span>
                      <span>Cache: {m.total_cache_read_tokens.toLocaleString()} tok</span>
                    </div>
                    <div class="progress-track">
                      <div class="progress-fill" style="width: {m.percentage}%; background: {CHART_COLORS[i % CHART_COLORS.length]}"></div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {/if}
      </section>

    {:else if activeTab === 'recommendations'}
      <section class="page">
        <div class="page-header">
          <span class="panel-label">RECOMMENDATIONS</span>
          <span class="panel-count">{recommendations.length} total</span>
        </div>
        {#if recommendations.length === 0}
          <div class="empty">NO_RECOMMENDATIONS_AVAILABLE. RUN_SYNC_FIRST.</div>
        {:else}
          {#each recommendations as rec}
            <div class="rec-card {severityClass(rec.severity)}">
              <div class="rec-header">
                <span class="rec-sev {severityClass(rec.severity)}">{rec.severity.toUpperCase()}</span>
                <span class="rec-cat">{rec.category}</span>
              </div>
              <strong class="rec-title">{rec.title}</strong>
              <p class="rec-desc">{rec.description}</p>
              {#if rec.action}
                <div class="rec-action">{rec.action}</div>
              {/if}
            </div>
          {/each}
        {/if}
      </section>
    {/if}
  </main>

  <!-- Status Bar -->
  <footer class="status-bar">
    <div class="status-left">
      <span class="status-dot"></span>
      <span class="status-text">API: NOMINAL</span>
      <span class="status-sep">|</span>
      <span class="status-text">SESSIONS: {sessions.length}</span>
      <span class="status-sep">|</span>
      <span class="status-text">PERIOD: {selectedDays}d</span>
    </div>
    <div class="status-right">
      <span class="status-text">SYNC_{ingestResult ? 'COMPLETE' : 'PENDING'}</span>
    </div>
  </footer>
</div>

<style>
  /* Shell */
  .shell {
    display: grid;
    grid-template-columns: var(--nav-width) 1fr;
    grid-template-rows: var(--top-bar-height) 1fr var(--status-bar-height);
    height: 100vh;
    overflow: hidden;
  }

  /* Side Nav */
  .side-nav {
    grid-row: 1 / -1;
    grid-column: 1;
    background: var(--surface-lowest);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 12px 0;
    z-index: 50;
  }

  .nav-logo {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 16px 16px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
  }

  .logo-icon {
    font-size: 22px;
    color: var(--accent);
    font-variation-settings: 'FILL' 1, 'wght' 500, 'GRAD' 0, 'opsz' 24;
  }

  .logo-text {
    font-family: var(--font-display);
    font-weight: 800;
    font-size: 14px;
    letter-spacing: 0.12em;
    color: var(--on-surface);
  }

  .nav-items {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 8px;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-left: 2px solid transparent;
    color: var(--outline-variant);
    cursor: pointer;
    transition: all 75ms ease-out;
    font-family: var(--font-display);
    width: 100%;
    text-align: left;
  }

  .nav-btn:hover {
    color: var(--outline);
    background: var(--accent-dim);
  }

  .nav-btn.active {
    color: var(--accent);
    border-left-color: var(--accent);
    background: var(--accent-dim);
    cursor: crosshair;
  }

  .nav-label {
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.02em;
  }

  .nav-bottom {
    margin-top: auto;
    border-top: 1px solid var(--border);
    padding-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-inline: 8px;
  }

  /* Top Bar */
  .top-bar {
    grid-row: 1;
    grid-column: 2;
    background: var(--surface-lowest);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 40;
  }

  .top-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .top-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .top-title {
    font-family: var(--font-data);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--on-surface);
    text-transform: uppercase;
  }

  .divider {
    width: 1px;
    height: 16px;
    background: var(--border);
  }

  .day-btns {
    display: flex;
    gap: 2px;
  }

  .day-btns button {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--outline-variant);
    padding: 2px 8px;
    font-family: var(--font-data);
    font-size: 10px;
    cursor: pointer;
    transition: all 75ms ease-out;
    border-radius: 0;
  }

  .day-btns button:hover { color: var(--outline); border-color: var(--border-hover); }
  .day-btns button.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  select {
    background: var(--surface-lowest);
    color: var(--on-surface);
    border: 1px solid var(--border);
    padding: 4px 8px;
    font-family: var(--font-data);
    font-size: 10px;
    cursor: pointer;
    border-radius: 0;
    min-width: 120px;
    text-transform: uppercase;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--outline-variant);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    transition: color 75ms ease-out;
  }

  .icon-btn:hover { color: var(--on-surface); }

  .ingest-badge {
    font-family: var(--font-data);
    font-size: 10px;
    color: var(--positive);
  }

  /* Main Content */
  .content {
    grid-row: 2;
    grid-column: 2;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .content::-webkit-scrollbar { width: 6px; }
  .content::-webkit-scrollbar-track { background: var(--bg); }
  .content::-webkit-scrollbar-thumb { background: var(--border); }

  /* Status Bar */
  .status-bar {
    grid-row: 3;
    grid-column: 2;
    background: var(--surface-lowest);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }

  .status-left, .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--positive);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .status-text {
    font-family: var(--font-data);
    font-size: 10px;
    color: var(--outline-variant);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-sep {
    color: var(--border);
    font-size: 10px;
  }

  /* Loading */
  .loading {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .scan-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
    opacity: 0.4;
    animation: scan 1.5s ease-in-out infinite;
  }

  @keyframes scan {
    0% { top: 0; }
    100% { top: 100%; }
  }

  .loading-text {
    font-family: var(--font-data);
    font-size: 12px;
    color: var(--outline-variant);
    letter-spacing: 0.1em;
    margin-top: 80px;
  }

  /* Grid-line technique */
  .grid-line-container {
    display: grid;
    gap: 1px;
    background-color: var(--border);
    border: 1px solid var(--border);
  }

  .grid-line-item {
    background-color: var(--bg);
  }

  /* Data Ribbon */
  .ribbon {
    grid-template-columns: repeat(4, 1fr);
  }

  .ribbon-tile {
    padding: 16px;
  }

  .tile-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .tile-label {
    font-family: var(--font-data);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.05em;
    color: var(--outline-variant);
    text-transform: uppercase;
  }

  .tile-trend {
    font-family: var(--font-data);
    font-size: 10px;
    color: var(--primary);
  }

  .tile-value {
    font-family: var(--font-data);
    font-size: 24px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    margin-bottom: 8px;
    font-variant-numeric: tabular-nums;
  }

  .sparkline {
    height: 28px;
    display: flex;
    align-items: flex-end;
    gap: 2px;
  }

  .sparkline .bar {
    flex: 1;
    background: rgba(255, 181, 150, 0.3);
    min-height: 2px;
    transition: height 300ms ease-out;
  }

  .sparkline .bar.accent { background: rgba(232, 93, 4, 0.4); }
  .sparkline .bar.dim { background: rgba(255, 181, 150, 0.15); }

  /* 60/40 Split */
  .split-row {
    display: grid;
    grid-template-columns: 6fr 4fr;
    gap: 1px;
    background-color: var(--border);
    border: 1px solid var(--border);
    margin-top: 1px;
  }

  .split-left { background: var(--bg); }
  .split-right {
    background: var(--bg);
    display: flex;
    flex-direction: column;
  }

  .panel {
    background: var(--bg);
    border: 1px solid var(--border);
    margin-top: 4px;
  }

  .full-panel {
    margin-top: 4px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px 8px;
  }

  .panel-label {
    font-family: var(--font-data);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.05em;
    color: var(--outline-variant);
    text-transform: uppercase;
  }

  .panel-count {
    font-family: var(--font-data);
    font-size: 10px;
    color: var(--outline-variant);
  }

  .chart-wrap {
    position: relative;
    height: 240px;
    padding: 0 16px 16px;
  }

  .chart-wrap-sm {
    position: relative;
    height: 200px;
    padding: 0 16px 16px;
  }

  .chart-wrap-tall {
    position: relative;
    height: 320px;
    padding: 0 16px 16px;
  }

  .model-info {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .model-name {
    font-family: var(--font-data);
    font-size: 14px;
    font-weight: 700;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }

  /* Page layouts */
  .page {
    padding: 16px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  /* KPI Row */
  .kpi-row {
    grid-template-columns: repeat(4, 1fr);
    margin-bottom: 4px;
  }

  .kpi-tile {
    padding: 16px;
  }

  .kpi-value {
    display: block;
    font-family: var(--font-data);
    font-size: 20px;
    font-weight: 700;
    color: var(--on-surface);
    margin-top: 4px;
    font-variant-numeric: tabular-nums;
  }

  /* Ledger Table */
  .ledger-section {
    margin-top: 4px;
  }

  .ledger-wrap {
    overflow-x: auto;
  }

  .ledger-wrap.full {
    max-height: calc(100vh - var(--top-bar-height) - var(--status-bar-height) - 80px);
    overflow-y: auto;
  }

  table.ledger {
    width: 100%;
    border-collapse: collapse;
  }

  table.ledger th {
    text-align: left;
    padding: 8px 12px;
    font-family: var(--font-data);
    font-size: 11px;
    font-weight: 500;
    color: var(--outline-variant);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--bg);
    z-index: 1;
  }

  table.ledger td {
    padding: 6px 12px;
    font-family: var(--font-display);
    font-size: 13px;
    border-bottom: 1px solid rgba(42, 42, 46, 0.4);
    color: var(--on-surface-variant);
  }

  table.ledger tr:hover td {
    background: var(--accent-dim);
    cursor: crosshair;
  }

  table.ledger tr:nth-child(even) td {
    background: rgba(15, 14, 10, 0.3);
  }

  table.ledger tr:nth-child(even):hover td {
    background: var(--accent-dim);
  }

  .mono {
    font-family: var(--font-data);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }

  .truncate {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    display: inline-block;
    background: rgba(232, 93, 4, 0.1);
    border: 1px solid rgba(232, 93, 4, 0.3);
    padding: 1px 6px;
    font-family: var(--font-data);
    font-size: 11px;
    color: var(--accent);
    border-radius: 0;
    font-variant-numeric: tabular-nums;
  }

  /* Tool Grid */
  .tool-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1px;
    background-color: var(--border);
    border: 1px solid var(--border);
  }

  .tool-card {
    background: var(--bg);
    padding: 16px;
  }

  .tool-top {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 8px;
  }

  .tool-name {
    font-weight: 600;
    color: var(--accent);
    font-family: var(--font-display);
    font-size: 14px;
  }

  .tool-stat {
    font-family: var(--font-data);
    font-size: 11px;
    color: var(--outline);
  }

  .tool-meta {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-data);
    font-size: 11px;
    color: var(--outline-variant);
    margin-bottom: 10px;
  }

  .tool-meta .error { color: var(--critical); }

  .progress-track {
    height: 4px;
    background: var(--border);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 300ms ease-out;
  }

  /* Cache Card */
  .cache-card {
    background: rgba(96, 168, 64, 0.06);
    border: 1px solid rgba(96, 168, 64, 0.2);
    padding: 16px;
    text-align: center;
    margin-top: 4px;
  }

  .cache-value {
    display: block;
    font-family: var(--font-data);
    font-size: 20px;
    font-weight: 700;
    color: var(--positive);
    font-variant-numeric: tabular-nums;
  }

  .cache-label {
    display: block;
    font-family: var(--font-data);
    font-size: 11px;
    color: var(--outline-variant);
    margin-top: 4px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  /* Expensive Session */
  .expensive-row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
  }

  .cost-highlight {
    font-family: var(--font-data);
    font-size: 16px;
    font-weight: 700;
    color: var(--critical);
    font-variant-numeric: tabular-nums;
  }

  .meta {
    font-family: var(--font-data);
    font-size: 11px;
    color: var(--outline-variant);
  }

  /* Recommendation Cards */
  .rec-card {
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 14px 18px;
    margin-bottom: 1px;
    transition: background 75ms ease-out;
  }

  .rec-card:hover { background: rgba(20, 19, 15, 0.8); }
  .rec-card.critical { border-left: 2px solid var(--critical); }
  .rec-card.warning { border-left: 2px solid var(--warning); }
  .rec-card.info { border-left: 2px solid var(--border); }

  .rec-header {
    display: flex;
    gap: 8px;
    margin-bottom: 6px;
  }

  .rec-sev {
    font-family: var(--font-data);
    font-size: 10px;
    letter-spacing: 0.05em;
  }

  .rec-sev.critical { color: var(--critical); }
  .rec-sev.warning { color: var(--warning); }
  .rec-sev.info { color: #555; }

  .rec-cat {
    font-family: var(--font-data);
    font-size: 10px;
    color: var(--outline-variant);
    letter-spacing: 0.05em;
  }

  .rec-title {
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 600;
    color: var(--on-surface);
  }

  .rec-desc {
    font-family: var(--font-display);
    font-size: 13px;
    color: var(--outline);
    margin-top: 4px;
    line-height: 1.4;
  }

  .rec-action {
    font-family: var(--font-data);
    font-size: 12px;
    color: var(--accent);
    margin-top: 8px;
  }

  .empty {
    font-family: var(--font-data);
    font-size: 12px;
    color: var(--border);
    letter-spacing: 0.05em;
    text-align: center;
    padding: 80px 0;
  }
</style>
