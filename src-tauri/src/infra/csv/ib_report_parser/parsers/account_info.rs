use crate::infra::csv::ib_report_parser::records::FieldRow;

use super::Section;

pub type AccountInformationSection = Section<FieldRow>;

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

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
