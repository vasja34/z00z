#![allow(missing_docs)]

use std::collections::HashSet;

use thiserror::Error;
use z00z_storage::settlement::{CheckRoot, ProofItem, SettlementPath, TerminalLeaf};

use crate::{
    key::ReceiverSecret,
    tx::{
        prover::{Prover, ProverImpl},
        spend_rules::{verify_spend_rules, SpendIn, SpendStmt},
        state_witness::{proof_root, MemberWit},
        tx_wire::SPEND_PROOF_SUITE,
        ResolvedInput, SpendInputLeaf, SpendInputRef,
    },
};

const SPEND_PROOF_ARTIFACT_PREFIX: &[u8] = b"z00z.spend.proof.backend.v2";
const SPEND_PROOF_HASH_LEN: usize = 32;
const SPEND_PROOF_PUB_HASH_CTX: &[u8] = b"z00z.spend.public.hash.v1";
const SPEND_PROOF_THEOREM_CTX: &[u8] = b"z00z.spend.theorem.proof.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendProofStmt {
    pub statement: Vec<u8>,
    pub pkg_digest: [u8; 32],
    pub prev_root: CheckRoot,
    pub chain_id: u32,
    pub tx_version: u8,
    pub chain_type: String,
    pub chain_name: String,
    pub input_refs: Vec<SpendInputRef>,
    pub input_leaves: Vec<SpendInputLeaf>,
    /// Output theorem leaves use `TerminalLeaf.terminal_id()` as the public `leaf_ad_id` namespace.
    pub output_leaves: Vec<TerminalLeaf>,
    pub nullifiers: Vec<[u8; 32]>,
}

impl SpendProofStmt {
    pub fn new(statement: Vec<u8>) -> Result<Self, SpendProofBackendError> {
        if statement.is_empty() {
            return Err(SpendProofBackendError::EmptyStatement);
        }
        Ok(Self {
            statement,
            pkg_digest: [0u8; 32],
            prev_root: CheckRoot::new([0u8; 32]),
            chain_id: 0,
            tx_version: 0,
            chain_type: String::new(),
            chain_name: String::new(),
            input_refs: Vec::new(),
            input_leaves: Vec::new(),
            output_leaves: Vec::new(),
            nullifiers: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        statement: Vec<u8>,
        pkg_digest: [u8; 32],
        prev_root: CheckRoot,
        chain_id: u32,
        tx_version: u8,
        chain_type: String,
        chain_name: String,
        input_refs: Vec<SpendInputRef>,
        input_leaves: Vec<SpendInputLeaf>,
        output_leaves: Vec<TerminalLeaf>,
        nullifiers: Vec<[u8; 32]>,
    ) -> Result<Self, SpendProofBackendError> {
        if statement.is_empty() {
            return Err(SpendProofBackendError::EmptyStatement);
        }
        if input_refs.len() != input_leaves.len() || input_refs.len() != nullifiers.len() {
            return Err(SpendProofBackendError::StatementShapeMismatch);
        }
        Ok(Self {
            statement,
            pkg_digest,
            prev_root,
            chain_id,
            tx_version,
            chain_type,
            chain_name,
            input_refs,
            input_leaves,
            output_leaves,
            nullifiers,
        })
    }

    fn statement_hash(&self) -> [u8; 32] {
        *blake3::hash(&self.statement).as_bytes()
    }

    fn public_hash(&self) -> [u8; 32] {
        let stmt_hash = self.statement_hash();
        let mut bytes = Vec::with_capacity(SPEND_PROOF_PUB_HASH_CTX.len() + stmt_hash.len());
        bytes.extend_from_slice(SPEND_PROOF_PUB_HASH_CTX);
        bytes.extend_from_slice(&stmt_hash);
        *blake3::hash(&bytes).as_bytes()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendMembershipWitness {
    pub path: SettlementPath,
    pub leaf: TerminalLeaf,
    pub proof: Vec<u8>,
    pub proof_item: ProofItem,
}

impl SpendMembershipWitness {
    pub fn new(
        path: SettlementPath,
        leaf: TerminalLeaf,
        proof: Vec<u8>,
        proof_item: ProofItem,
    ) -> Result<Self, SpendProofBackendError> {
        let witness = Self {
            path,
            leaf,
            proof,
            proof_item,
        };
        let member_wit = witness.member_wit()?;
        if member_wit.proof_item().path() != witness.path
            || member_wit
                .proof_item()
                .terminal_leaf()
                .map_err(|_| SpendProofBackendError::MembershipWitnessMismatch)?
                != &witness.leaf
        {
            return Err(SpendProofBackendError::MembershipWitnessMismatch);
        }
        Ok(witness)
    }

    #[must_use]
    pub fn from_resolved(input: &ResolvedInput) -> Self {
        Self {
            path: input.path(),
            leaf: input.leaf().clone(),
            proof: input.member_wit().proof().to_vec(),
            proof_item: input.member_wit().proof_item().clone(),
        }
    }

    fn member_wit(&self) -> Result<MemberWit, SpendProofBackendError> {
        MemberWit::new(self.proof.clone(), self.proof_item.clone())
            .map_err(|_| SpendProofBackendError::MembershipWitnessMismatch)
    }
}

pub struct SpendProofWitness {
    pub receiver_secret: ReceiverSecret,
    pub input_s_in: Vec<[u8; 32]>,
    pub membership: Vec<SpendMembershipWitness>,
}

impl SpendProofWitness {
    pub fn new(
        receiver_secret: ReceiverSecret,
        input_s_in: Vec<[u8; 32]>,
        membership: Vec<SpendMembershipWitness>,
    ) -> Self {
        Self {
            receiver_secret,
            input_s_in,
            membership,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendProofArtifact {
    pub proof_hex: String,
    pub pub_hash_hex: String,
}

impl SpendProofArtifact {
    pub fn from_wire_hex(proof_hex: &str) -> Result<Self, SpendProofBackendError> {
        let proof_bytes = decode_canonical_hex(proof_hex)?;
        let DecodedArtifact::CanonicalTheorem { public_hash, .. } =
            decode_artifact_bytes(&proof_bytes)?;
        Ok(Self {
            proof_hex: hex::encode(proof_bytes),
            pub_hash_hex: hex::encode(public_hash),
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpendProofBackendError {
    #[error("empty spend statement")]
    EmptyStatement,
    #[error("typed spend statement shape mismatch")]
    StatementShapeMismatch,
    #[error("invalid spend proof hex")]
    InvalidProofHex,
    #[error("invalid spend proof payload")]
    InvalidProofPayload,
    #[error("spend proof payload does not bind the carried statement")]
    StatementMismatch,
    #[error("spend proof payload public hash mismatch")]
    PublicHashMismatch,
    #[error("empty spend proof witness")]
    EmptyWitness,
    #[error("spend proof witness input count mismatch")]
    WitnessInputMismatch,
    #[error("missing spend membership witness")]
    MissingMembershipWitness,
    #[error("spend membership witness does not match carried public inputs")]
    MembershipWitnessMismatch,
    #[error("spend proof witness does not satisfy carried public inputs")]
    WitnessRelationMismatch,
    #[error("spend output range relation mismatch")]
    RangeRelationMismatch,
    #[error("unsupported spend proof suite")]
    UnsupportedSuite,
    #[error("missing theorem proof bytes")]
    MissingTheoremProof,
    #[error("theorem proof does not satisfy spend relations")]
    TheoremRelationMismatch,
}

pub trait SpendProofBackend {
    fn suite_id(&self) -> &'static str;

    fn prove(
        &self,
        stmt: &SpendProofStmt,
        wit: &SpendProofWitness,
    ) -> Result<SpendProofArtifact, SpendProofBackendError>;

    fn verify(
        &self,
        stmt: &SpendProofStmt,
        proof: &SpendProofArtifact,
    ) -> Result<(), SpendProofBackendError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CanonicalSpendProofBackend;

pub fn default_spend_proof_backend() -> CanonicalSpendProofBackend {
    CanonicalSpendProofBackend
}

impl CanonicalSpendProofBackend {
    fn validate_statement_shape(
        &self,
        stmt: &SpendProofStmt,
    ) -> Result<usize, SpendProofBackendError> {
        let input_count = stmt.input_refs.len();
        if input_count == 0
            || stmt.input_leaves.is_empty()
            || stmt.nullifiers.is_empty()
            || stmt.output_leaves.is_empty()
        {
            return Err(SpendProofBackendError::StatementShapeMismatch);
        }
        if stmt.input_leaves.len() != input_count || stmt.nullifiers.len() != input_count {
            return Err(SpendProofBackendError::StatementShapeMismatch);
        }
        for (input_ref, input_leaf) in stmt.input_refs.iter().zip(stmt.input_leaves.iter()) {
            if input_ref.asset_id != input_leaf.asset_id
                || input_ref.serial_id != input_leaf.serial_id
            {
                return Err(SpendProofBackendError::StatementShapeMismatch);
            }
        }
        Ok(input_count)
    }

    fn validate_membership(
        &self,
        stmt: &SpendProofStmt,
        wit: &SpendProofWitness,
        input_count: usize,
    ) -> Result<(), SpendProofBackendError> {
        if wit.membership.is_empty() {
            return Err(SpendProofBackendError::MissingMembershipWitness);
        }
        if wit.membership.len() != input_count {
            return Err(SpendProofBackendError::WitnessInputMismatch);
        }

        let want_root = proof_root(stmt.prev_root);
        for ((input_ref, input_leaf), member) in stmt
            .input_refs
            .iter()
            .zip(stmt.input_leaves.iter())
            .zip(wit.membership.iter())
        {
            if member.path.terminal_id().into_bytes() != input_ref.asset_id
                || member.path.serial_id.get() != input_ref.serial_id
            {
                return Err(SpendProofBackendError::MembershipWitnessMismatch);
            }
            if member.leaf.asset_id != input_leaf.asset_id
                || member.leaf.serial_id != input_leaf.serial_id
                || member.leaf.r_pub != input_leaf.r_pub
                || member.leaf.owner_tag != input_leaf.owner_tag
                || member.leaf.c_amount != input_leaf.c_amt
            {
                return Err(SpendProofBackendError::MembershipWitnessMismatch);
            }

            let member_wit = member.member_wit()?;
            if member_wit.proof_root() != want_root
                || member_wit.proof_item().path() != member.path
                || member_wit
                    .proof_item()
                    .terminal_leaf()
                    .map_err(|_| SpendProofBackendError::MembershipWitnessMismatch)?
                    != &member.leaf
            {
                return Err(SpendProofBackendError::MembershipWitnessMismatch);
            }
            member_wit
                .check(want_root, &member.path, &member.leaf)
                .map_err(|_| SpendProofBackendError::MembershipWitnessMismatch)?;
        }
        Ok(())
    }

    fn validate_output_ranges(&self, stmt: &SpendProofStmt) -> Result<(), SpendProofBackendError> {
        let prover =
            ProverImpl::new().map_err(|_| SpendProofBackendError::RangeRelationMismatch)?;
        for leaf in &stmt.output_leaves {
            if leaf.range_proof.is_empty() {
                return Err(SpendProofBackendError::RangeRelationMismatch);
            }
            let ok = prover
                .verify_proof(&leaf.range_proof, &leaf.c_amount)
                .map_err(|_| SpendProofBackendError::RangeRelationMismatch)?;
            if !ok {
                return Err(SpendProofBackendError::RangeRelationMismatch);
            }
        }
        Ok(())
    }

    fn validate_public_relations(
        &self,
        stmt: &SpendProofStmt,
    ) -> Result<(), SpendProofBackendError> {
        self.validate_output_ranges(stmt)?;

        let mut input_refs = HashSet::new();
        let mut input_leaf_ids = HashSet::new();
        let mut nullifiers = HashSet::new();
        for ((input_ref, input_leaf), nullifier) in stmt
            .input_refs
            .iter()
            .zip(stmt.input_leaves.iter())
            .zip(stmt.nullifiers.iter())
        {
            if !input_refs.insert((input_ref.asset_id, input_ref.serial_id)) {
                return Err(SpendProofBackendError::TheoremRelationMismatch);
            }
            if !input_leaf_ids.insert(input_leaf.leaf_ad_id) {
                return Err(SpendProofBackendError::TheoremRelationMismatch);
            }
            if !nullifiers.insert(*nullifier) {
                return Err(SpendProofBackendError::TheoremRelationMismatch);
            }
        }

        let mut output_leaf_ids = HashSet::new();
        for output_leaf in &stmt.output_leaves {
            if !output_leaf_ids.insert(output_leaf.asset_id) {
                return Err(SpendProofBackendError::TheoremRelationMismatch);
            }
        }
        if !input_leaf_ids.is_disjoint(&output_leaf_ids) {
            return Err(SpendProofBackendError::TheoremRelationMismatch);
        }

        let input_commitments = stmt
            .input_leaves
            .iter()
            .map(|leaf| {
                z00z_crypto::Commitment::from_bytes(&leaf.c_amt)
                    .map(|commitment| commitment.as_commitment().clone())
                    .map_err(|_| SpendProofBackendError::StatementShapeMismatch)
            })
            .collect::<Result<Vec<_>, SpendProofBackendError>>()?;
        let output_commitments = stmt
            .output_leaves
            .iter()
            .map(|leaf| {
                z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
                    .map(|commitment| commitment.as_commitment().clone())
                    .map_err(|_| SpendProofBackendError::StatementShapeMismatch)
            })
            .collect::<Result<Vec<_>, SpendProofBackendError>>()?;
        let input_sum = input_commitments
            .iter()
            .skip(1)
            .fold(input_commitments[0].clone(), |accumulator, commitment| {
                &accumulator + commitment
            });
        let output_sum = output_commitments
            .iter()
            .skip(1)
            .fold(output_commitments[0].clone(), |accumulator, commitment| {
                &accumulator + commitment
            });
        if input_sum != output_sum {
            return Err(SpendProofBackendError::TheoremRelationMismatch);
        }

        Ok(())
    }

    fn validate_witness(
        &self,
        stmt: &SpendProofStmt,
        wit: &SpendProofWitness,
    ) -> Result<(), SpendProofBackendError> {
        let input_count = self.validate_statement_shape(stmt)?;
        if wit.input_s_in.is_empty() {
            return Err(SpendProofBackendError::EmptyWitness);
        }
        if wit.input_s_in.len() != input_count {
            return Err(SpendProofBackendError::WitnessInputMismatch);
        }
        self.validate_membership(stmt, wit, input_count)?;
        self.validate_output_ranges(stmt)?;

        let receiver_secret = ReceiverSecret::from_bytes(*wit.receiver_secret.as_bytes())
            .map_err(|_| SpendProofBackendError::WitnessRelationMismatch)?;
        let spend_ins = stmt
            .input_leaves
            .iter()
            .zip(wit.input_s_in.iter())
            .zip(stmt.nullifiers.iter())
            .map(|((leaf, s_in), nullifier)| {
                let c_in = z00z_crypto::Commitment::from_bytes(&leaf.c_amt)
                    .map_err(|_| SpendProofBackendError::StatementShapeMismatch)?
                    .as_commitment()
                    .clone();
                Ok(SpendIn {
                    chain_id: stmt.chain_id,
                    r_pub_in: leaf.r_pub,
                    owner_tag_in: leaf.owner_tag,
                    leaf_ad_id_in: leaf.leaf_ad_id,
                    nullifier_in: Some(*nullifier),
                    s_in: *s_in,
                    c_in,
                })
            })
            .collect::<Result<Vec<_>, SpendProofBackendError>>()?;
        let c_outs = stmt
            .output_leaves
            .iter()
            .map(|leaf| {
                z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
                    .map(|commitment| commitment.as_commitment().clone())
                    .map_err(|_| SpendProofBackendError::StatementShapeMismatch)
            })
            .collect::<Result<Vec<_>, SpendProofBackendError>>()?;
        let rules_stmt = SpendStmt {
            receiver_secret,
            spend_ins,
            c_outs,
            range_ok: true,
        };
        verify_spend_rules(&rules_stmt)
            .map_err(|_| SpendProofBackendError::WitnessRelationMismatch)?;
        Ok(())
    }

    fn theorem_bytes(&self, stmt: &SpendProofStmt) -> Vec<u8> {
        let statement_hash = stmt.statement_hash();
        let public_hash = stmt.public_hash();
        let mut proof_bytes = Vec::with_capacity(
            SPEND_PROOF_THEOREM_CTX.len() + SPEND_PROOF_SUITE.len() + (SPEND_PROOF_HASH_LEN * 2),
        );
        proof_bytes.extend_from_slice(SPEND_PROOF_THEOREM_CTX);
        proof_bytes.extend_from_slice(SPEND_PROOF_SUITE.as_bytes());
        proof_bytes.extend_from_slice(&statement_hash);
        proof_bytes.extend_from_slice(&public_hash);
        blake3::hash(&proof_bytes).as_bytes().to_vec()
    }

    fn encode_artifact(&self, stmt: &SpendProofStmt) -> SpendProofArtifact {
        let stmt_hash = stmt.statement_hash();
        let pub_hash = stmt.public_hash();
        let theorem_bytes = self.theorem_bytes(stmt);
        let mut proof_bytes = Vec::with_capacity(
            SPEND_PROOF_ARTIFACT_PREFIX.len()
                + 1
                + SPEND_PROOF_SUITE.len()
                + (SPEND_PROOF_HASH_LEN * 2)
                + 4
                + theorem_bytes.len(),
        );
        proof_bytes.extend_from_slice(SPEND_PROOF_ARTIFACT_PREFIX);
        proof_bytes.push(SPEND_PROOF_SUITE.len() as u8);
        proof_bytes.extend_from_slice(SPEND_PROOF_SUITE.as_bytes());
        proof_bytes.extend_from_slice(&stmt_hash);
        proof_bytes.extend_from_slice(&pub_hash);
        proof_bytes.extend_from_slice(&(theorem_bytes.len() as u32).to_le_bytes());
        proof_bytes.extend_from_slice(&theorem_bytes);
        SpendProofArtifact {
            proof_hex: hex::encode(proof_bytes),
            pub_hash_hex: hex::encode(pub_hash),
        }
    }
}

impl SpendProofBackend for CanonicalSpendProofBackend {
    fn suite_id(&self) -> &'static str {
        SPEND_PROOF_SUITE
    }

    fn prove(
        &self,
        stmt: &SpendProofStmt,
        wit: &SpendProofWitness,
    ) -> Result<SpendProofArtifact, SpendProofBackendError> {
        self.validate_witness(stmt, wit)?;
        Ok(self.encode_artifact(stmt))
    }

    fn verify(
        &self,
        stmt: &SpendProofStmt,
        proof: &SpendProofArtifact,
    ) -> Result<(), SpendProofBackendError> {
        self.validate_statement_shape(stmt)?;
        self.validate_public_relations(stmt)?;
        let proof_bytes = decode_canonical_hex(&proof.proof_hex)?;
        let DecodedArtifact::CanonicalTheorem {
            statement_hash,
            public_hash,
            theorem_bytes,
        } = decode_artifact_bytes(&proof_bytes)?;
        if stmt.statement_hash() != statement_hash {
            return Err(SpendProofBackendError::StatementMismatch);
        }
        let expected_pub_hash = stmt.public_hash();
        if expected_pub_hash != public_hash {
            return Err(SpendProofBackendError::PublicHashMismatch);
        }
        if !proof.pub_hash_hex.is_empty() && proof.pub_hash_hex != hex::encode(expected_pub_hash) {
            return Err(SpendProofBackendError::PublicHashMismatch);
        }
        if theorem_bytes != self.theorem_bytes(stmt) {
            return Err(SpendProofBackendError::TheoremRelationMismatch);
        }
        Ok(())
    }
}

fn decode_canonical_hex(value: &str) -> Result<Vec<u8>, SpendProofBackendError> {
    let bytes = hex::decode(value).map_err(|_| SpendProofBackendError::InvalidProofHex)?;
    if hex::encode(&bytes) != value {
        return Err(SpendProofBackendError::InvalidProofHex);
    }
    Ok(bytes)
}

enum DecodedArtifact {
    CanonicalTheorem {
        statement_hash: [u8; 32],
        public_hash: [u8; 32],
        theorem_bytes: Vec<u8>,
    },
}

fn decode_artifact_bytes(proof_bytes: &[u8]) -> Result<DecodedArtifact, SpendProofBackendError> {
    if !proof_bytes.starts_with(SPEND_PROOF_ARTIFACT_PREFIX) {
        return Err(SpendProofBackendError::InvalidProofPayload);
    }

    let mut cursor = SPEND_PROOF_ARTIFACT_PREFIX.len();
    let suite_len = *proof_bytes
        .get(cursor)
        .ok_or(SpendProofBackendError::InvalidProofPayload)? as usize;
    cursor += 1;

    let suite_end = cursor + suite_len;
    let suite_id = std::str::from_utf8(
        proof_bytes
            .get(cursor..suite_end)
            .ok_or(SpendProofBackendError::InvalidProofPayload)?,
    )
    .map_err(|_| SpendProofBackendError::InvalidProofPayload)?
    .to_string();
    if suite_id != SPEND_PROOF_SUITE {
        return Err(SpendProofBackendError::UnsupportedSuite);
    }
    cursor = suite_end;

    let statement_hash: [u8; 32] = proof_bytes
        .get(cursor..cursor + SPEND_PROOF_HASH_LEN)
        .ok_or(SpendProofBackendError::InvalidProofPayload)?
        .try_into()
        .map_err(|_| SpendProofBackendError::InvalidProofPayload)?;
    cursor += SPEND_PROOF_HASH_LEN;
    let public_hash: [u8; 32] = proof_bytes
        .get(cursor..cursor + SPEND_PROOF_HASH_LEN)
        .ok_or(SpendProofBackendError::InvalidProofPayload)?
        .try_into()
        .map_err(|_| SpendProofBackendError::InvalidProofPayload)?;
    cursor += SPEND_PROOF_HASH_LEN;
    let proof_len = u32::from_le_bytes(
        proof_bytes
            .get(cursor..cursor + 4)
            .ok_or(SpendProofBackendError::InvalidProofPayload)?
            .try_into()
            .map_err(|_| SpendProofBackendError::InvalidProofPayload)?,
    ) as usize;
    cursor += 4;

    let proof_end = cursor + proof_len;
    let theorem_bytes = proof_bytes
        .get(cursor..proof_end)
        .ok_or(SpendProofBackendError::InvalidProofPayload)?;
    if proof_end != proof_bytes.len() {
        return Err(SpendProofBackendError::InvalidProofPayload);
    }
    if theorem_bytes.is_empty() {
        return Err(SpendProofBackendError::MissingTheoremProof);
    }

    Ok(DecodedArtifact::CanonicalTheorem {
        statement_hash,
        public_hash,
        theorem_bytes: theorem_bytes.to_vec(),
    })
}
