//! sharecli - Shared CLI process manager
//!
//! Thin CLI wrapper around local process runtime.
//!
//! Features:
//! - Process management via local runtime types
//! - Multi-project orchestration

pub mod cast;
pub mod commands;
pub mod config;
pub mod config_watcher;
pub mod coordination;
pub mod health_check;
pub mod monitoring;
pub mod notifier;
pub mod runtime;
pub mod serve_lock;
pub mod spawn_policy;
pub mod watchdog;

pub use anyhow::Result;
pub use runtime::{
    ManagedProcess, ProcessFilter, ProcessInfo, ProcessPool, ProjectLimits, ProjectResources,
    SharedRuntime,
};
pub mod config_loader;
pub mod env_manager;
pub mod health;
pub mod log_sink;
pub mod metrics;
pub mod proc_table;
pub mod scheduler;
pub mod signals;

pub mod api;
pub mod argparse;
pub mod astar;
pub mod backoff;
pub mod base64_util;
pub mod binary_search;
pub mod bloom;
pub mod cache;
pub mod config_merger;
pub mod credit_card;
pub mod cron_parser;
pub mod csv_util;
pub mod deque;
pub mod disjoint_set;
pub mod erf;
pub mod feature_flags;
pub mod graph;
pub mod hash_util;
pub mod ipv4_util;
pub mod itoa;
pub mod jsonpath_lite;
pub mod lazy;
pub mod levenshtein;
pub mod lru;
pub mod matrix;
pub mod money;
pub mod object_pool;
pub mod perm;
pub mod pin;
pub mod priority_queue;
pub mod queue;
pub mod queue2;
pub mod rate_limiter;
pub mod rational;
pub mod retry;
pub mod ring_buffer;
pub mod slice_ext;
pub mod sliding_window;
pub mod sorted_vec;
pub mod sortedset;
pub mod stack;
pub mod stats;
pub mod stopwatch;
pub mod stream;
pub mod tar_util;
pub mod template;
pub mod text_slab;
pub mod trim;
pub mod typed_id;
pub mod utf8v;
pub mod uuid;
pub mod vlq;
pub mod xml_escape;

pub mod color;
pub mod distance;

pub mod bucks;
pub mod md_table;

pub mod jsonschema_subset;
pub mod radix_trie;

pub mod binary_search_ex;
pub mod kmp_search;

pub mod bloom_filter;
pub mod lru_cache_ext;

pub mod skiplist;
pub mod trie_compressed;

pub mod flatbuffers_lite;
pub mod lz4_block;

pub mod crc64;
pub mod glob_pattern;

pub mod base85;
pub mod xxhash3;

pub mod apfs_uuid;
pub mod xxtea;

pub mod base_n_radix;
pub mod word_count;

pub mod csv_writer;
pub mod mime_qp;

pub mod html_escape;
pub mod ipaddr_validation;

pub mod json_pointer;
pub mod markdown_inline;

pub mod dhcp_options;
pub mod dhcpv6_msg;
pub mod dns_zone;
pub mod ini_parser;
pub mod json5;
pub mod macho_parse;
pub mod msgpack;
pub mod ntp_timestamp;
pub mod s_expression;
pub mod ssh_known_hosts;
pub mod tar_header;
pub mod toml_lite;
pub mod url_safe_base64;
pub mod wasm_opcode;
pub mod zip_crc32;

pub mod bitcoin_bech32;
pub mod ldap_filter;
pub mod oauth1_signature;
pub mod pem_decode;
pub mod ssh_packet;
pub mod uuid_v7;
pub mod x509_chain;

pub mod flac_metadata_block;
pub mod m3u8_playlist;

pub mod cue_sheet;
pub mod imap_response;
pub mod mp4_box;
pub mod smtp_envelope;

// L138: sharecli library expansion — Keccak / SHA-3 hash + segment tree + Bellman-Ford + matrix ops
pub mod bellman_ford;
pub mod keccak;
pub mod matrix_ops;
pub mod segment_tree_basic;

// L130: sharecli library expansion — ANSI terminal codes + CRC checksums
pub mod ansi;
pub mod cyclic_check;
pub mod webmanifest;
pub mod webvtt_cue;

// L139: sharecli library expansion — file listing + URL-safe base64 + Roman numerals
pub mod asn1_ber;
pub mod asn1_ber_parity;
pub mod base64url;
pub mod bip39_mnemonic;
pub mod bip39_wordlist;
pub mod bmp_image;
pub mod cdp_meraki_discovery;
pub mod coap_option_parse;
pub mod dns_query_parser;
pub mod dnssec_chain;
pub mod ipsec_esp_parse;
pub mod mapi_props;
pub mod mapi_props_parity;
pub mod natural_sort;
pub mod pres_header_parity;
pub mod pres_header_parse;
pub mod qoi_image;
pub mod rdp_neg;
pub mod roman_numeral;
pub mod snmpv3_msg;
pub mod x12_edi_segment;

// L137: sharecli parity expansion — FIPS 202 SHA-3 + RFC 5869 HKDF + RFC 7693 BLAKE2 + RFC 8439 ChaCha20
pub mod blake2;
pub mod chacha20;
pub mod hkdf;
pub mod sha3_keccak;

// L139: sharecli library expansion — Kahn/DFS topological sort + integer square root + Catalan numbers + 3D vector math
pub mod catalan_number;
pub mod sqrt_integer;
pub mod topological_sort;
pub mod vector_3d;
