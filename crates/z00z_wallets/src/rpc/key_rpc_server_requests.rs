impl KeyRpcImpl {
    pub(crate) async fn create_payment_request_checked(
        &self,
        cap: VerifiedSession,
        amount: Option<u64>,
        expiry_secs: u64,
        metadata: Option<RuntimePaymentRequestMetaInput>,
    ) -> RpcResult<RuntimeCreatePaymentRequestResponse> {
        if expiry_secs == 0 {
            return Err(invalid_params("expiry_secs must be greater than 0"));
        }

        let wallet_id = cap.wallet_id().clone();
        let payment_id = parse_payment_id(&metadata)?;
        let keys = self
            .service
            .receiver_keys(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
        let chain_id = wallet_chain_id(&self.service, &wallet_id).await?;

        let request = PaymentRequest::generate(
            &keys,
            make_req_params(amount, expiry_secs, metadata, payment_id),
            chain_id,
        )
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        request
            .verify()
            .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

        Ok(req_response(&request))
    }

    pub(crate) async fn validate_payment_request_checked(
        &self,
        cap: VerifiedSession,
        request_compact: String,
    ) -> RpcResult<RuntimeValidatePaymentRequestResponse> {
        let wallet_id = cap.wallet_id().clone();
        let request = decode_request_compact(&request_compact).map_err(map_req_decode_err)?;
        let chain_id = wallet_chain_id(&self.service, &wallet_id).await?;

        let mut pins = self
            .service
            .load_tofu(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

        let outcome = request
            .validate_all(&mut pins, chain_id)
            .map_err(map_req_validate_err)?;

        self.service
            .save_tofu(&wallet_id, &pins)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

        Ok(validate_req_response(&request, outcome))
    }

    pub(crate) async fn export_public_material_checked(
        &self,
        cap: VerifiedSession,
        account: u32,
        password: String,
    ) -> RpcResult<RuntimePubMaterialExportResponse> {
        let wallet_id = cap.wallet_id().clone();
        let response = build_pub_export(&self.service, &wallet_id, account, password).await?;

        audit_event(
            &self.service,
            Some(wallet_id.clone()),
            "wallet.key.export_public_material",
            AuditResult::Success,
            Some(format!("account={account}")),
        )
        .await;

        Ok(response)
    }
}
