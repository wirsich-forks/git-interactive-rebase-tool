#!/usr/bin/env bash

set -euo pipefail

: '
This script will add to all crates a full list of enabled lints.
'

files=(
	'src/main.rs'
	'src/config/src/lib.rs'
	'src/display/src/lib.rs'
	'src/input/src/lib.rs'
	'src/view/src/lib.rs'
)

content="\
// This section is autogenerated, do not modify directly
// enable all rustc's built-in lints
#![cfg_attr(allow_unknown_lints, allow(unknown_lints))]
#![deny(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	warnings
)]
// rustc's additional allowed by default lints
#![deny(
	absolute_paths_not_starting_with_crate,
	deprecated_in_future,
	disjoint_capture_drop_reorder,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	noop_method_call,
	or_patterns_back_compat,
	pointer_structural_match,
	semicolon_in_expressions_from_macros,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unsafe_code,
	unsafe_op_in_unsafe_fn,
	unstable_features,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences
)]
// enable all of Clippy's lints
#![deny(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic, clippy::restriction)]
#![allow(
	clippy::blanket_clippy_restriction_lints,
	clippy::implicit_return,
	clippy::missing_docs_in_private_items,
	clippy::redundant_pub_crate,
	clippy::tabs_in_doc_comments,
)]
#![deny(
	rustdoc::bare_urls,
	rustdoc::broken_intra_doc_links,
	rustdoc::invalid_codeblock_attributes,
	rustdoc::invalid_html_tags,
	rustdoc::missing_crate_level_docs,
	rustdoc::private_doc_tests,
	rustdoc::private_intra_doc_links
)]\
"
content="${content//$'\n'/\\n}"

for f in "${files[@]}"; do
	awk -i inplace '
		BEGIN       {p=1}
		/^\/\/ LINT-REPLACE-START/   {
			print;
			print "'"${content}"'";
			p=0
		}
		/^\/\/ LINT-REPLACE-END/     {p=1}
		p
	' "$f"
done



