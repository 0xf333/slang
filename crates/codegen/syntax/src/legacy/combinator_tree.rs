use std::{cell::Cell, fmt::Write};

use codegen_ebnf::EbnfSerializer;
use codegen_schema::types::{ProductionDefinition, ProductionRef};

use super::{
    char_first_set::CharFirstSet, code_generator::CodeGenerator,
    combinator_context::CombinatorContext, combinator_node::CombinatorNode,
};

pub struct CombinatorTree<'context> {
    pub context: &'context CombinatorContext<'context>,
    pub production: ProductionRef,
    pub root_node: Cell<Option<&'context CombinatorNode<'context>>>,
}

impl<'context> CombinatorTree<'context> {
    pub fn new(
        context: &'context CombinatorContext<'context>,
        production: &ProductionRef,
    ) -> &'context CombinatorTree<'context> {
        context.alloc_tree(CombinatorTree {
            context: context,
            production: production.clone(),
            root_node: Cell::new(None),
        })
    }

    pub fn char_first_set(&self) -> CharFirstSet {
        self.root_node.get().unwrap().char_first_set()
    }

    pub fn ensure_tree_is_built(&'context self) {
        if self.root_node.get().is_none() {
            let version = &self.context.version;
            self.root_node.set(match &self.production.definition {
                ProductionDefinition::Scanner { version_map, .. } => version_map
                    .get_for_version(version)
                    .map(|scanner| CombinatorNode::from_scanner(self, &scanner)),
                ProductionDefinition::TriviaParser { version_map, .. }
                | ProductionDefinition::Parser { version_map, .. } => version_map
                    .get_for_version(version)
                    .map(|parser| CombinatorNode::from_parser(self, &parser)),
                ProductionDefinition::PrecedenceParser { version_map, .. } => version_map
                    .get_for_version(version)
                    .map(|parser| CombinatorNode::from_precedence_parser(self, &parser)),
            });
        }
    }

    pub fn add_to_generated_code(&self, code: &mut CodeGenerator) {
        let first_version = self.context.language.versions.first().unwrap();
        let version = &self.context.version;
        let matches_version = match self.production.versions() {
            Some(versions) => versions.contains(&version),
            None => version == first_version,
        };
        if !matches_version {
            return;
        }

        let name = &self.production.name;
        let comment = self.generate_comment();

        match self.production.definition {
            ProductionDefinition::Scanner { .. } => {
                let definition = self.root_node.get().map(|node| {
                    if node.char_first_set().includes_epsilon {
                        unreachable!(
                            "Validation should have discovered that scanner {name} can be empty"
                        );
                    }

                    (comment, node.to_scanner_code(code))
                });
                code.add_scanner(name.clone(), version, definition);
            }
            ProductionDefinition::TriviaParser { .. } => {
                let definition = self
                    .root_node
                    .get()
                    .map(|node| (comment, node.to_parser_code(code, true)));
                code.add_parser(name.clone(), version, definition);
            }
            ProductionDefinition::Parser { .. } | ProductionDefinition::PrecedenceParser { .. } => {
                let definition = self
                    .root_node
                    .get()
                    .map(|node| (comment, node.to_parser_code(code, false)));
                code.add_parser(name.clone(), version, definition);
            }
        }
    }

    fn generate_comment(&self) -> String {
        let mut comment = String::new();

        if self.production.versions().is_some() {
            writeln!(comment, "(* v{version} *)", version = self.context.version).unwrap();
        }

        if let Some(outputs) = EbnfSerializer::serialize_version(
            &self.context.language,
            &self.production,
            &self.context.version,
        ) {
            for ebnf in outputs.values() {
                writeln!(comment, "{ebnf}").unwrap();
            }
        }

        return comment.lines().map(|line| format!("// {line}\n")).collect();
    }
}
