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

trait StringRecordExt {
    fn field(&self, idx: usize) -> Option<String>;
}

impl StringRecordExt for StringRecord {
    fn field(&self, idx: usize) -> Option<String> {
        self.get(idx).map(|value| value.trim().to_string())
    }
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
            name: record.field(2)?,
            value: record.field(3).unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
            asset_category: record.field(2)?,
            symbol: record.field(3).unwrap_or_default(),
            prior_quantity: record.field(4).unwrap_or_default(),
            current_quantity: record.field(5).unwrap_or_default(),
            prior_price: record.field(6).unwrap_or_default(),
            current_price: record.field(7).unwrap_or_default(),
            pl_position: record.field(8).unwrap_or_default(),
            pl_transaction: record.field(9).unwrap_or_default(),
            pl_commissions: record.field(10).unwrap_or_default(),
            pl_other: record.field(11).unwrap_or_default(),
            pl_total: record.field(12).unwrap_or_default(),
            code: record.field(13).unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
            data_discriminator: record.field(2).unwrap_or_default(),
            asset_category: record.field(3).unwrap_or_default(),
            currency: record.field(4).unwrap_or_default(),
            symbol: record.field(5).unwrap_or_default(),
            quantity: record.field(6).unwrap_or_default(),
            multiplier: record.field(7).unwrap_or_default(),
            cost_price: record.field(8).unwrap_or_default(),
            cost_basis: record.field(9).unwrap_or_default(),
            close_price: record.field(10).unwrap_or_default(),
            value: record.field(11).unwrap_or_default(),
            unrealized_pl: record.field(12).unwrap_or_default(),
            code: record.field(13).unwrap_or_default(),
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
