use csv::StringRecord;

use crate::infra::csv::ib_report_parser::records::FieldRow;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountInformationSection {
    pub rows: Vec<FieldRow>,
}

impl AccountInformationSection {
    pub fn from_records(records: &[StringRecord]) -> Self {
        let rows = records
            .iter()
            .filter_map(FieldRow::from_record)
            .collect::<Vec<_>>();
        Self { rows }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_account_info_section() {
        let records = vec![
            StringRecord::from(vec!["Account Information", "Header", "Field Name", "Value"]),
            StringRecord::from(vec!["Account Information", "Data", "Field Name", "Value"]),
        ];

        let section = AccountInformationSection::from_records(&records);
        assert_eq!(section.rows.len(), 2);
        assert_eq!(section.rows[0].name, "Field Name");
    }
}
