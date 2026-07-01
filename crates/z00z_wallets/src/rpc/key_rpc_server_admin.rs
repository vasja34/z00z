use z00z_crypto::expert::encoding::SafePassword;

impl KeyRpcImpl {
    pub(crate) async fn rotate_master_key_checked(
        &self,
        cap: VerifiedSessionNoTouch,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeRotateKeyResponse> {
        let wallet_id = cap.wallet_id().clone();

        let safe_password = SafePassword::from(password.as_str());

        check_rotate_password(&self.service, &wallet_id, &safe_password).await?;
        check_rotate_confirm(&self.service, &wallet_id, &confirmation).await?;

        match self
            .service
            .rotate_master_key_precheck(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?
        {
            crate::services::RateLimitPrecheck::Allowed => {}
            crate::services::RateLimitPrecheck::RateLimited {
                retry_after_seconds,
                current_count,
                max_requests,
            } => {
                audit_event(
                    &self.service,
                    Some(wallet_id.clone()),
                    "wallet.key.rotate_master_key",
                    AuditResult::RateLimited,
                    Some(format!(
                        "retry_after_seconds={retry_after_seconds},current_count={current_count},max_requests={max_requests}"
                    )),
                )
                .await;

                let data = crate::rpc::types::security::RuntimeRateLimitError {
                    method: "wallet.key.rotate_master_key".to_string(),
                    tier: "rotate_master_key".to_string(),
                    current_count,
                    max_requests,
                    window_seconds: 3_600,
                    retry_after_seconds,
                };

                return Err(ErrorObjectOwned::owned(
                    crate::rpc::types::security::SecurityErrorCode::RateLimitExceeded
                        .code(),
                    crate::rpc::types::security::SecurityErrorCode::RateLimitExceeded
                        .message()
                        .to_string(),
                    Some(data),
                ));
            }
        }

        let response = match finish_rotate(&self.service, cap.session(), &safe_password).await {
            Ok(response) => response,
            Err(error) => {
                self.service
                    .rollback_rotate_master_key_precheck(&wallet_id)
                    .await;
                audit_event(
                    &self.service,
                    Some(wallet_id.clone()),
                    "wallet.key.rotate_master_key",
                    AuditResult::Failure,
                    Some(format!("finish_rotate_failed={error}")),
                )
                .await;
                return Err(error);
            }
        };

        audit_event(
            &self.service,
            Some(wallet_id.clone()),
            "wallet.key.rotate_master_key",
            AuditResult::Success,
            Some(format!("records_rewrapped={}", response.records_rewrapped)),
        )
        .await;

        Ok(response)
    }

    pub(crate) async fn list_receivers_checked(
        &self,
        cap: VerifiedSession,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeReceiverFilter>,
    ) -> RpcResult<RuntimeListReceiversResponse> {
        let wallet_id = cap.wallet_id().clone();
        let limit = validate_limit(limit)?;
        let start = cursor.as_deref().map_or(Ok(0), decode_cursor)?;

        let mut entries = self
            .service
            .list_cached_receivers(&wallet_id)
            .await
            .map_err(|error| ErrorObjectOwned::owned(-32603, error.to_string(), None::<()>))?;
        entries.sort_by_key(|(path, _)| *path);
        apply_receiver_filter(&mut entries, filter.as_ref());

        let total_count = entries.len();
        if start > total_count {
            return Err(invalid_params("Cursor out of range"));
        }

        let end = (start + limit).min(total_count);
        let labels = self.service.get_receiver_labels(&wallet_id).await;
        let slice = &entries[start..end];
        let mut items = Vec::with_capacity(slice.len());

        for (path, public_key) in slice {
            let receiver_id = hex::encode(public_key);
            let reused = entries
                .iter()
                .filter(|(_, cached_public_key)| cached_public_key == public_key)
                .count()
                > 1;
            let label = labels.iter().find_map(|(item, label)| {
                if item == &receiver_id {
                    Some(label.clone())
                } else {
                    None
                }
            });

            items.push(PersistReceiverInfo {
                receiver_id: receiver_id.clone(),
                path: path.to_string(),
                public_key: receiver_id,
                balance: None,
                used: reused,
                internal: path.change().index() == 1,
                label,
                index: path.address_index().index(),
            });
        }

        if let Some(filter) = filter {
            if let Some(used) = filter.used {
                items.retain(|item| item.used == used);
            }
        }

        let has_more = end < total_count;
        Ok(RuntimeListReceiversResponse {
            items,
            next_cursor: has_more.then(|| encode_cursor(end)),
            has_more,
            total_count: Some(total_count),
        })
    }

    pub(crate) async fn label_receiver_checked(
        &self,
        cap: VerifiedSession,
        receiver_id: String,
        label: String,
    ) -> RpcResult<RuntimeLabelReceiverResponse> {
        if label.trim().is_empty() {
            return Err(invalid_params("Label is required"));
        }
        if label.len() > 64 {
            return Err(invalid_params("Label too long (max 64 chars)"));
        }

        let public_key_vec = hex::decode(&receiver_id)
            .map_err(|_| invalid_params("receiver_id must be valid hex"))?;
        let public_key: [u8; 32] = public_key_vec
            .try_into()
            .map_err(|_| invalid_params("receiver_id must be 32 bytes"))?;

        let wallet_id = cap.wallet_id().clone();
        let cached = self
            .service
            .list_cached_receivers(&wallet_id)
            .await
            .map_err(|error| ErrorObjectOwned::owned(-32603, error.to_string(), None::<()>))?;

        let found = cached
            .iter()
            .any(|(_, cached_public_key)| cached_public_key == &public_key);
        if !found {
            return Err(not_found("Receiver not found in wallet"));
        }

        self.service
            .upsert_receiver_label(&wallet_id, receiver_id.clone(), label.clone())
            .await;

        Ok(RuntimeLabelReceiverResponse {
            receiver_id,
            label,
            status: RuntimeOperationStatus {
                success: true,
                message: "ok".to_string(),
            },
        })
    }
}
