# Product Context

## 🎯 Why This Project Exists

Z00Z explores a blockchain model where ownership and much of the transaction
construction flow live off-chain, while the chain acts as a finalization and
verification surface. The project is oriented around privacy, scalability, and
programmability without falling back to a traditional address-and-balance model.

## 👥 Who It Serves

- Privacy-conscious end users who want cash-like digital transaction behavior
- Builders working on wallets, runtime flows, asset logic, and integrations
- Operators and reviewers who need deterministic tooling and explicit
  verification paths
- Enterprise or regulatory use cases that may require selective traceability

## 💡 User Value

- Strong privacy defaults
- High-throughput design goals
- Flexible architecture for assets, wallets, and runtime extensions
- Support for both anonymous-style and traceable-style operational modes

## 🔄 Product Modes

### 🔒 Privacy-First Mode

Focuses on unlinkable flows, minimized on-chain disclosure, and off-chain
ownership semantics.

### 🧾 Traceable Mode

Allows transparent or auditable behavior where explicitly required by the use
case, while still preserving the stateless architectural direction.

## ⭐ Product Expectations

- Security-sensitive code must be explicit, reviewable, and bounded
- Wallet and protocol behavior should remain understandable despite high
  cryptographic complexity
- Documentation should help contributors navigate a large multi-crate workspace
- The system should favor deterministic, testable abstractions over hidden side
  effects
