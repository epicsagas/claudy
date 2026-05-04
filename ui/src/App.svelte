<script>
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Chart, registerables } from 'chart.js/auto';

  Chart.register(...registerables);
  Chart.defaults.color = '#888';
  Chart.defaults.borderColor = '#222';
  Chart.defaults.font.family = "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif";
  Chart.defaults.font.size = 11;
  Chart.defaults.plugins.legend.labels.boxWidth = 12;
  Chart.defaults.plugins.legend.labels.padding = 16;
  Chart.defaults.animation.duration = 400;

  const COLORS = ['#7c6ff7', '#5cdb5c', '#f0a030', '#4080e0', '#e04040', '#c084fc', '#22d3ee', '#f472b6'];

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

  // Filters
  let selectedProject = $state('');
  let selectedDays = $state(30);

  // Chart canvas refs
  let tokenTrendCanvas = $state();
  let costByModelCanvas = $state();
  let tokenAreaCanvas = $state();
  let toolBarCanvas = $state();
  let costTrendCanvas = $state();

  // Chart instances
  let tokenTrendChart = null;
  let costByModelChart = null;
  let tokenAreaChart = null;
  let toolBarChart = null;
  let costTrendChart = null;

  const tabs = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'tokens', label: 'Tokens' },
    { id: 'tools', label: 'Tools' },
    { id: 'cost', label: 'Cost' },
    { id: 'sessions', label: 'Sessions' },
    { id: 'recommendations', label: 'Tips' },
  ];

  const dayOptions = [7, 14, 30, 90];

  onMount(async () => {
    const projs = await invoke('get_projects').catch(() => []);
    projects = projs || [];
    await loadData();
  });

  $effect(() => {
    const tab = activeTab;
    const trends = tokenTrends;
    const tools = toolDist;
    const cost = costMetrics;
    const isLoading = loading;

    tick().then(() => {
      if (isLoading) return;
      if (tab === 'dashboard') {
        renderDashboardCharts();
      } else if (tab === 'tokens') {
        renderTokenCharts();
      } else if (tab === 'tools') {
        renderToolCharts();
      } else if (tab === 'cost') {
        renderCostCharts();
      }
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

  function onFilterChange() {
    loadData();
  }

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
            data: dates.map(d => {
              const pt = tokenTrends.find(t => t.date === d && t.model === model);
              return pt ? pt.input_tokens : 0;
            }),
            borderColor: COLORS[mi % COLORS.length],
            backgroundColor: COLORS[mi % COLORS.length] + '20',
            borderWidth: 2,
            pointRadius: 2,
            tension: 0.3,
            fill: false,
          },
          {
            label: `${model} output`,
            data: dates.map(d => {
              const pt = tokenTrends.find(t => t.date === d && t.model === model);
              return pt ? pt.output_tokens : 0;
            }),
            borderColor: COLORS[mi % COLORS.length],
            borderWidth: 1.5,
            borderDash: [4, 3],
            pointRadius: 1,
            tension: 0.3,
            fill: false,
          },
        ]),
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { position: 'bottom' },
          tooltip: {
            callbacks: {
              label: ctx => `${ctx.dataset.label}: ${ctx.parsed.y.toLocaleString()}`,
            },
          },
        },
        scales: {
          y: {
            ticks: { callback: v => v >= 1000000 ? `${(v / 1000000).toFixed(1)}M` : v >= 1000 ? `${(v / 1000).toFixed(0)}K` : v },
            grid: { color: '#1a1a1a' },
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
          backgroundColor: costMetrics.by_model.map((_, i) => COLORS[i % COLORS.length]),
          borderWidth: 0,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        cutout: '65%',
        plugins: {
          legend: { position: 'right' },
          tooltip: {
            callbacks: {
              label: ctx => `${ctx.label}: $${ctx.parsed.toFixed(4)}`,
            },
          },
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
            borderColor: COLORS[mi % COLORS.length],
            backgroundColor: COLORS[mi % COLORS.length] + '25',
            borderWidth: 2,
            fill: true,
            tension: 0.3,
            pointRadius: 2,
          },
          {
            label: `${model} output`,
            data: dates.map(d => tokenTrends.find(t => t.date === d && t.model === model)?.output_tokens || 0),
            borderColor: COLORS[(mi + 4) % COLORS.length],
            backgroundColor: COLORS[(mi + 4) % COLORS.length] + '25',
            borderWidth: 2,
            fill: true,
            tension: 0.3,
            pointRadius: 2,
          },
        ]),
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { position: 'bottom' },
          tooltip: {
            callbacks: {
              label: ctx => `${ctx.dataset.label}: ${ctx.parsed.y.toLocaleString()}`,
            },
          },
        },
        scales: {
          y: {
            stacked: true,
            ticks: { callback: v => v >= 1000000 ? `${(v / 1000000).toFixed(1)}M` : v >= 1000 ? `${(v / 1000).toFixed(0)}K` : v },
            grid: { color: '#1a1a1a' },
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
            backgroundColor: toolDist.map((_, i) => COLORS[i % COLORS.length] + 'cc'),
            borderColor: toolDist.map((_, i) => COLORS[i % COLORS.length]),
            borderWidth: 1,
            borderRadius: 4,
          },
          {
            label: 'Errors',
            data: toolDist.map(t => t.error_count),
            backgroundColor: '#e0404066',
            borderColor: '#e04040',
            borderWidth: 1,
            borderRadius: 4,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        indexAxis: 'y',
        plugins: {
          legend: { position: 'bottom' },
        },
        scales: {
          x: { grid: { color: '#1a1a1a' }, ticks: { callback: v => v.toLocaleString() } },
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
          borderColor: '#7c6ff7',
          backgroundColor: '#7c6ff730',
          borderWidth: 2,
          fill: true,
          tension: 0.3,
          pointRadius: 3,
          pointBackgroundColor: '#7c6ff7',
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: {
          legend: { display: false },
          tooltip: {
            callbacks: {
              label: ctx => `$${ctx.parsed.y.toFixed(4)}`,
            },
          },
        },
        scales: {
          y: {
            ticks: { callback: v => `$${v.toFixed(2)}` },
            grid: { color: '#1a1a1a' },
          },
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

<main>
  <header>
    <h1>Claudy Analytics</h1>
    <nav>
      {#each tabs as tab}
        <button class:active={activeTab === tab.id} onclick={() => activeTab = tab.id}>
          {tab.label}
        </button>
      {/each}
      <button class="ingest" onclick={runIngestion}>Sync</button>
    </nav>
  </header>

  <!-- Filters -->
  <div class="filters">
    <div class="filter-group">
      <label for="project-filter">Project</label>
      <select id="project-filter" bind:value={selectedProject} onchange={onFilterChange}>
        <option value="">All Projects</option>
        {#each projects as p}
          <option value={p.encoded_dir}>{p.display_name}</option>
        {/each}
      </select>
    </div>
    <div class="filter-group">
      <span class="filter-label">Period</span>
      <div class="day-btns">
        {#each dayOptions as d}
          <button class:active={selectedDays === d} onclick={() => { selectedDays = d; onFilterChange(); }}>
            {d}d
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if activeTab === 'dashboard'}
    <div class="dashboard">
      <div class="stats-grid">
        <div class="stat-card">
          <span class="stat-value">{dashboardStats?.total_sessions ?? 0}</span>
          <span class="stat-label">Sessions</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{formatCost(dashboardStats?.total_cost_usd ?? 0)}</span>
          <span class="stat-label">Total Cost</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{Math.round(dashboardStats?.avg_tokens_per_session ?? 0).toLocaleString()}</span>
          <span class="stat-label">Avg Tokens/Session</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{((dashboardStats?.cache_hit_ratio ?? 0) * 100).toFixed(1)}%</span>
          <span class="stat-label">Cache Hit Ratio</span>
        </div>
      </div>

      {#if dashboardStats?.most_used_model}
        <div class="model-badge">Primary: {dashboardStats.most_used_model}</div>
      {/if}

      {#if ingestResult}
        <div class="ingest-result">
          Synced {ingestResult.files_ingested}/{ingestResult.files_scanned} files
          ({ingestResult.sessions_created} sessions, {ingestResult.turns_created} turns)
          in {ingestResult.elapsed_ms}ms
        </div>
      {/if}

      <div class="charts-row">
        <div class="chart-panel">
          <h3>Token Trends</h3>
          <div class="chart-wrap">
            <canvas bind:this={tokenTrendCanvas}></canvas>
          </div>
        </div>
        <div class="chart-panel chart-panel--sm">
          <h3>Cost by Model</h3>
          <div class="chart-wrap">
            <canvas bind:this={costByModelCanvas}></canvas>
          </div>
        </div>
      </div>

      {#if recommendations.length > 0}
        <section class="recommendations">
          <h2>Recommendations ({recommendations.filter(r => r.severity === 'Critical' || r.severity === 'Warning').length} alerts)</h2>
          {#each recommendations.slice(0, 5) as rec}
            <div class="rec-card {severityClass(rec.severity)}">
              <strong>{rec.title}</strong>
              <p>{rec.description}</p>
              {#if rec.action}
                <em class="rec-action">{rec.action}</em>
              {/if}
            </div>
          {/each}
        </section>
      {/if}

      <section class="recent-sessions">
        <h2>Recent Sessions</h2>
        <table>
          <thead>
            <tr>
              <th>Project</th>
              <th>Model</th>
              <th>Turns</th>
              <th>Duration</th>
              <th>Cost</th>
              <th>Started</th>
            </tr>
          </thead>
          <tbody>
            {#each sessions.slice(0, 15) as s}
              <tr>
                <td>{s.project_name || s.session_uuid.slice(0, 8)}</td>
                <td><span class="model-tag">{s.model || '-'}</span></td>
                <td>{s.total_turns}</td>
                <td>{formatDuration(s.total_duration_ms)}</td>
                <td>{formatCost(s.total_cost_usd)}</td>
                <td>{s.started_at?.slice(0, 10) || '-'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </section>
    </div>
  {:else if activeTab === 'sessions'}
    <section class="sessions-list">
      <h2>All Sessions ({sessions.length})</h2>
      <table>
        <thead>
          <tr>
            <th>UUID</th>
            <th>Model</th>
            <th>Turns</th>
            <th>Duration</th>
            <th>Cost</th>
            <th>First Message</th>
            <th>Started</th>
          </tr>
        </thead>
        <tbody>
          {#each sessions as s}
            <tr>
              <td class="mono">{s.session_uuid.slice(0, 12)}</td>
              <td><span class="model-tag">{s.model || '-'}</span></td>
              <td>{s.total_turns}</td>
              <td>{formatDuration(s.total_duration_ms)}</td>
              <td>{formatCost(s.total_cost_usd)}</td>
              <td class="truncate">{s.first_message || '-'}</td>
              <td>{s.started_at || '-'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {:else if activeTab === 'tokens'}
    <section class="tokens-page">
      <h2>Token Usage ({selectedDays}d)</h2>
      <div class="chart-panel chart-panel--full">
        <h3>Input vs Output Tokens</h3>
        <div class="chart-wrap chart-wrap--tall">
          <canvas bind:this={tokenAreaCanvas}></canvas>
        </div>
      </div>

      <div class="stats-grid" style="margin-top: 20px;">
        <div class="stat-card">
          <span class="stat-value">{tokenTrends.reduce((s, t) => s + t.input_tokens, 0).toLocaleString()}</span>
          <span class="stat-label">Total Input</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{tokenTrends.reduce((s, t) => s + t.output_tokens, 0).toLocaleString()}</span>
          <span class="stat-label">Total Output</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{formatCost(tokenTrends.reduce((s, t) => s + t.total_cost_usd, 0))}</span>
          <span class="stat-label">Total Cost</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{(() => {
            const inp = tokenTrends.reduce((s, t) => s + t.input_tokens, 0);
            const out = tokenTrends.reduce((s, t) => s + t.output_tokens, 0);
            return out > 0 ? (inp / out).toFixed(1) + ':1' : '-';
          })()}</span>
          <span class="stat-label">Input/Output Ratio</span>
        </div>
      </div>

      <h3 style="margin-top: 24px;">Daily Breakdown</h3>
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Model</th>
            <th>Input</th>
            <th>Output</th>
            <th>Cost</th>
            <th>Sessions</th>
          </tr>
        </thead>
        <tbody>
          {#each tokenTrends as t}
            <tr>
              <td>{t.date}</td>
              <td><span class="model-tag">{t.model}</span></td>
              <td>{t.input_tokens.toLocaleString()}</td>
              <td>{t.output_tokens.toLocaleString()}</td>
              <td>{formatCost(t.total_cost_usd)}</td>
              <td>{t.session_count}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {:else if activeTab === 'tools'}
    <section class="tools-page">
      <h2>Tool Distribution</h2>
      <div class="chart-panel chart-panel--full">
        <h3>Call Count by Tool</h3>
        <div class="chart-wrap chart-wrap--tall">
          <canvas bind:this={toolBarCanvas}></canvas>
        </div>
      </div>

      <h3 style="margin-top: 24px;">Details</h3>
      <div class="tool-list">
        {#each toolDist as tool}
          <div class="stat-card tool-card">
            <div class="tool-info">
              <span class="tool-name">{tool.tool_name}</span>
              <span class="tool-usage">{tool.call_count} calls ({tool.percentage.toFixed(1)}%)</span>
            </div>
            <div class="tool-meta">
              <span>Avg: {formatDuration(tool.avg_duration_ms || 0)}</span>
              <span style="color: {tool.error_count > 0 ? '#e04040' : '#666'}">Errors: {tool.error_count}</span>
            </div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: {tool.percentage}%"></div>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {:else if activeTab === 'cost'}
    <section class="cost-page">
      <h2>Cost Efficiency</h2>
      <div class="chart-panel chart-panel--full">
        <h3>Daily Cost Trend</h3>
        <div class="chart-wrap">
          <canvas bind:this={costTrendCanvas}></canvas>
        </div>
      </div>

      {#if costMetrics}
        <div class="stats-grid" style="margin-top: 20px;">
          <div class="stat-card">
            <span class="stat-value">{formatCost(costMetrics.total_cost_usd)}</span>
            <span class="stat-label">Total Cost ({selectedDays}d)</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{formatCost(costMetrics.avg_cost_per_session)}</span>
            <span class="stat-label">Avg per Session</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{formatCost(costMetrics.avg_cost_per_turn)}</span>
            <span class="stat-label">Avg per Turn</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{formatCost(costMetrics.weekly_avg_cost)}</span>
            <span class="stat-label">Weekly Avg (7d)</span>
          </div>
        </div>

        {#if costMetrics.cache_savings_usd > 0}
          <div class="cache-savings-card">
            <span class="savings-value">~{formatCost(costMetrics.cache_savings_usd)} saved</span>
            <span class="savings-label">Estimated cache savings in period</span>
          </div>
        {/if}

        {#if costMetrics.most_expensive_session}
          <div class="expensive-session">
            <h3>Most Expensive Session</h3>
            <div class="stat-card" style="text-align: left;">
              <div class="tool-info">
                <span class="tool-name">{costMetrics.most_expensive_session.project_name}</span>
                <span class="tool-usage" style="color: #e04040; font-weight: 600;">{formatCost(costMetrics.most_expensive_session.cost_usd)}</span>
              </div>
              <div class="tool-meta">
                <span>{costMetrics.most_expensive_session.turns} turns</span>
                <span class="model-tag">{costMetrics.most_expensive_session.model || '-'}</span>
                <span>{costMetrics.most_expensive_session.started_at?.slice(0, 10) || '-'}</span>
              </div>
            </div>
          </div>
        {/if}

        {#if costMetrics.by_model.length > 0}
          <div class="model-breakdown">
            <h3>Cost by Model</h3>
            <div class="tool-list">
              {#each costMetrics.by_model as m, i}
                <div class="stat-card tool-card">
                  <div class="tool-info">
                    <span class="tool-name" style:color={COLORS[i % COLORS.length]}>{m.model}</span>
                    <span class="tool-usage">{formatCost(m.total_cost_usd)} ({m.percentage.toFixed(1)}%)</span>
                  </div>
                  <div class="tool-meta">
                    <span>Sessions: {m.session_count}</span>
                    <span>Avg: {formatCost(m.avg_cost_per_session)}</span>
                    <span>Cache read: {m.total_cache_read_tokens.toLocaleString()} tok</span>
                  </div>
                  <div class="progress-bar">
                    <div class="progress-fill" style="width: {m.percentage}%; background: {COLORS[i % COLORS.length]}"></div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
      <p class="cost-note">Metrics based on your usage over the last {selectedDays} days.</p>
    </section>
  {:else if activeTab === 'recommendations'}
    <section class="recommendations-page">
      <h2>Recommendations ({recommendations.length})</h2>
      {#if recommendations.length === 0}
        <p class="placeholder">No recommendations yet. Run a sync first.</p>
      {:else}
        {#each recommendations as rec}
          <div class="rec-card {severityClass(rec.severity)}">
            <div class="rec-header">
              <span class="rec-severity">{rec.severity}</span>
              <span class="rec-category">{rec.category}</span>
            </div>
            <strong>{rec.title}</strong>
            <p>{rec.description}</p>
            {#if rec.action}
              <div class="rec-action">{rec.action}</div>
            {/if}
          </div>
        {/each}
      {/if}
    </section>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    background: #0a0a0a;
    color: #e0e0e0;
  }

  main { max-width: 1280px; margin: 0 auto; padding: 20px; }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    border-bottom: 1px solid #2a2a2a;
    padding-bottom: 12px;
  }

  h1 { font-size: 1.4rem; color: #7c6ff7; margin: 0; }
  h2 { font-size: 1.1rem; color: #ccc; margin-bottom: 12px; }
  h3 { font-size: 0.95rem; color: #aaa; margin: 0 0 8px; }

  nav { display: flex; gap: 4px; }

  button {
    background: #1a1a1a;
    color: #aaa;
    border: 1px solid #333;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.15s;
  }
  button:hover { background: #252525; color: #ddd; }
  button.active { background: #7c6ff7; color: white; border-color: #7c6ff7; }
  button.ingest { background: #1a3a1a; border-color: #2a5a2a; color: #5cdb5c; }

  .filters {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 12px 16px;
    background: #111;
    border: 1px solid #222;
    border-radius: 10px;
    margin-bottom: 20px;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-group label {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: #666;
    letter-spacing: 0.5px;
  }

  select {
    background: #1a1a1a;
    color: #ccc;
    border: 1px solid #333;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    min-width: 160px;
  }
  select:hover { border-color: #444; }

  .day-btns { display: flex; gap: 4px; }
  .day-btns button { padding: 4px 12px; font-size: 0.8rem; }

  .loading { text-align: center; padding: 60px; color: #666; }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }
  .stat-card {
    background: #141414;
    border: 1px solid #222;
    border-radius: 10px;
    padding: 20px;
    text-align: center;
  }
  .stat-value { display: block; font-size: 1.8rem; font-weight: 700; color: #7c6ff7; }
  .stat-label { display: block; font-size: 0.8rem; color: #666; margin-top: 4px; }

  .model-badge {
    display: inline-block;
    background: #1a1a2a;
    border: 1px solid #2a2a44;
    padding: 4px 12px;
    border-radius: 6px;
    font-size: 0.8rem;
    color: #9c8ff7;
    margin-bottom: 16px;
  }

  .ingest-result {
    background: #1a2a1a;
    border: 1px solid #2a4a2a;
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 20px;
    font-size: 0.85rem;
    color: #5cdb5c;
  }

  .charts-row {
    display: grid;
    grid-template-columns: 1fr 340px;
    gap: 16px;
    margin-bottom: 24px;
  }

  .chart-panel {
    background: #141414;
    border: 1px solid #222;
    border-radius: 10px;
    padding: 16px;
  }

  .chart-panel--sm { min-width: 0; }
  .chart-panel--full { margin-bottom: 20px; }

  .chart-wrap {
    position: relative;
    height: 240px;
  }

  .chart-wrap--tall {
    height: 320px;
  }

  section { margin-bottom: 24px; }

  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; padding: 8px 12px; font-size: 0.75rem; color: #666; text-transform: uppercase; border-bottom: 1px solid #222; }
  td { padding: 8px 12px; font-size: 0.85rem; border-bottom: 1px solid #1a1a1a; }
  tr:hover td { background: #111; }

  .mono { font-family: 'SF Mono', 'Menlo', monospace; font-size: 0.8rem; }
  .truncate { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .model-tag {
    display: inline-block;
    background: #1a1a2a;
    border: 1px solid #2a2a44;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.78rem;
    color: #9c8ff7;
  }

  .rec-card {
    background: #141414;
    border: 1px solid #222;
    border-radius: 8px;
    padding: 14px 18px;
    margin-bottom: 10px;
  }
  .rec-card.warning { border-left: 3px solid #f0a030; }
  .rec-card.critical { border-left: 3px solid #e04040; }
  .rec-card.info { border-left: 3px solid #4080e0; }
  .rec-header { display: flex; gap: 8px; margin-bottom: 6px; font-size: 0.7rem; text-transform: uppercase; }
  .rec-severity { color: #999; }
  .rec-category { color: #666; }
  .rec-action { margin-top: 8px; font-size: 0.85rem; color: #7c6ff7; }

  .tool-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }
  .tool-card { text-align: left; padding: 16px; }
  .tool-info { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px; }
  .tool-name { font-weight: 600; color: #7c6ff7; font-size: 1rem; }
  .tool-usage { font-size: 0.8rem; color: #888; }
  .tool-meta { display: flex; justify-content: space-between; font-size: 0.75rem; color: #666; margin-bottom: 12px; }
  .progress-bar { height: 6px; background: #222; border-radius: 3px; overflow: hidden; }
  .progress-fill { height: 100%; background: #7c6ff7; border-radius: 3px; transition: width 0.3s; }

  .cache-savings-card {
    background: #1a2a1a;
    border: 1px solid #2a4a2a;
    border-radius: 10px;
    padding: 20px;
    text-align: center;
    margin: 20px 0;
  }
  .savings-value { display: block; font-size: 1.4rem; font-weight: 700; color: #5cdb5c; }
  .savings-label { display: block; font-size: 0.8rem; color: #666; margin-top: 4px; }

  .expensive-session { margin: 20px 0; }

  .cost-note { margin-top: 24px; font-size: 0.85rem; color: #666; font-style: italic; }
  .placeholder { text-align: center; padding: 60px; color: #555; }

  @media (max-width: 900px) {
    .charts-row { grid-template-columns: 1fr; }
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
    .filters { flex-wrap: wrap; }
  }
</style>
