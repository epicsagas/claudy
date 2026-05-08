---
name: analytics-insights
description: Analyze Claude Code usage patterns and provide actionable insights. Triggers on "usage insights", "analyze usage", "analytics insights", "usage patterns", "cost analysis". Reports on costs, model distribution, tool usage, cache efficiency, and session patterns.
---

# Usage Analytics Insights

Analyze the user's Claude Code usage data and provide actionable insights.

## Process

### Step 1: Determine the time range

- If the user specifies a date range (e.g., "last 2 weeks", "past month", "4월 데이터", "from April 1 to April 30"), calculate the appropriate `--days` value or `--from`/`--to` dates.
- Default to 7 days if no range is specified.

### Step 2: Run the insights command

```bash
claudy analytics insights --days 7
```

With a specific date range:
```bash
claudy analytics insights --from 2026-04-01 --to 2026-04-30
```

For a specific project:
```bash
claudy analytics insights --days 14 --project my-project
```

If the command fails with "no such command" or similar, the user may need to build claudy with the analytics feature first.

### Step 3: Analyze the JSON output

The output is a compact JSON with these sections:
- **period**: date range and number of days
- **overview**: total sessions, cost, turns, avg tokens, most-used model
- **daily_costs**: per-day cost breakdown by model
- **model_distribution**: cost and token breakdown per model
- **tool_usage**: top 10 tools by call count
- **notable_sessions**: top 5 most expensive sessions
- **cost_analysis**: averages and cache savings
- **cache_efficiency**: hit ratio and savings

Analyze these areas:

1. **Cost trends**: Is spending increasing, decreasing, or stable? Any spikes? Compare daily costs.
2. **Model distribution**: Are expensive models (Opus) used for tasks that could use cheaper ones (Sonnet/Haiku)?
3. **Tool usage**: Which tools are used most? Any high error rates? Unusual patterns?
4. **Cache efficiency**: Is caching being leveraged? Low hit ratio = potential savings.
5. **Notable sessions**: Any unusually expensive sessions that could be optimized?

### Step 4: Present findings

Respond in the user's language (Korean if they wrote in Korean, English otherwise).

Structure:

#### Summary
2-3 sentence overview of the period's usage.

#### Cost Analysis
Daily cost trends, weekly average, most expensive sessions.

#### Model Usage
Which models, costs per model, switching suggestions.

#### Tool Patterns
Most used tools, error rates, efficiency observations.

#### Cache Performance
Hit ratio, savings, suggestions for improvement.

#### Recommendations
Numbered list of 2-3 actionable items with specific estimated savings where possible.
