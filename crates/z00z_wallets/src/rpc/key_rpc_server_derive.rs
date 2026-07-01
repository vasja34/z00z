impl KeyRpcImpl {
    pub(crate) async fn derive_receiver_checked(
        &self,
        cap: VerifiedSessionNoTouch,
        path: String,
    ) -> RpcResult<RuntimeDeriveReceiverResponse> {
        let wallet_id = cap.wallet_id().clone();

        self.service
            .key_derive_rate_limit_precheck(&wallet_id, 20)
            .await
            .map_err(|e| ErrorObjectOwned::owned(-32000, e, None::<()>))?;

        let parsed_path: Bip44Path = path.parse().map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("Invalid BIP44 path: {e}"), None::<()>)
        })?;

        let public_key_bytes = self
            .service
            .derive_public_key_for_path(&wallet_id, parsed_path)
            .await
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        Ok(RuntimeDeriveReceiverResponse {
            public_key: hex::encode(public_key_bytes),
            path: parsed_path.to_string(),
        })
    }

    pub(crate) async fn get_receiver_card_checked(
        &self,
        cap: VerifiedSession,
    ) -> RpcResult<RuntimeGetReceiverCardResponse> {
        let wallet_id = cap.wallet_id().clone();
        let keys = self
            .service
            .receiver_keys(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

        let card = keys
            .export_receiver_card()
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        let record = ReceiverCardRecord::new(&card, card.canonical_encoding(), 0)
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        let card_compact = record
            .to_compact()
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
        let decoded_record = ReceiverCardRecord::from_compact(&card_compact, None)
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
        let decoded = decoded_record
            .decode_card()
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
        decoded
            .validate_signature()
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        let owner_handle_display = format_receiver_handle(&decoded.owner_handle)
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        Ok(RuntimeGetReceiverCardResponse {
            owner_handle: hex::encode(decoded.owner_handle),
            view_key: hex::encode(decoded.view_pk),
            identity_key: hex::encode(decoded.identity_pk),
            signature: hex::encode(decoded.signature),
            card_compact,
            registry_entry_id: hex::encode(decoded_record.registry_entry_id),
            card_epoch: decoded_record.card_epoch,
            owner_handle_display,
        })
    }

    async fn validate_receiver_card_impl(
        &self,
        card_compact: String,
    ) -> RpcResult<RuntimeValidateReceiverCardResponse> {
        match ReceiverCardRecord::from_compact(&card_compact, None) {
            Ok(_) => Ok(RuntimeValidateReceiverCardResponse {
                result: RuntimeValidationResult::valid(),
                format: Some("receiver_card".to_string()),
            }),
            Err(error) => Ok(RuntimeValidateReceiverCardResponse {
                result: RuntimeValidationResult::invalid(error.to_string()),
                format: None,
            }),
        }
    }
}
