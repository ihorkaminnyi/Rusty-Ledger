use crate::infra::csv::ib_report_parser::records::MarkToMarketRecord;

use super::Section;

pub type MarkToMarketSection = Section<MarkToMarketRecord>;

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    fn builds_mark_to_market_section() {
        let records = vec![
            StringRecord::from(vec![
                "Mark-to-Market Performance Summary",
                "Header",
                "Asset Category",
                "Symbol",
                "Prior Quantity",
                "Current Quantity",
                "Prior Price",
                "Current Price",
                "P/L Position",
                "P/L Transaction",
                "P/L Commissions",
                "P/L Other",
                "P/L Total",
                "Code",
            ]),
            StringRecord::from(vec![
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
            ]),
        ];

        let section = MarkToMarketSection::from_records(&records);
        assert_eq!(section.rows.len(), 2);
        assert_eq!(section.rows[1].symbol, "AAPL");
    }
}
