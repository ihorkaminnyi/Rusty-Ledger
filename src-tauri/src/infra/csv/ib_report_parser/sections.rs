use csv::{ReaderBuilder, StringRecord};

use crate::infra::csv::ib_report_parser::parsers::{
    AccountInformationSection, MarkToMarketSection, OpenPositionsSection, StatementSection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IBReportSection {
    Statement,
    AccountInformation,
    MarkToMarketPerformanceSummary,
    OpenPositions,
}

impl IBReportSection {
    pub fn as_str(&self) -> &'static str {
        match self {
            IBReportSection::Statement => "Statement",
            IBReportSection::AccountInformation => "Account Information",
            IBReportSection::MarkToMarketPerformanceSummary => "Mark-to-Market Performance Summary",
            IBReportSection::OpenPositions => "Open Positions",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Statement" => Some(IBReportSection::Statement),
            "Account Information" => Some(IBReportSection::AccountInformation),
            "Mark-to-Market Performance Summary" => {
                Some(IBReportSection::MarkToMarketPerformanceSummary)
            }
            "Open Positions" => Some(IBReportSection::OpenPositions),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CSVSections {
    pub statement: Vec<StringRecord>,
    pub account_info: Vec<StringRecord>,
    pub mark_to_market: Vec<StringRecord>,
    pub open_positions: Vec<StringRecord>,
}

impl CSVSections {
    pub fn new() -> Self {
        Self {
            statement: Vec::new(),
            account_info: Vec::new(),
            mark_to_market: Vec::new(),
            open_positions: Vec::new(),
        }
    }

    fn push_record(&mut self, section: IBReportSection, record: &StringRecord) {
        let target = match section {
            IBReportSection::Statement => &mut self.statement,
            IBReportSection::AccountInformation => &mut self.account_info,
            IBReportSection::MarkToMarketPerformanceSummary => &mut self.mark_to_market,
            IBReportSection::OpenPositions => &mut self.open_positions,
        };
        target.push(record.clone());
    }

    // TODO: looks like must be ParseError not String
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut sections = CSVSections::new();
        let mut current_section: Option<IBReportSection> = None;
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(content.as_bytes());

        for result in reader.records() {
            let record = result.map_err(|err| format!("Failed to read CSV: {err}"))?;
            if record.is_empty() {
                continue;
            }

            let first_field = record.get(0).map(|value| value.trim()).unwrap_or("");

            match (
                first_field.is_empty(),
                IBReportSection::from_str(first_field),
            ) {
                (true, _) => {
                    if let Some(section) = current_section {
                        sections.push_record(section, &record);
                    }
                    continue;
                }
                (_, Some(section)) => {
                    current_section = Some(section);
                    sections.push_record(section, &record);
                    continue;
                }
                _ => {}
            }

            if record
                .get(1)
                .map(|value| value.trim().eq_ignore_ascii_case("header"))
                .unwrap_or(false)
            {
                current_section = None;
            }
        }

        Ok(sections)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParsedSections {
    pub statement: StatementSection,
    pub account_info: AccountInformationSection,
    pub mark_to_market: MarkToMarketSection,
    pub open_positions: OpenPositionsSection,
}

impl From<CSVSections> for ParsedSections {
    fn from(sections: CSVSections) -> Self {
        ParsedSections {
            statement: StatementSection::from_records(&sections.statement),
            account_info: AccountInformationSection::from_records(&sections.account_info),
            mark_to_market: MarkToMarketSection::from_records(&sections.mark_to_market),
            open_positions: OpenPositionsSection::from_records(&sections.open_positions),
        }
    }
}

pub fn parse_sections(content: &str) -> Result<ParsedSections, String> {
    CSVSections::parse(content).map(ParsedSections::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_matches_known_sections() {
        assert_eq!(
            IBReportSection::from_str("Statement"),
            Some(IBReportSection::Statement)
        );
        assert_eq!(
            IBReportSection::from_str("Account Information"),
            Some(IBReportSection::AccountInformation)
        );
        assert_eq!(
            IBReportSection::from_str("Mark-to-Market Performance Summary"),
            Some(IBReportSection::MarkToMarketPerformanceSummary)
        );
        assert_eq!(
            IBReportSection::from_str("Open Positions"),
            Some(IBReportSection::OpenPositions)
        );
        assert_eq!(IBReportSection::from_str("Unknown Section"), None);
    }

    #[test]
    fn identify_sections_groups_known_headers_and_skips_unknown() {
        let csv = "\
Statement,Header,Field Name,Field Value
Statement,Data,BrokerName,Interactive Brokers LLC
Account Information,Header,Field Name,Field Value
Account Information,Data,Account,U123456
Net Asset Value,Header,Asset Class,Prior Total
Net Asset Value,Data,Total,123
Mark-to-Market Performance Summary,Header,Asset Category,Symbol
Mark-to-Market Performance Summary,Data,Stocks,AAPL
Open Positions,Header,DataDiscriminator,Asset Category
Open Positions,Data,Summary,Stocks
";

        let sections = CSVSections::parse(csv).expect("parser should succeed");

        let expected_statement = vec![
            StringRecord::from(vec!["Statement", "Header", "Field Name", "Field Value"]),
            StringRecord::from(vec![
                "Statement",
                "Data",
                "BrokerName",
                "Interactive Brokers LLC",
            ]),
        ];
        assert_eq!(sections.statement, expected_statement);

        let expected_account_info = vec![
            StringRecord::from(vec![
                "Account Information",
                "Header",
                "Field Name",
                "Field Value",
            ]),
            StringRecord::from(vec!["Account Information", "Data", "Account", "U123456"]),
        ];
        assert_eq!(sections.account_info, expected_account_info);

        let expected_mark_to_market = vec![
            StringRecord::from(vec![
                "Mark-to-Market Performance Summary",
                "Header",
                "Asset Category",
                "Symbol",
            ]),
            StringRecord::from(vec![
                "Mark-to-Market Performance Summary",
                "Data",
                "Stocks",
                "AAPL",
            ]),
        ];
        assert_eq!(sections.mark_to_market, expected_mark_to_market);

        let expected_open_positions = vec![
            StringRecord::from(vec![
                "Open Positions",
                "Header",
                "DataDiscriminator",
                "Asset Category",
            ]),
            StringRecord::from(vec!["Open Positions", "Data", "Summary", "Stocks"]),
        ];
        assert_eq!(sections.open_positions, expected_open_positions);

        assert!(sections
            .statement
            .iter()
            .all(|record| record.get(0).unwrap_or_default() != "Net Asset Value"));
        assert!(sections
            .account_info
            .iter()
            .all(|record| record.get(0).unwrap_or_default() != "Net Asset Value"));
        assert!(sections
            .mark_to_market
            .iter()
            .all(|record| record.get(0).unwrap_or_default() != "Net Asset Value"));
        assert!(sections
            .open_positions
            .iter()
            .all(|record| record.get(0).unwrap_or_default() != "Net Asset Value"));
    }

    #[test]
    fn parse_sections_returns_typed_structs() {
        let csv = "\
Statement,Header,Field Name,Field Value
Statement,Data,BrokerName,Interactive Brokers LLC
Account Information,Header,Field Name,Field Value
Account Information,Data,Account,U123456
Mark-to-Market Performance Summary,Header,Asset Category,Symbol
Mark-to-Market Performance Summary,Data,Stocks,AAPL
Open Positions,Header,DataDiscriminator,Asset Category
Open Positions,Data,Summary,Stocks
";

        let parsed = parse_sections(csv).expect("parse sections");
        assert_eq!(parsed.statement.rows.len(), 2);
        assert_eq!(parsed.account_info.rows.len(), 2);
        assert_eq!(parsed.mark_to_market.rows.len(), 2);
        assert_eq!(parsed.open_positions.rows.len(), 2);
    }
}
