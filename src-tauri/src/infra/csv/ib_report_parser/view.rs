use heck::ToSnakeCase;
use serde::Serialize;

use crate::infra::csv::ib_report_parser::{
    parsers::{
        AccountInformationSection, MarkToMarketSection, OpenPositionsSection, StatementSection,
    },
    records::{FieldRow, MarkToMarketRecord, OpenPositionRecord, RowKind},
    sections::ParsedSections,
};

#[derive(Debug, Clone, Serialize)]
pub struct ReportViewModel {
    pub statement: StatementInfo,
    pub account_info: AccountInfo,
    pub mark_to_market: Vec<MarkToMarketRecord>,
    pub open_positions: Vec<OpenPositionRecord>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatementInfo {
    pub title: Option<String>,
    pub broker_name: Option<String>,
    pub broker_address: Option<String>,
    pub period: Option<String>,
    pub when_generated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub account_capabilities: Option<String>,
    pub account_type: Option<String>,
    pub base_currency: Option<String>,
    pub customer_type: Option<String>,
    pub name: Option<String>,
}

impl From<ParsedSections> for ReportViewModel {
    fn from(parsed: ParsedSections) -> Self {
        ReportViewModel {
            statement: parsed.statement.into_view(),
            account_info: parsed.account_info.into_view(),
            mark_to_market: parsed.mark_to_market.into_view(),
            open_positions: parsed.open_positions.into_view(),
        }
    }
}

impl ReportViewModel {
    pub fn from_sections(parsed: ParsedSections) -> Self {
        parsed.into()
    }
}

trait SectionView {
    type Output: Serialize;
    fn into_view(self) -> Self::Output;
}

impl SectionView for StatementSection {
    type Output = StatementInfo;
    fn into_view(self) -> Self::Output {
        statement_rows_to_info(self.rows)
    }
}

impl SectionView for AccountInformationSection {
    type Output = AccountInfo;
    fn into_view(self) -> Self::Output {
        account_rows_to_info(self.rows)
    }
}

impl SectionView for MarkToMarketSection {
    type Output = Vec<MarkToMarketRecord>;
    fn into_view(self) -> Self::Output {
        self.rows
    }
}

impl SectionView for OpenPositionsSection {
    type Output = Vec<OpenPositionRecord>;
    fn into_view(self) -> Self::Output {
        self.rows
    }
}

fn account_rows_to_info(rows: Vec<FieldRow>) -> AccountInfo {
    let mut info = AccountInfo::default();

    for row in rows.into_iter().filter(|row| row.kind != RowKind::Header) {
        let FieldRow { name, value, .. } = row;
        let key = name.to_snake_case();
        if key.is_empty() || key == "account" {
            continue;
        }

        match key.as_str() {
            "account_capabilities" => info.account_capabilities = Some(value),
            "account_type" => info.account_type = Some(value),
            "base_currency" => info.base_currency = Some(value),
            "customer_type" => info.customer_type = Some(value),
            "name" => info.name = Some(value),
            _ => {}
        }
    }

    info
}

fn statement_rows_to_info(rows: Vec<FieldRow>) -> StatementInfo {
    let mut info = StatementInfo::default();

    for row in rows.into_iter().filter(|row| row.kind != RowKind::Header) {
        let FieldRow { name, value, .. } = row;
        let key = name.to_snake_case();
        if key.is_empty() {
            continue;
        }

        match key.as_str() {
            "title" => info.title = Some(value),
            "broker_name" => info.broker_name = Some(value),
            "broker_address" => info.broker_address = Some(value),
            "period" => info.period = Some(value),
            "when_generated" => info.when_generated = Some(value),
            _ => {}
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_statement_rows_into_struct() {
        let rows = vec![
            FieldRow {
                kind: RowKind::Data,
                name: "Title".into(),
                value: "Account Statement".into(),
            },
            FieldRow {
                kind: RowKind::Data,
                name: "Broker Name".into(),
                value: "Interactive Brokers".into(),
            },
            FieldRow {
                kind: RowKind::Data,
                name: "Period".into(),
                value: "2024-01-01 - 2024-12-31".into(),
            },
        ];

        let statement = statement_rows_to_info(rows);
        assert_eq!(statement.title.as_deref(), Some("Account Statement"));
        assert_eq!(
            statement.broker_name.as_deref(),
            Some("Interactive Brokers")
        );
        assert_eq!(statement.period.as_deref(), Some("2024-01-01 - 2024-12-31"));
    }
}
