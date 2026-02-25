// SPDX-License-Identifier: Apache-2.0

use crate::docs_commands::{
    crate_doc_contract_status, docs_inventory_payload, docs_markdown_files, docs_registry_payload,
    has_required_section, load_quality_policy, registry_validate_payload, search_synonyms,
    workspace_crate_roots,
};
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

include!("docs_command_runtime/payload_builders.rs");
include!("docs_command_runtime/subprocess_support.rs");
include!("docs_command_runtime/command_dispatch.rs");
