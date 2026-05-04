use claudy::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
use claudy::ports::analytics_ports::AnalyticsStore;

fn main() {
    let db_path = dirs::home_dir()
        .unwrap()
        .join("Library")
        .join("Application Support")
        .join("com.claudy.analytics")
        .join("analytics.db");

    eprintln!("DB: {:?}", db_path);

    let store = SqliteAnalyticsStore::open(db_path.to_str().unwrap()).unwrap();
    store.initialize_schema().unwrap();

    let mut ok = true;

    macro_rules! check {
        ($label:expr, $expr:expr) => {
            match $expr {
                Ok(_) => eprintln!("  OK  {}", $label),
                Err(e) => {
                    eprintln!("  FAIL {} — {}", $label, e);
                    ok = false;
                }
            }
        };
    }

    check!("get_sessions (30d)", store.get_sessions(100, Some(30), None).map(|_| ()));
    check!("get_recommendations", store.get_recommendations().map(|_| ()));
    check!("aggregate_token_trends", store.aggregate_token_trends(30, None).map(|_| ()));
    check!("aggregate_tool_distribution", store.aggregate_tool_distribution(Some(30), None).map(|_| ()));
    check!("aggregate_cost_metrics", store.aggregate_cost_metrics(30, None).map(|_| ()));
    check!("aggregate_dashboard_stats", store.aggregate_dashboard_stats(30, None).map(|_| ()));

    if ok {
        eprintln!("\nAll checks passed.");
    } else {
        eprintln!("\nSome checks FAILED.");
        std::process::exit(1);
    }
}
