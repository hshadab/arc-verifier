//! Zero-Knowledge Proof Circuits for Tokenized Fund Management
//!
//! This library implements circuits for proving compliance with fund investment policies
//! without revealing sensitive portfolio information.

pub mod position_limit;
pub mod liquidity_reserve;
pub mod whitelist;
pub mod range_proof;
pub mod utils;

// Nova-compatible circuits using BN254
pub mod nova_circuits;

// Composite circuit that combines all checks
pub mod composite_circuit;

pub use position_limit::PositionLimitCircuit;
pub use liquidity_reserve::LiquidityReserveCircuit;
pub use whitelist::WhitelistCircuit;
pub use nova_circuits::{NovaLiquidityCircuit, NovaPositionLimitCircuit};
pub use composite_circuit::{FundComplianceCircuit, FundComplianceParams};
