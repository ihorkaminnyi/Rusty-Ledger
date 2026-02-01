use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::BackendError;

use super::summary::{PortfolioSummary, PositionSummary};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetAllocation {
    pub symbol: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub target_percent: Decimal,
}

#[derive(Debug)]
pub struct ValidatedTargets(Vec<TargetAllocation>);

impl ValidatedTargets {
    pub fn new(targets: Vec<TargetAllocation>) -> Result<Self, BackendError> {
        if targets.is_empty() {
            return Err(BackendError::Validation {
                reason: "Target allocations are required to compute a rebalance plan.".to_string(),
            });
        }

        let total_target_percent: Decimal = targets.iter().map(|t| t.target_percent).sum();
        let tolerance = rust_decimal::dec!(0.01);
        let hundred = rust_decimal::dec!(100);

        if (total_target_percent - hundred).abs() > tolerance {
            return Err(BackendError::Validation {
                reason: format!(
                    "Target allocations must sum to 100%, but they currently sum to {:.2}%.",
                    total_target_percent
                ),
            });
        }

        Ok(Self(targets))
    }

    pub fn as_slice(&self) -> &[TargetAllocation] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TradeAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceStrategy {
    #[serde(rename = "Buy Only")]
    BuyOnly,
    #[serde(rename = "Full Rebalance")]
    Full,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeInstruction {
    pub symbol: String,
    pub action: TradeAction,
    #[serde(with = "rust_decimal::serde::float")]
    pub value_delta: Decimal,
    #[serde(
        with = "rust_decimal::serde::float_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub quantity_delta: Option<Decimal>,
    #[serde(
        with = "rust_decimal::serde::float_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub price_used: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebalancePlan {
    #[serde(with = "rust_decimal::serde::float")]
    pub total_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub deposit_amount: Decimal,
    pub trades: Vec<TradeInstruction>,
    pub rebalance_strategy: RebalanceStrategy,
}

pub struct PortfolioRebalancer;

impl PortfolioRebalancer {
    pub fn calculate(
        summary: &PortfolioSummary,
        targets: &ValidatedTargets,
        deposit_amount: Decimal,
        rebalance_strategy: RebalanceStrategy,
    ) -> RebalancePlan {
        let total_value = if summary.totals.market_value < Decimal::ZERO {
            Decimal::ZERO
        } else {
            summary.totals.market_value
        };
        let positions: HashMap<&str, &PositionSummary> = summary
            .positions
            .iter()
            .map(|position| (position.symbol.as_str(), position))
            .collect();

        match rebalance_strategy {
            RebalanceStrategy::BuyOnly => {
                Self::calculate_buy_only(total_value, &positions, targets, deposit_amount)
            }
            RebalanceStrategy::Full => Self::calculate_full_rebalance(
                total_value,
                &positions,
                targets,
                deposit_amount,
                rebalance_strategy,
            ),
        }
    }

    fn calculate_buy_only(
        current_total_value: Decimal,
        positions: &HashMap<&str, &PositionSummary>,
        targets: &ValidatedTargets,
        deposit_amount: Decimal,
    ) -> RebalancePlan {
        let mut trades = Vec::new();
        let hundred = rust_decimal::dec!(100);

        let projected_total_value = current_total_value + deposit_amount;

        struct DeficitInfo<'a> {
            symbol: &'a str,
            deficit: Decimal,
            price: Decimal,
        }

        let mut deficits = Vec::new();
        let mut total_deficit = Decimal::ZERO;

        for target in targets.as_slice() {
            let symbol = target.symbol.trim();
            if symbol.is_empty() {
                continue;
            }

            let target_ideal_value = projected_total_value * (target.target_percent / hundred);

            let (current_value, price) = positions
                .get(symbol)
                .map(|p| (p.market_value, p.price))
                .unwrap_or((Decimal::ZERO, Decimal::ZERO));

            if price.is_zero() {
                continue;
            }

            let deficit = if target_ideal_value > current_value {
                target_ideal_value - current_value
            } else {
                Decimal::ZERO
            };

            if deficit > Decimal::ZERO {
                deficits.push(DeficitInfo {
                    symbol,
                    deficit,
                    price,
                });
                total_deficit += deficit;
            }
        }

        if total_deficit > Decimal::ZERO && deposit_amount > Decimal::ZERO {
            for item in deficits {
                let weight = item.deficit / total_deficit;
                let amount_to_invest = deposit_amount * weight;

                if amount_to_invest.is_zero() {
                    continue;
                }

                // TODO: Додати підтримку дробових акцій, якщо брокер дозволяє
                let quantity = (amount_to_invest / item.price).round();

                if quantity.is_zero() {
                    continue;
                }

                let value_delta = quantity * item.price;

                trades.push(TradeInstruction {
                    symbol: item.symbol.to_string(),
                    action: TradeAction::Buy,
                    value_delta,
                    quantity_delta: Some(quantity),
                    price_used: Some(item.price),
                });
            }
        }
        RebalancePlan {
            total_value: current_total_value,
            deposit_amount,
            trades,
            rebalance_strategy: RebalanceStrategy::BuyOnly,
        }
    }

    fn calculate_full_rebalance(
        total_value: Decimal,
        positions: &HashMap<&str, &PositionSummary>,
        targets: &ValidatedTargets,
        deposit_amount: Decimal,
        rebalance_strategy: RebalanceStrategy,
    ) -> RebalancePlan {
        let (mut trades, mut handled) =
            Self::calculate_target_trades(total_value, positions, targets);
        let sell_off_trades = Self::calculate_sell_off_trades(positions, &mut handled);
        trades.extend(sell_off_trades);

        RebalancePlan {
            total_value,
            deposit_amount,
            trades,
            rebalance_strategy,
        }
    }

    fn calculate_target_trades<'a>(
        total_value: Decimal,
        positions: &HashMap<&'a str, &'a PositionSummary>,
        targets: &'a ValidatedTargets,
    ) -> (Vec<TradeInstruction>, HashSet<&'a str>) {
        let mut trades = Vec::new();
        let mut handled = HashSet::new();
        let hundred = rust_decimal::dec!(100);

        for target in targets.as_slice() {
            let symbol = target.symbol.trim();
            if symbol.is_empty() {
                continue;
            }

            let desired_value = total_value * (target.target_percent / hundred);
            let position = positions.get(symbol);
            let current_value = position
                .map(|pos| pos.market_value)
                .unwrap_or(Decimal::ZERO);
            let delta_value = desired_value - current_value;

            if delta_value.is_zero() {
                handled.insert(symbol);
                continue;
            }

            let price_used = position
                .map(|pos| pos.price)
                .filter(|price| !price.is_zero());

            let (quantity_delta, value_delta) = match price_used {
                Some(price) => {
                    let rounded_qty = (delta_value / price).round();
                    if rounded_qty.is_zero() {
                        handled.insert(symbol);
                        continue;
                    }
                    let adjusted_value = rounded_qty * price;
                    (Some(rounded_qty), adjusted_value)
                }
                None => (None, delta_value),
            };

            let action = if value_delta.is_sign_positive() {
                TradeAction::Buy
            } else {
                TradeAction::Sell
            };

            trades.push(TradeInstruction {
                symbol: symbol.to_string(),
                action,
                value_delta,
                quantity_delta,
                price_used,
            });

            handled.insert(symbol);
        }

        (trades, handled)
    }

    fn calculate_sell_off_trades<'a>(
        positions: &HashMap<&'a str, &'a PositionSummary>,
        handled: &mut HashSet<&'a str>,
    ) -> Vec<TradeInstruction> {
        let mut trades = Vec::new();

        for (symbol, position) in positions
            .iter()
            .filter(|(symbol, _)| !handled.contains(**symbol))
        {
            if position.market_value.is_zero() {
                continue;
            }

            let rounded_quantity = -(position.quantity.round());
            let rounded_value = rounded_quantity * position.price;

            trades.push(TradeInstruction {
                symbol: (*symbol).to_string(),
                action: TradeAction::Sell,
                value_delta: rounded_value,
                quantity_delta: Some(rounded_quantity),
                price_used: Some(position.price),
            });
        }

        trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        infra::csv::ib_report_parser::view::{AccountInfo, StatementInfo},
        portfolio::summary::PortfolioTotals,
    };

    fn summary_with_positions(positions: Vec<PositionSummary>) -> PortfolioSummary {
        let market_value: Decimal = positions.iter().map(|pos| pos.market_value).sum();
        PortfolioSummary {
            statement: StatementInfo::default(),
            account_info: AccountInfo::default(),
            totals: PortfolioTotals {
                market_value,
                cost_basis: Decimal::ZERO,
                unrealized_pl: Decimal::ZERO,
                unrealized_pl_percent: Decimal::ZERO,
                total_positions: positions.len(),
            },
            positions,
        }
    }

    fn position(symbol: &str, market_value: Decimal, price: Decimal) -> PositionSummary {
        let quantity = if price.is_zero() {
            Decimal::ZERO
        } else {
            market_value / price
        };
        PositionSummary {
            symbol: symbol.into(),
            quantity,
            price,
            market_value,
            allocation_percent: Decimal::ZERO,
            unrealized_pl: Decimal::ZERO,
            roi_percent: Decimal::ZERO,
        }
    }

    #[test]
    fn generates_buys_and_sells_for_targets() {
        let summary = summary_with_positions(vec![
            position("VTI", rust_decimal::dec!(700), rust_decimal::dec!(70)),
            position("VXUS", rust_decimal::dec!(300), rust_decimal::dec!(60)),
        ]);
        let targets = ValidatedTargets::new(vec![
            TargetAllocation {
                symbol: "VTI".into(),
                target_percent: rust_decimal::dec!(60),
            },
            TargetAllocation {
                symbol: "VXUS".into(),
                target_percent: rust_decimal::dec!(40),
            },
        ])
        .unwrap();

        let plan = PortfolioRebalancer::calculate(
            &summary,
            &targets,
            Decimal::ZERO,
            RebalanceStrategy::Full,
        );

        assert_eq!(plan.trades.len(), 2);
        assert!(plan
            .trades
            .iter()
            .any(|trade| matches!(trade.action, TradeAction::Sell) && trade.symbol == "VTI"));
        assert!(plan
            .trades
            .iter()
            .any(|trade| matches!(trade.action, TradeAction::Buy) && trade.symbol == "VXUS"));
    }

    #[test]
    fn sells_positions_without_targets() {
        let summary = summary_with_positions(vec![position(
            "BND",
            rust_decimal::dec!(500),
            rust_decimal::dec!(80),
        )]);
        let targets = ValidatedTargets::new(vec![TargetAllocation {
            symbol: "VTI".into(),
            target_percent: rust_decimal::dec!(100),
        }])
        .unwrap();
        let plan = PortfolioRebalancer::calculate(
            &summary,
            &targets,
            Decimal::ZERO,
            RebalanceStrategy::Full,
        );

        assert_eq!(plan.trades.len(), 2);
        assert!(plan
            .trades
            .iter()
            .any(|trade| matches!(trade.action, TradeAction::Sell) && trade.symbol == "BND"));
    }
}
