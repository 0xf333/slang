use proc_macro2::TokenStream;
use quote::quote;

use super::boilerplate;

pub fn mod_head() -> TokenStream {
    quote!(
        pub mod kinds;
        pub mod lex;
        pub mod cst;
        pub mod parse;
        pub mod language;
    )
}

pub fn kinds_head() -> TokenStream {
    quote!(
        use serde::Serialize;
        use strum_macros::*;
    )
}

pub fn cst_visitor_head() -> TokenStream {
    quote!(
        #[allow(unused_variables)]
        pub trait Visitor<E> {
            fn enter_rule(
                &mut self,
                kind: RuleKind,
                children: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorEntryResponse, E> {
                Ok(VisitorEntryResponse::StepIn)
            }

            fn exit_rule(
                &mut self,
                kind: RuleKind,
                children: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorExitResponse, E> {
                Ok(VisitorExitResponse::StepIn)
            }

            fn enter_token(
                &mut self,
                kind: TokenKind,
                lex_node: &Rc<lex::Node>,
                trivia: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorEntryResponse, E> {
                Ok(VisitorEntryResponse::StepIn)
            }

            fn exit_token(
                &mut self,
                kind: TokenKind,
                lex_node: &Rc<lex::Node>,
                trivia: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorExitResponse, E> {
                Ok(VisitorExitResponse::StepIn)
            }

            fn enter_group(
                &mut self,
                children: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorEntryResponse, E> {
                Ok(VisitorEntryResponse::StepIn)
            }

            fn exit_group(
                &mut self,
                children: &Vec<Rc<Node>>,
                node: &Rc<Node>,
                path: &Vec<Rc<Node>>,
            ) -> Result<VisitorExitResponse, E> {
                Ok(VisitorExitResponse::StepIn)
            }
        }

        pub enum VisitorEntryResponse {
            Quit,
            StepIn,
            StepOver,
        }

        pub enum VisitorExitResponse {
            Quit,
            StepIn,
        }

        pub trait Visitable<T: Visitor<E>, E> {
            fn visit(&self, visitor: &mut T) -> Result<VisitorExitResponse, E>;
            fn visit_with_path(
                &self,
                visitor: &mut T,
                path: &mut Vec<Rc<Node>>,
            ) -> Result<VisitorExitResponse, E>;
        }

        impl<T: Visitor<E>, E> Visitable<T, E> for Rc<Node> {
            fn visit(&self, visitor: &mut T) -> Result<VisitorExitResponse, E> {
                self.visit_with_path(visitor, &mut Vec::new())
            }

            fn visit_with_path(
                &self,
                visitor: &mut T,
                path: &mut Vec<Rc<Node>>,
            ) -> Result<VisitorExitResponse, E> {
                match self.as_ref() {
                    Node::Rule { kind, children } => {
                        match visitor.enter_rule(*kind, children, self, path)? {
                            VisitorEntryResponse::Quit => return Ok(VisitorExitResponse::Quit),
                            VisitorEntryResponse::StepOver => {}
                            VisitorEntryResponse::StepIn => {
                                path.push(self.clone());
                                for child in children {
                                    match child.visit_with_path(visitor, path)? {
                                        VisitorExitResponse::Quit => {
                                            path.pop();
                                            return Ok(VisitorExitResponse::Quit);
                                        }
                                        VisitorExitResponse::StepIn => {}
                                    }
                                }
                                path.pop();
                            }
                        }
                        visitor.exit_rule(*kind, children, self, path)
                    }
                    Node::Token {
                        kind,
                        lex_node,
                        trivia,
                    } => {
                        match visitor.enter_token(*kind, lex_node, trivia, self, path)? {
                            VisitorEntryResponse::Quit => return Ok(VisitorExitResponse::Quit),
                            VisitorEntryResponse::StepOver => {}
                            VisitorEntryResponse::StepIn => {
                                path.push(self.clone());
                                for child in trivia {
                                    match child.visit_with_path(visitor, path)? {
                                        VisitorExitResponse::Quit => {
                                            path.pop();
                                            return Ok(VisitorExitResponse::Quit);
                                        }
                                        VisitorExitResponse::StepIn => {}
                                    }
                                }
                                path.pop();
                            }
                        }
                        visitor.exit_token(*kind, lex_node, trivia, self, path)
                    }
                    Node::Group { children } => {
                        match visitor.enter_group(children, self, path)? {
                            VisitorEntryResponse::Quit => return Ok(VisitorExitResponse::Quit),
                            VisitorEntryResponse::StepOver => {}
                            VisitorEntryResponse::StepIn => {
                                path.push(self.clone());
                                for child in children {
                                    match child.visit_with_path(visitor, path)? {
                                        VisitorExitResponse::Quit => {
                                            path.pop();
                                            return Ok(VisitorExitResponse::Quit);
                                        }
                                        VisitorExitResponse::StepIn => {}
                                    }
                                }
                                path.pop();
                            }
                        }
                        visitor.exit_group(children, self, path)
                    }
                }
            }
        }
    )
}

pub fn language_head() -> TokenStream {
    let error_renderer = boilerplate::error_renderer();

    quote!(
        use std::rc::Rc;
        use std::collections::BTreeMap;

        use chumsky::{error::SimpleReason, BoxedParser, Span, Parser as ChumskyParser};
        use ariadne::{Color, Config, Fmt, Label, Report, ReportKind, Source};
        use semver::Version;

        use super::{
            cst,
            parse::ErrorType,
            parse::create_parsers,
            kinds::ProductionKind,
        };

        pub struct Language {
            parsers: BTreeMap<ProductionKind, Parser>,
            version: Version,
        }

        impl Language {
            pub fn new(version: Version) -> Self {
                Self {
                    parsers: create_parsers(&version),
                    version,
                }
            }

            pub fn version(&self) -> &Version {
                &self.version
            }

            pub fn get_parser(&self, kind: ProductionKind) -> &Parser {
                &self.parsers[&kind]
            }
        }

        pub struct Parser(BoxedParser<'static, char, Rc<cst::Node>, ErrorType>);

        impl Parser {
            pub(super) fn new(inner: BoxedParser<'static, char, Rc<cst::Node>, ErrorType>) -> Self {
                Self(inner)
            }

            pub fn parse(&self, input: &str) -> ParserOutput {
                let (parse_tree, errors) = self.0.parse_recovery(input);
                ParserOutput { parse_tree, errors }
            }
        }

        #[derive(PartialEq)]
        pub struct ParserOutput {
            parse_tree: Option<Rc<cst::Node>>,
            errors: Vec<ErrorType>,
        }

        impl ParserOutput {
            pub fn parse_tree(&self) -> Option<Rc<cst::Node>> {
                self.parse_tree.clone()
            }

            pub fn error_count(&self) -> usize {
                self.errors.len()
            }

            pub fn errors_as_strings(&self, source_id: &str, source: &str, with_colour: bool) -> Vec<String> {
                return self
                    .errors
                    .iter()
                    .map(|error| render_error_report(error, source_id, source, with_colour))
                    .collect();
            }

            pub fn is_valid(&self) -> bool {
                self.parse_tree.is_some() && self.errors.is_empty()
            }
        }

        #error_renderer
    )
}