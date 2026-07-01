impl TxRpcImpl {
    async fn get_transaction_history_impl(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
        filter: Option<RuntimeTxHistoryFilter>,
        sort: Option<RuntimeTxHistorySort>,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let requested_limit = pagination.limit.unwrap_or(50);
        if requested_limit == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid limit: must be > 0".to_string(),
                None::<()>,
            ));
        }
        let limit = requested_limit.min(50);
        let include_total = pagination.include_total.unwrap_or(false);

        let mut items = self.load_wallet_tx_items(&wallet_id).await?;

        if let Some(filter) = filter {
            if let (Some(from), Some(to)) = (filter.from_date, filter.to_date) {
                if from > to {
                    return Err(ErrorObjectOwned::owned(
                        -32602,
                        "Invalid date range: from_date > to_date".to_string(),
                        None::<()>,
                    ));
                }
            }

            if let (Some(min), Some(max)) = (filter.min_amount, filter.max_amount) {
                if min > max {
                    return Err(ErrorObjectOwned::owned(
                        -32602,
                        "Invalid amount range: min_amount > max_amount".to_string(),
                        None::<()>,
                    ));
                }
            }

            items.retain(|tx| {
                if let Some(status) = filter.status.as_ref() {
                    if std::mem::discriminant(&tx.status) != std::mem::discriminant(status) {
                        return false;
                    }
                }
                if let Some(from) = filter.from_date {
                    if tx.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = filter.to_date {
                    if tx.timestamp > to {
                        return false;
                    }
                }
                if let Some(min) = filter.min_amount {
                    if tx.amount < min {
                        return false;
                    }
                }
                if let Some(max) = filter.max_amount {
                    if tx.amount > max {
                        return false;
                    }
                }
                true
            });
        }

        let sort = sort.unwrap_or(RuntimeTxHistorySort {
            by: TxHistorySortBy::Timestamp,
            direction: SortDirection::Desc,
        });

        items.sort_by(|left, right| {
            let primary = match sort.by {
                TxHistorySortBy::Timestamp => left.timestamp.cmp(&right.timestamp),
                TxHistorySortBy::Amount => left.amount.cmp(&right.amount),
            };

            let primary = match sort.direction {
                SortDirection::Asc => primary,
                SortDirection::Desc => primary.reverse(),
            };

            primary
                .then_with(|| right.timestamp.cmp(&left.timestamp))
                .then_with(|| left.id.0.cmp(&right.id.0))
        });

        let total_count = items.len();
        let start_idx = if let Some(cursor) = pagination.cursor.as_ref() {
            match items.iter().position(|tx| tx.id.0 == *cursor) {
                Some(pos) => pos.saturating_add(1),
                None => {
                    return Err(ErrorObjectOwned::owned(
                        -32602,
                        "Invalid cursor".to_string(),
                        None::<()>,
                    ));
                }
            }
        } else {
            0
        };

        let page: Vec<PersistTxInfo> = items.into_iter().skip(start_idx).take(limit).collect();
        let has_more = start_idx.saturating_add(page.len()) < total_count;
        let next_cursor = if has_more {
            page.last().map(|tx| tx.id.0.clone())
        } else {
            None
        };

        Ok(RuntimePaginatedResponse {
            items: page,
            next_cursor,
            has_more,
            total_count: if include_total {
                Some(total_count)
            } else {
                None
            },
        })
    }

    async fn list_pending_transactions_impl(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let requested_limit = pagination.limit.unwrap_or(50);
        if requested_limit == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid limit: must be > 0".to_string(),
                None::<()>,
            ));
        }
        let limit = requested_limit.min(50);
        let include_total = pagination.include_total.unwrap_or(false);

        let mut items = self.load_wallet_pending_tx_items(&wallet_id).await?;
        items.sort_by(|left, right| {
            right
                .timestamp
                .cmp(&left.timestamp)
                .then_with(|| left.id.0.cmp(&right.id.0))
        });

        let total_count = items.len();
        let start_idx = if let Some(cursor) = pagination.cursor.as_ref() {
            match items.iter().position(|tx| tx.id.0 == *cursor) {
                Some(pos) => pos.saturating_add(1),
                None => {
                    return Err(ErrorObjectOwned::owned(
                        -32602,
                        "Invalid cursor".to_string(),
                        None::<()>,
                    ));
                }
            }
        } else {
            0
        };

        let page: Vec<PersistTxInfo> = items.into_iter().skip(start_idx).take(limit).collect();
        let has_more = start_idx.saturating_add(page.len()) < total_count;
        let next_cursor = if has_more {
            page.last().map(|tx| tx.id.0.clone())
        } else {
            None
        };

        Ok(RuntimePaginatedResponse {
            items: page,
            next_cursor,
            has_more,
            total_count: if include_total {
                Some(total_count)
            } else {
                None
            },
        })
    }
}
