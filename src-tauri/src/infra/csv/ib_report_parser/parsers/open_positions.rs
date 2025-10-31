use crate::infra::csv::ib_report_parser::records::OpenPositionRecord;

use super::Section;

pub type OpenPositionsSection = Section<OpenPositionRecord>;

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    fn builds_open_positions_section() {
        let records = vec![
            StringRecord::from(vec![
                "Open Positions",
                "Header",
                "DataDiscriminator",
                "Asset Category",
                "Currency",
                "Symbol",
                "Quantity",
                "Mult",
                "Cost Price",
                "Cost Basis",
                "Close Price",
                "Value",
                "Unrealized P/L",
                "Code",
            ]),
            StringRecord::from(vec![
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
            ]),
        ];

        let section = OpenPositionsSection::from_records(&records);
        assert_eq!(section.rows.len(), 2);
        assert_eq!(section.rows[1].symbol, "AAPL");
    }
}
