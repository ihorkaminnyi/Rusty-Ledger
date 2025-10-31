use csv::StringRecord;

use crate::infra::csv::ib_report_parser::records::{
    FieldRow, MarkToMarketRecord, OpenPositionRecord,
};

pub mod account_info;
pub mod mark_to_market;
pub mod open_positions;
pub mod statement;

pub use account_info::AccountInformationSection;
pub use mark_to_market::MarkToMarketSection;
pub use open_positions::OpenPositionsSection;
pub use statement::StatementSection;

pub trait SectionRow: Sized {
    fn from_record(record: &StringRecord) -> Option<Self>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<T> {
    pub rows: Vec<T>,
}

impl<T> Section<T> {
    pub fn new(rows: Vec<T>) -> Self {
        Self { rows }
    }
}

impl<T: SectionRow> Section<T> {
    pub fn from_records(records: &[StringRecord]) -> Self {
        let rows = records.iter().filter_map(T::from_record).collect();
        Self { rows }
    }
}

impl<T> Default for Section<T> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}

impl SectionRow for FieldRow {
    fn from_record(record: &StringRecord) -> Option<Self> {
        FieldRow::from_record(record)
    }
}

impl SectionRow for MarkToMarketRecord {
    fn from_record(record: &StringRecord) -> Option<Self> {
        MarkToMarketRecord::from_record(record)
    }
}

impl SectionRow for OpenPositionRecord {
    fn from_record(record: &StringRecord) -> Option<Self> {
        OpenPositionRecord::from_record(record)
    }
}
