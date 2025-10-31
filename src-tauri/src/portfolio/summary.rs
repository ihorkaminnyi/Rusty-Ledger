use std::str::FromStr;

use serde::Serialize;

use crate::{
    error::ParseError,
    infra::csv::ib_report_parser::{
        records::{OpenPositionRecord, RowKind},
        view::{AccountInfo, ReportViewModel, StatementInfo},
    },
};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSummary {
    pub statement: StatementInfo,
    pub account_info: AccountInfo,
    pub totals: PortfolioTotals,
    pub positions: Vec<PositionSummary>,
}

impl PortfolioSummary {
    pub fn try_from_report(view: ReportViewModel) -> Result<Self, ParseError> {
        let ReportViewModel {
            statement,
            account_info,
            mark_to_market: _,
            open_positions,
        } = view;

        let (positions, market_value_sum) = build_positions(&open_positions)?;
        let totals = build_totals(&open_positions, market_value_sum)?;

        Ok(PortfolioSummary {
            statement,
            account_info,
            totals,
            positions,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioTotals {
    #[serde(with = "rust_decimal::serde::float")]
    pub market_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub cost_basis: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub unrealized_pl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub unrealized_pl_percent: Decimal,
    pub total_positions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSummary {
    pub symbol: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub quantity: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub market_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub allocation_percent: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub unrealized_pl: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub roi_percent: Decimal,
}

fn build_positions(
    records: &[OpenPositionRecord],
) -> Result<(Vec<PositionSummary>, Decimal), ParseError> {
    struct InterimPosition {
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        market_value: Decimal,
        unrealized_pl: Decimal,
        invested_value: Decimal,
    }

    let interim: Result<Vec<InterimPosition>, _> = records
        .iter()
        .filter(|record| record.kind == RowKind::Data)
        .map(|record| -> Result<InterimPosition, ParseError> {
            Ok(InterimPosition {
                symbol: record.symbol.clone(),
                quantity: parse_decimal(&record.quantity)?,
                price: parse_decimal(&record.close_price)?,
                market_value: parse_decimal(&record.value)?,
                invested_value: parse_decimal(&record.cost_basis)?,
                unrealized_pl: parse_decimal(&record.unrealized_pl)?,
            })
        })
        .collect();
    let interim = interim?;
    let total_market_value = interim
        .iter()
        .fold(Decimal::ZERO, |acc, position| acc + position.market_value);
    let hundred = rust_decimal::dec!(100);

    let positions = interim
        .into_iter()
        .map(|raw| PositionSummary {
            symbol: raw.symbol,
            quantity: raw.quantity,
            price: raw.price,
            market_value: raw.market_value,
            allocation_percent: if total_market_value.is_zero() {
                Decimal::ZERO
            } else {
                (raw.market_value / total_market_value) * hundred
            },
            unrealized_pl: raw.unrealized_pl,
            roi_percent: if raw.invested_value.is_zero() {
                Decimal::ZERO
            } else {
                (raw.unrealized_pl / raw.invested_value) * hundred
            },
        })
        .collect();

    Ok((positions, total_market_value))
}

fn build_totals(
    records: &[OpenPositionRecord],
    market_value_sum: Decimal,
) -> Result<PortfolioTotals, ParseError> {
    let totals_row = records.iter().find(|record| record.kind == RowKind::Total);

    let mut positions_count = 0usize;
    let (cost_basis_sum, unrealized_sum) = records
        .iter()
        .filter(|record| record.kind == RowKind::Data)
        .try_fold(
            (Decimal::ZERO, Decimal::ZERO),
            |(cost_acc, unrealized_acc), record| -> Result<_, ParseError> {
                positions_count += 1;
                Ok((
                    cost_acc + parse_decimal(&record.cost_basis)?,
                    unrealized_acc + parse_decimal(&record.unrealized_pl)?,
                ))
            },
        )?;

    let (cost_basis, unrealized_pl) = match totals_row {
        Some(row) => (
            parse_decimal(&row.cost_basis)?,
            parse_decimal(&row.unrealized_pl)?,
        ),
        None => (cost_basis_sum, unrealized_sum),
    };
    let hundred = rust_decimal::dec!(100);

    Ok(PortfolioTotals {
        market_value: market_value_sum,
        cost_basis,
        unrealized_pl,
        unrealized_pl_percent: if cost_basis.is_zero() {
            Decimal::ZERO
        } else {
            (unrealized_pl / cost_basis) * hundred
        },
        total_positions: positions_count,
    })
}

fn parse_decimal(value: &str) -> Result<Decimal, ParseError> {
    let normalized = value.replace(',', "");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(Decimal::ZERO);
    }
    Decimal::from_str(trimmed).map_err(ParseError::from)
}
