pub mod parsers;
pub mod records;
pub mod sections;
pub mod view;

use csv::StringRecord;

use parsers::{
    AccountInformationSection, MarkToMarketSection, OpenPositionsSection, StatementSection,
};
use sections::{parse_sections, ParsedSections};
use view::ReportViewModel;

pub struct IBCSVParser;

impl IBCSVParser {
    pub fn parse_report(content: &str) -> Result<ParsedSections, String> {
        parse_sections(content)
    }

    pub fn parse_statement(records: &[StringRecord]) -> StatementSection {
        StatementSection::from_records(records)
    }

    pub fn parse_account_info(records: &[StringRecord]) -> AccountInformationSection {
        AccountInformationSection::from_records(records)
    }

    pub fn parse_mark_to_market(records: &[StringRecord]) -> MarkToMarketSection {
        MarkToMarketSection::from_records(records)
    }

    pub fn parse_open_positions(records: &[StringRecord]) -> OpenPositionsSection {
        OpenPositionsSection::from_records(records)
    }

    pub fn parse_report_view(content: &str) -> Result<ReportViewModel, String> {
        Self::parse_report(content).map(ReportViewModel::from)
    }
}
