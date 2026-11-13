//! Outbound integrations with other W platform services.
//!
//! Each submodule is the single, isolated place ERP talks to one external
//! service, so the rest of the codebase stays free of cross-service detail.

pub mod daas;
