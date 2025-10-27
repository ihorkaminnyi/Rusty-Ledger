use csv::StringRecord;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RowKind {
    Header,
    Data,
    Total,
    Other,
}

impl RowKind {
    pub fn parse(value: Option<&str>) -> Self {
        match value.unwrap_or_default().trim().to_lowercase().as_str() {
            "header" => RowKind::Header,
            "data" => RowKind::Data,
            "total" => RowKind::Total,
            _ => RowKind::Other,
        }
    }
}

fn field(record: &StringRecord, idx: usize) -> Option<String> {
    record.get(idx).map(|value| value.trim().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldRow {
    pub kind: RowKind,
    pub name: String,
    pub value: String,
}

impl FieldRow {
    pub fn from_record(record: &StringRecord) -> Option<Self> {
        Some(Self {
            kind: RowKind::parse(record.get(1)),
            name: field(record, 2)?,
            value: field(record, 3).unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarkToMarketRecord {
    pub kind: RowKind,
    pub asset_category: String,
    pub symbol: String,
    pub prior_quantity: String,
    pub current_quantity: String,
    pub prior_price: String,
    pub current_price: String,
    pub pl_position: String,
    pub pl_transaction: String,
    pub pl_commissions: String,
    pub pl_other: String,
    pub pl_total: String,
    pub code: String,
}

impl MarkToMarketRecord {
    pub fn from_record(record: &StringRecord) -> Option<Self> {
        Some(Self {
            kind: RowKind::parse(record.get(1)),
            asset_category: field(record, 2)?,
            symbol: field(record, 3).unwrap_or_default(),
            prior_quantity: field(record, 4).unwrap_or_default(),
            current_quantity: field(record, 5).unwrap_or_default(),
            prior_price: field(record, 6).unwrap_or_default(),
            current_price: field(record, 7).unwrap_or_default(),
            pl_position: field(record, 8).unwrap_or_default(),
            pl_transaction: field(record, 9).unwrap_or_default(),
            pl_commissions: field(record, 10).unwrap_or_default(),
            pl_other: field(record, 11).unwrap_or_default(),
            pl_total: field(record, 12).unwrap_or_default(),
            code: field(record, 13).unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenPositionRecord {
    pub kind: RowKind,
    pub data_discriminator: String,
    pub asset_category: String,
    pub currency: String,
    pub symbol: String,
    pub quantity: String,
    pub multiplier: String,
    pub cost_price: String,
    pub cost_basis: String,
    pub close_price: String,
    pub value: String,
    pub unrealized_pl: String,
    pub code: String,
}

impl OpenPositionRecord {
    pub fn from_record(record: &StringRecord) -> Option<Self> {
        Some(Self {
            kind: RowKind::parse(record.get(1)),
            data_discriminator: field(record, 2).unwrap_or_default(),
            asset_category: field(record, 3).unwrap_or_default(),
            currency: field(record, 4).unwrap_or_default(),
            symbol: field(record, 5).unwrap_or_default(),
            quantity: field(record, 6).unwrap_or_default(),
            multiplier: field(record, 7).unwrap_or_default(),
            cost_price: field(record, 8).unwrap_or_default(),
            cost_basis: field(record, 9).unwrap_or_default(),
            close_price: field(record, 10).unwrap_or_default(),
            value: field(record, 11).unwrap_or_default(),
            unrealized_pl: field(record, 12).unwrap_or_default(),
            code: field(record, 13).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_field_row() {
        let record = StringRecord::from(vec!["Statement", "Data", "Field Name", "Field Value"]);
        let row = FieldRow::from_record(&record).expect("field row");
        assert_eq!(row.kind, RowKind::Data);
        assert_eq!(row.name, "Field Name");
        assert_eq!(row.value, "Field Value");
    }

    #[test]
    fn parses_mark_to_market_record() {
        let record = StringRecord::from(vec![
            "Mark-to-Market Performance Summary",
            "Data",
            "Stocks",
            "AAPL",
            "6",
            "6",
            "254.4300",
            "254.6300",
            "1.2",
            "0",
            "0",
            "0",
            "1.2",
            "",
        ]);
        let parsed = MarkToMarketRecord::from_record(&record).expect("parsed");
        assert_eq!(parsed.asset_category, "Stocks");
        assert_eq!(parsed.symbol, "AAPL");
        assert_eq!(parsed.pl_total, "1.2");
    }

    #[test]
    fn parses_open_position_record() {
        let record = StringRecord::from(vec![
            "Open Positions",
            "Data",
            "Summary",
            "Stocks",
            "USD",
            "AAPL",
            "6",
            "1",
            "212.34",
            "1274.04",
            "254.63",
            "1527.78",
            "253.73",
            "",
        ]);
        let parsed = OpenPositionRecord::from_record(&record).expect("parsed");
        assert_eq!(parsed.asset_category, "Stocks");
        assert_eq!(parsed.currency, "USD");
        assert_eq!(parsed.unrealized_pl, "253.73");
    }
}
