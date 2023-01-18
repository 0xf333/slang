extern crate napi_build;

use anyhow::Result;
use codegen_parser::GrammarParserGeneratorExtensions;
use codegen_schema::types::grammar::Grammar;
use codegen_utils::context::CodegenContext;
use solidity_schema::SolidityGrammarExtensions;

fn main() -> Result<()> {
    return CodegenContext::with_context(|codegen| {
        let grammar = Grammar::load_solidity()?;

        let output_dir = codegen
            .repo_root
            .join("crates/solidity/outputs/typescript/lib/src/generated");

        grammar.generate_typescript_lib_sources(codegen, &output_dir);

        napi_build::setup();

        return Ok(());
    });
}