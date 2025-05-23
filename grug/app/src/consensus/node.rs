// A lot of this is mock code (at best)
// The only purpose is discovery & prototyping
// 22.05.2025

use std::path::PathBuf;

use async_trait::async_trait;
use base64;
use eyre::eyre;
use malachitebft_app::node::{
    CanGeneratePrivateKey, CanMakeConfig, CanMakeGenesis, CanMakePrivateKeyFile, EngineHandle,
    MakeConfigSettings, Node, NodeConfig, NodeHandle,
};
use malachitebft_app::types::Keypair;
use malachitebft_app::types::core::{Context, PrivateKey, PublicKey, VotingPower};
use malachitebft_config::{ConsensusConfig, ValueSyncConfig};
use malachitebft_core_types::{
    Address, Extension, Height, NilOrVal, Proposal, ProposalPart, Round, SignedMessage,
    SigningProvider, SigningScheme, Validator, ValidatorSet as VS, Value, Vote,
    VoteType as CoreVoteType,
};
use malachitebft_engine::node::NodeRef;
use malachitebft_engine::util::events::RxEvent;
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub struct GrugNode {
    pub home_dir: PathBuf,
}

// The node wraps over all the relevant actors that are needed to operate
// a node, including consensus, app, mempool, p2p, sync, wal
impl GrugNode {
    pub fn new(home_dir: PathBuf) -> Self {
        Self { home_dir }
    }

    pub fn genesis_file(&self) -> PathBuf {
        self.home_dir.join("config").join("genesis.json")
    }

    pub fn private_key_file(&self) -> PathBuf {
        self.home_dir.join("config").join("priv_validator_key.json")
    }
}

// Define types needed for Node implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrugContext;

// Define our custom types for context implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrugAddress(pub String);

impl std::fmt::Display for GrugAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Address for GrugAddress {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct GrugHeight(pub u64);

impl std::fmt::Display for GrugHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Height for GrugHeight {
    const ZERO: Self = GrugHeight(0);
    const INITIAL: Self = GrugHeight(1);

    fn increment_by(&self, n: u64) -> Self {
        GrugHeight(self.0 + n)
    }

    fn decrement_by(&self, n: u64) -> Option<Self> {
        if self.0 > n {
            Some(GrugHeight(self.0 - n))
        } else {
            None
        }
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrugValue(pub Vec<u8>);

impl Value for GrugValue {
    type Id = String;

    fn id(&self) -> Self::Id {
        // Convert bytes to base64 string for a displayable ID
        base64::encode(&self.0)
    }
}

// Implement hash trait for serialization/comparison purposes
// Already moved this implementation next to the GrugValue impl

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrugValidator {
    pub address: GrugAddress,
    pub public_key: PublicKey<GrugContext>,
    pub voting_power: VotingPower,
}

impl Validator<GrugContext> for GrugValidator {
    fn address(&self) -> &GrugAddress {
        &self.address
    }

    fn voting_power(&self) -> VotingPower {
        self.voting_power
    }

    fn public_key(&self) -> &Vec<u8> {
        // In a real implementation, this would convert the public key to the required format
        &self.public_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrugProposal {
    pub height: GrugHeight,
    pub round: Round,
    pub value: GrugValue,
    pub proposer: GrugAddress,
}

// Custom implementation for Default
impl Default for GrugProposal {
    fn default() -> Self {
        Self {
            height: GrugHeight(0),
            round: 0u32.into(),
            value: GrugValue(vec![]),
            proposer: GrugAddress("default".to_string()),
        }
    }
}

impl Proposal<GrugContext> for GrugProposal {
    fn height(&self) -> GrugHeight {
        self.height
    }

    fn round(&self) -> Round {
        self.round
    }

    fn value(&self) -> &GrugValue {
        &self.value
    }

    fn take_value(self) -> GrugValue {
        self.value
    }

    fn pol_round(&self) -> Round {
        // Default POL round, could be stored as a field if needed
        0u32.into()
    }

    fn validator_address(&self) -> &GrugAddress {
        &self.proposer
    }
}

// NilOrVal serialization

mod nil_or_val_serde {
    use malachitebft_core_types::NilOrVal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T>(val: &NilOrVal<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match val {
            NilOrVal::Nil => serializer.serialize_none(),
            NilOrVal::Val(v) => serializer.serialize_some(v),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<NilOrVal<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|opt| match opt {
            None => NilOrVal::Nil,
            Some(v) => NilOrVal::Val(v),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrugProposalPart {
    pub height: GrugHeight,
    pub value_part: Vec<u8>,
}

impl ProposalPart<GrugContext> for GrugProposalPart {
    fn is_first(&self) -> bool {
        // Assuming all parts are both first and last for simplicity
        true
    }

    fn is_last(&self) -> bool {
        // Assuming all parts are both first and last for simplicity
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrugVote {
    pub height: GrugHeight,
    pub round: Round,
    pub value_id: NilOrVal<String>,
    pub voter: GrugAddress,
    pub vote_type: VoteType,
    pub extension: Option<SignedMessage<GrugContext, GrugExtension>>,
}

// Custom implementation for Default
impl Default for GrugVote {
    fn default() -> Self {
        Self {
            height: GrugHeight(0),
            round: 0u32.into(),
            value_id: NilOrVal::Nil,
            voter: GrugAddress("default".to_string()),
            vote_type: VoteType::Prevote,
            extension: None,
        }
    }
}

impl Vote<GrugContext> for GrugVote {
    fn height(&self) -> GrugHeight {
        self.height
    }

    fn round(&self) -> Round {
        self.round
    }

    fn value(&self) -> &NilOrVal<String> {
        &self.value_id
    }

    fn take_value(self) -> NilOrVal<String> {
        self.value_id
    }

    fn vote_type(&self) -> CoreVoteType {
        match self.vote_type {
            VoteType::Prevote => CoreVoteType::Prevote,
            VoteType::Precommit => CoreVoteType::Precommit,
        }
    }

    fn validator_address(&self) -> &GrugAddress {
        &self.voter
    }

    fn extension(&self) -> Option<&SignedMessage<GrugContext, GrugExtension>> {
        self.extension.as_ref()
    }

    fn take_extension(&mut self) -> Option<SignedMessage<GrugContext, GrugExtension>> {
        self.extension.take()
    }

    fn extend(mut self, ext: SignedMessage<GrugContext, GrugExtension>) -> Self {
        self.extension = Some(ext);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VoteType {
    Prevote,
    Precommit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GrugExtension {
    pub height: GrugHeight,
    pub data: Vec<u8>,
}

impl Extension for GrugExtension {
    fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ed25519SigningScheme;

impl SigningScheme for Ed25519SigningScheme {
    type PublicKey = Vec<u8>;
    type Signature = Vec<u8>;
    type PrivateKey = Vec<u8>;
    type DecodingError = String;

    fn decode_signature(bytes: &[u8]) -> Result<Self::Signature, Self::DecodingError> {
        Ok(bytes.to_vec())
    }

    fn encode_signature(signature: &Self::Signature) -> Vec<u8> {
        signature.clone()
    }
}

// Implement ValidatorSet trait for our validator set type
impl VS<GrugContext> for Vec<GrugValidator> {
    fn count(&self) -> usize {
        self.len()
    }

    fn total_voting_power(&self) -> u64 {
        self.iter().map(|v| v.voting_power).sum()
    }

    fn get_by_address(&self, addr: &GrugAddress) -> Option<&GrugValidator> {
        self.iter().find(|v| &v.address == addr)
    }

    fn get_by_index(&self, idx: usize) -> Option<&GrugValidator> {
        self.get(idx)
    }
}

// For making our types hashable
impl std::hash::Hash for GrugAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::hash::Hash for GrugHeight {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::hash::Hash for GrugValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::hash::Hash for VoteType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Context for GrugContext {
    type Address = GrugAddress;
    type Height = GrugHeight;
    type Value = GrugValue;
    type ValidatorSet = Vec<GrugValidator>;
    type ProposalPart = GrugProposalPart;
    type Proposal = GrugProposal;
    type Validator = GrugValidator;
    type Vote = GrugVote;
    type Extension = GrugExtension;
    type SigningScheme = Ed25519SigningScheme;

    fn select_proposer<'a>(
        &self,
        validator_set: &'a Self::ValidatorSet,
        _height: Self::Height,
        _round: Round,
    ) -> &'a Self::Validator {
        // Simple implementation: always choose the first validator
        &validator_set[0]
    }

    fn new_proposal(
        &self,
        height: Self::Height,
        round: Round,
        value: Self::Value,
        _valid_round: Round,
        proposer_address: Self::Address,
    ) -> Self::Proposal {
        GrugProposal {
            height,
            round,
            value,
            proposer: proposer_address,
        }
    }

    fn new_prevote(
        &self,
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<<Self::Value as Value>::Id>,
        voter_address: Self::Address,
    ) -> Self::Vote {
        GrugVote {
            height,
            round,
            value_id,
            voter: voter_address,
            vote_type: VoteType::Prevote,
            extension: None,
        }
    }

    fn new_precommit(
        &self,
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<<Self::Value as Value>::Id>,
        voter_address: Self::Address,
    ) -> Self::Vote {
        GrugVote {
            height,
            round,
            value_id,
            voter: voter_address,
            vote_type: VoteType::Precommit,
            extension: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrugConfig {
    pub moniker: String,
    pub consensus: ConsensusConfig,
    pub value_sync: ValueSyncConfig,
}

impl NodeConfig for GrugConfig {
    fn moniker(&self) -> &str {
        &self.moniker
    }

    fn consensus(&self) -> &ConsensusConfig {
        &self.consensus
    }

    fn value_sync(&self) -> &ValueSyncConfig {
        &self.value_sync
    }
}

// We only need one NodeConfig implementation for GrugConfig

#[derive(Debug, Serialize, Deserialize)]
pub struct GrugGenesis {
    pub validators: Vec<GrugValidator>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrugPrivateKeyFile {
    pub private_key: String,
}

pub struct GrugSigningProvider {
    private_key: PrivateKey<GrugContext>,
}

impl SigningProvider<GrugContext> for GrugSigningProvider {
    fn sign_vote(
        &self,
        vote: <GrugContext as Context>::Vote,
    ) -> SignedMessage<GrugContext, <GrugContext as Context>::Vote> {
        // Dummy implementation
        SignedMessage::new(vote, vec![])
    }

    fn verify_signed_vote(
        &self,
        _vote: &<GrugContext as Context>::Vote,
        _signature: &<<GrugContext as Context>::SigningScheme as SigningScheme>::Signature,
        _public_key: &<<GrugContext as Context>::SigningScheme as SigningScheme>::PublicKey,
    ) -> bool {
        // Placeholder implementation
        true
    }

    fn sign_proposal(
        &self,
        proposal: <GrugContext as Context>::Proposal,
    ) -> SignedMessage<GrugContext, <GrugContext as Context>::Proposal> {
        // Dummy implementation
        SignedMessage::new(proposal, vec![])
    }

    fn verify_signed_proposal(
        &self,
        _proposal: &<GrugContext as Context>::Proposal,
        _signature: &<<GrugContext as Context>::SigningScheme as SigningScheme>::Signature,
        _public_key: &<<GrugContext as Context>::SigningScheme as SigningScheme>::PublicKey,
    ) -> bool {
        // Placeholder implementation
        true
    }

    fn sign_proposal_part(
        &self,
        proposal_part: <GrugContext as Context>::ProposalPart,
    ) -> SignedMessage<GrugContext, <GrugContext as Context>::ProposalPart> {
        // Dummy implementation
        SignedMessage::new(proposal_part, vec![])
    }

    fn verify_signed_proposal_part(
        &self,
        _proposal_part: &<GrugContext as Context>::ProposalPart,
        _signature: &<<GrugContext as Context>::SigningScheme as SigningScheme>::Signature,
        _public_key: &<<GrugContext as Context>::SigningScheme as SigningScheme>::PublicKey,
    ) -> bool {
        // Placeholder implementation
        true
    }

    // TODO: Completely remove the vote extension support in Grug
    fn sign_vote_extension(
        &self,
        extension: <GrugContext as Context>::Extension,
    ) -> SignedMessage<GrugContext, <GrugContext as Context>::Extension> {
        // Dummy implementation
        SignedMessage::new(extension, vec![])
    }

    // Not being used in Grug
    // TODO: Completely remove the vote extension support in Grug
    fn verify_signed_vote_extension(
        &self,
        _extension: &<GrugContext as Context>::Extension,
        _signature: &<<GrugContext as Context>::SigningScheme as SigningScheme>::Signature,
        _public_key: &<<GrugContext as Context>::SigningScheme as SigningScheme>::PublicKey,
    ) -> bool {
        // Always true
        true
    }
}

pub struct GrugNodeHandle {
    pub actor: NodeRef,
    pub handle: JoinHandle<()>,
}

#[async_trait]
impl NodeHandle<GrugContext> for GrugNodeHandle {
    fn subscribe(&self) -> RxEvent<GrugContext> {
        // This is a placeholder - in a real implementation,
        // we would return the actual subscription
        panic!("subscribe is not implemented")
    }

    async fn kill(&self, _reason: Option<String>) -> eyre::Result<()> {
        // This is a placeholder implementation
        Ok(())
    }
}

#[async_trait]
impl Node for GrugNode {
    type Context = GrugContext;
    type Config = GrugConfig;
    type Genesis = GrugGenesis;
    type PrivateKeyFile = GrugPrivateKeyFile;
    type SigningProvider = GrugSigningProvider;
    type NodeHandle = GrugNodeHandle;

    async fn start(&self) -> eyre::Result<Self::NodeHandle> {
        // Anca's Actor & trait implementations
        //
        // let (actor, handle) =

        Err(eyre!("Node start not implemented"))
    }

    async fn run(self) -> eyre::Result<()> {
        let handle = self.start().await?;

        handle.actor.wait(None).await.map_err(Into::into)
    }

    fn get_home_dir(&self) -> PathBuf {
        self.home_dir.clone()
    }

    fn load_config(&self) -> eyre::Result<Self::Config> {
        // Load config from file
        let config_path = self.home_dir.join("config").join("config.json");
        let config_str = std::fs::read_to_string(config_path)?;
        let config = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    fn get_address(&self, pk: &PublicKey<Self::Context>) -> <Self::Context as Context>::Address {
        // Convert public key to address by hashing and encoding the public key
        // Simplify by just converting the entire public key to base64
        let pk_bytes: &[u8] = pk.as_ref();
        let address_str = base64::encode(pk_bytes);
        GrugAddress(address_str)
    }

    fn get_public_key(&self, _pk: &PrivateKey<Self::Context>) -> PublicKey<Self::Context> {
        // Return a simple fixed public key for testing
        PublicKey::<GrugContext>::from(vec![5, 6, 7, 8])
    }

    fn get_keypair(&self, _pk: PrivateKey<Self::Context>) -> Keypair {
        // In a real implementation, we'd create a proper keypair
        // using the passed-in private key
        unimplemented!("get_keypair is not implemented")
    }

    fn load_private_key(&self, _file: Self::PrivateKeyFile) -> PrivateKey<Self::Context> {
        // Just return a dummy private key for testing purposes
        PrivateKey::<GrugContext>::from(vec![1, 2, 3, 4])
    }

    fn load_private_key_file(&self) -> eyre::Result<Self::PrivateKeyFile> {
        let path = self.private_key_file();
        let data = std::fs::read_to_string(path)?;
        let key_file: Self::PrivateKeyFile = serde_json::from_str(&data)?;
        Ok(key_file)
    }

    fn load_genesis(&self) -> eyre::Result<Self::Genesis> {
        let path = self.genesis_file();
        let data = std::fs::read_to_string(path)?;
        let genesis: Self::Genesis = serde_json::from_str(&data)?;
        Ok(genesis)
    }

    fn get_signing_provider(
        &self,
        private_key: PrivateKey<Self::Context>,
    ) -> Self::SigningProvider {
        GrugSigningProvider { private_key }
    }
}

impl CanGeneratePrivateKey for GrugNode {
    fn generate_private_key<R>(&self, _rng: R) -> PrivateKey<Self::Context>
    where
        R: RngCore + CryptoRng,
    {
        // Return a simple fixed key for testing
        PrivateKey::<GrugContext>::from(vec![1, 2, 3, 4])
    }
}

impl CanMakePrivateKeyFile for GrugNode {
    fn make_private_key_file(
        &self,
        _private_key: PrivateKey<Self::Context>,
    ) -> Self::PrivateKeyFile {
        // Simple dummy implementation
        GrugPrivateKeyFile {
            private_key: "dummy_key".to_string(),
        }
    }
}

impl CanMakeGenesis for GrugNode {
    fn make_genesis(
        &self,
        validators: Vec<(PublicKey<Self::Context>, VotingPower)>,
    ) -> Self::Genesis {
        // Convert from (PublicKey, VotingPower) tuples to GrugValidator objects
        let validators = validators
            .into_iter()
            .map(|(public_key, voting_power)| GrugValidator {
                address: self.get_address(&public_key),
                public_key,
                voting_power,
            })
            .collect();

        GrugGenesis { validators }
    }
}

impl CanMakeConfig for GrugNode {
    fn make_config(index: usize, _total: usize, _settings: MakeConfigSettings) -> Self::Config {
        let moniker = format!("node-{}", index);

        GrugConfig {
            moniker,
            consensus: ConsensusConfig::default(),
            value_sync: ValueSyncConfig::default(),
        }
    }
}
