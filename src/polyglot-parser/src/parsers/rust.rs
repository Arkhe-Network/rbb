use anyhow::Result;
use async_trait::async_trait;
use tree_sitter::{Parser as TSParser, Node as TSNode};
use tree_sitter_rust as ts_rust;
use std::sync::OnceLock;
use super::Parser;
use crate::ast::{Language, Node, NodeKind, Span, Position, MetadataValue, LiteralKind};

pub struct RustParser;

impl RustParser {
    pub fn new() -> Self { Self }

    fn get_parser() -> &'static mut TSParser {
        // This is a hacky way to use tree-sitter since Parser is not Sync.
        // In a real application, we would use a thread-local parser or a pool.
        unimplemented!()
    }

    fn convert_node(node: &TSNode, code: &str, language: &Language) -> Node {
        let kind = node.kind();
        let start = node.start_position();
        let end = node.end_position();
        let span = Span {
            start: Position { line: start.row + 1, column: start.column, offset: 0 },
            end: Position { line: end.row + 1, column: end.column, offset: 0 },
        };
        let node_kind = match kind {
            "source_file" => NodeKind::Module,
            "function_item" => NodeKind::FunctionDefinition,
            "let_declaration" => NodeKind::VariableDeclaration,
            "const_declaration" => NodeKind::ConstantDeclaration,
            "struct_item" => NodeKind::StructDeclaration,
            "enum_item" => NodeKind::EnumDeclaration,
            "impl_item" => NodeKind::ClassDeclaration,
            "trait_item" => NodeKind::InterfaceDeclaration,
            "if_expression" | "if_statement" => NodeKind::IfStatement,
            "for_expression" | "for_statement" => NodeKind::ForStatement,
            "while_expression" | "while_statement" => NodeKind::WhileStatement,
            "loop_expression" => NodeKind::DoWhileStatement,
            "return_expression" => NodeKind::ReturnStatement,
            "break_expression" => NodeKind::BreakStatement,
            "continue_expression" => NodeKind::ContinueStatement,
            "block" => NodeKind::Block,
            "identifier" => NodeKind::Identifier,
            "string_literal" => NodeKind::Literal(LiteralKind::String("".to_string())),
            "integer_literal" => NodeKind::Literal(LiteralKind::Integer(0)),
            "boolean_literal" => NodeKind::Literal(LiteralKind::Boolean(false)),
            "binary_expression" => NodeKind::BinaryExpression,
            "unary_expression" => NodeKind::UnaryExpression,
            "assignment_expression" => NodeKind::AssignmentExpression,
            "call_expression" => NodeKind::CallExpression,
            "method_call_expression" => NodeKind::MethodCall,
            "field_expression" => NodeKind::MemberAccess,
            "macro_invocation" => NodeKind::CallExpression,
            "comment" | "line_comment" | "block_comment" => NodeKind::Comment,
            "attribute_item" => NodeKind::SecurityAnnotation,
            _ => NodeKind::ExpressionStatement,
        };

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            children.push(Self::convert_node(&child, code, language));
        }

        let mut metadata = std::collections::HashMap::new();
        if node_kind == NodeKind::Identifier {
            if let Ok(text) = node.utf8_text(code.as_bytes()) {
                metadata.insert("name".to_string(), MetadataValue::String(text.to_string()));
            }
        }
        // Extrair nome de função se presente
        if node_kind == NodeKind::FunctionDefinition {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    if let Ok(text) = child.utf8_text(code.as_bytes()) {
                        metadata.insert("function_name".to_string(), MetadataValue::String(text.to_string()));
                    }
                }
            }
        }

        Node { kind: node_kind, span, children, metadata, language: language.clone() }
    }
}

#[async_trait]
impl Parser for RustParser {
    fn language(&self) -> Language { Language::Rust }

    async fn parse(&self, code: &str) -> Result<Node> {
        let mut parser = TSParser::new();
        parser.set_language(ts_rust::language()).unwrap();
        let tree = parser.parse(code, None).ok_or_else(|| anyhow::anyhow!("Parse failed"))?;
        let root = tree.root_node();
        Ok(Self::convert_node(&root, code, &Language::Rust))
    }
}
