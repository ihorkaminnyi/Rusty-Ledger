use crate::infra::csv::ib_report_parser::records::FieldRow;

use super::Section;

pub type StatementSection = Section<FieldRow>;

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    fn builds_statement_section() {
        let records = vec![
            StringRecord::from(vec!["Statement", "Header", "Field Name", "Value"]),
            StringRecord::from(vec!["Statement", "Data", "Field Name", "Value"]),
        ];

        let section = StatementSection::from_records(&records);
        assert_eq!(section.rows.len(), 2);
        assert_eq!(section.rows[0].name, "Field Name");
        assert_eq!(
            section.rows[1].kind,
            crate::infra::csv::ib_report_parser::records::RowKind::Data
        );
    }
}
