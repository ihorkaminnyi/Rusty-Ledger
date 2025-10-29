use std::num::ParseFloatError;

use serde::Serialize;

use crate::{
    error::ParseError,
    infra::csv::ib_report_parser::{
        records::{OpenPositionRecord, RowKind},
        view::{AccountInfo, ReportViewModel, StatementInfo},
    },
};

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
    pub market_value: f64,
    pub cost_basis: f64,
    pub unrealized_pl: f64,
    pub unrealized_pl_percent: f64,
    pub total_positions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSummary {
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub market_value: f64,
    pub allocation_percent: f64,
    pub unrealized_pl: f64,
    pub roi_percent: f64,
}

fn build_positions(
    records: &[OpenPositionRecord],
) -> Result<(Vec<PositionSummary>, f64), ParseError> {
    struct InterimPosition {
        symbol: String,
        quantity: f64,
        price: f64,
        market_value: f64,
        unrealized_pl: f64,
        invested_value: f64,
    }

    let interim: Result<Vec<InterimPosition>, _> = records
        .iter()
        .filter(|record| record.kind == RowKind::Data)
        .map(|record| -> Result<InterimPosition, ParseError> {
            Ok(InterimPosition {
                symbol: record.symbol.clone(),
                quantity: parse_number(&record.quantity)?,
                price: parse_number(&record.close_price)?,
                market_value: parse_number(&record.value)?,
                invested_value: parse_number(&record.cost_basis)?,
                unrealized_pl: parse_number(&record.unrealized_pl)?,
            })
        })
        .collect();
    let interim = interim?;
    let total_market_value: f64 = interim.iter().map(|position| position.market_value).sum();

    let positions = interim
        .into_iter()
        .map(|raw| PositionSummary {
            symbol: raw.symbol,
            quantity: raw.quantity,
            price: raw.price,
            market_value: raw.market_value,
            allocation_percent: if total_market_value.abs() > f64::EPSILON {
                (raw.market_value / total_market_value) * 100.0
            } else {
                0.0
            },
            unrealized_pl: raw.unrealized_pl,
            roi_percent: if raw.invested_value.abs() > f64::EPSILON {
                (raw.unrealized_pl / raw.invested_value) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    Ok((positions, total_market_value))
}

fn build_totals(
    records: &[OpenPositionRecord],
    market_value_sum: f64,
) -> Result<PortfolioTotals, ParseError> {
    let totals_row = records.iter().find(|record| record.kind == RowKind::Total);

    let mut positions_count = 0usize;
    let (cost_basis_sum, unrealized_sum) = records
        .iter()
        .filter(|record| record.kind == RowKind::Data)
        .try_fold(
            (0.0, 0.0),
            |(cost_acc, unrealized_acc), record| -> Result<_, ParseError> {
                positions_count += 1;
                Ok((
                    cost_acc + parse_number(&record.cost_basis)?,
                    unrealized_acc + parse_number(&record.unrealized_pl)?,
                ))
            },
        )?;

    let (cost_basis, unrealized_pl) = match totals_row {
        Some(row) => (
            parse_number(&row.cost_basis)?,
            parse_number(&row.unrealized_pl)?,
        ),
        None => (cost_basis_sum, unrealized_sum),
    };

    Ok(PortfolioTotals {
        market_value: market_value_sum,
        cost_basis,
        unrealized_pl,
        unrealized_pl_percent: if cost_basis.abs() > f64::EPSILON {
            (unrealized_pl / cost_basis) * 100.0
        } else {
            0.0
        },
        total_positions: positions_count,
    })
}

fn parse_number(value: &str) -> Result<f64, ParseFloatError> {
    let normalized = value.replace(',', "");
    normalized.parse::<f64>()
}
