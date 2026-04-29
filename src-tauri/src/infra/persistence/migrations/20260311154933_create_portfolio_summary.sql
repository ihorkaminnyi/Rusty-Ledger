CREATE TABLE IF NOT EXISTS portfolio_summaries (
    id               INTEGER  PRIMARY KEY AUTOINCREMENT,

    -- AccountInfo
    account_type             TEXT,
    customer_type            TEXT,
    account_capabilities     TEXT,
    base_currency            TEXT,
    account_name             TEXT,

    -- StatementInfo
    statement_period TEXT,
    generated_at     TEXT,
    statement_title  TEXT,
    broker_name      TEXT,
    broker_address   TEXT,

    -- PortfolioTotals
    market_value          TEXT NOT NULL,
    cost_basis            TEXT NOT NULL,
    unrealized_pl         TEXT NOT NULL,
    unrealized_pl_percent TEXT NOT NULL,
    total_positions       INTEGER NOT NULL,

    created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS portfolio_positions (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    summary_id           INTEGER NOT NULL
                             REFERENCES portfolio_summaries(id)
                             ON DELETE CASCADE,

    symbol               TEXT NOT NULL,
    quantity             TEXT NOT NULL,
    price                TEXT NOT NULL,
    market_value         TEXT NOT NULL,
    allocation_percent   TEXT NOT NULL,
    unrealized_pl        TEXT NOT NULL,
    roi_percent          TEXT NOT NULL,

    UNIQUE(summary_id, symbol)
);

CREATE INDEX IF NOT EXISTS idx_portfolio_positions_summary_id
    ON portfolio_positions(summary_id);
