use crate::ir::{
    PropertyDirection, SlintComponent, SlintFile, SlintGlobal, SlintProperty, SlintType,
    SlintTypeDef,
};

use i_slint_compiler::diagnostics::BuildDiagnostics;
use i_slint_compiler::parser::{SyntaxKind, SyntaxNode, SyntaxToken, parse, syntax_nodes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

pub fn parse_slint_file(source: impl Into<String>) -> Result<SlintFile, ParseError> {
    let mut diagnostics = BuildDiagnostics::default();
    let root = parse(source.into(), None, &mut diagnostics);

    if diagnostics.has_errors() {
        return Err(ParseError {
            message: diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }

    let document = syntax_nodes::Document::from(root);
    Ok(slint_file_from_document(&document))
}

fn slint_file_from_document(document: &syntax_nodes::Document) -> SlintFile {
    let mut file = SlintFile::default();

    for component in document.Component() {
        push_component_or_global(&mut file, &component);
    }

    for exports in document.ExportsList() {
        if let Some(component) = exports.Component() {
            push_component_or_global(&mut file, &component);
        }

        file.type_defs
            .extend(exports.StructDeclaration().map(parse_struct_def));
        file.type_defs
            .extend(exports.EnumDeclaration().map(parse_enum_def));
    }

    file.type_defs
        .extend(document.StructDeclaration().map(parse_struct_def));
    file.type_defs
        .extend(document.EnumDeclaration().map(parse_enum_def));

    file
}

fn push_component_or_global(file: &mut SlintFile, component: &syntax_nodes::Component) {
    let name = declared_identifier_text(&component.DeclaredIdentifier());
    let properties = component
        .Element()
        .PropertyDeclaration()
        .filter_map(|property| parse_property(&property))
        .collect();

    if is_global_component(component) {
        file.globals.push(SlintGlobal { name, properties });
    } else {
        file.components.push(SlintComponent { name, properties });
    }
}

fn parse_property(property: &syntax_nodes::PropertyDeclaration) -> Option<SlintProperty> {
    let ty = property.Type().map(|ty| parse_type_node(&ty))?;
    let name = declared_identifier_text(&property.DeclaredIdentifier());

    Some(SlintProperty {
        rust_name: slint_name_to_rust_name(&name),
        name,
        ty,
        direction: property_direction(property),
    })
}

fn parse_struct_def(decl: syntax_nodes::StructDeclaration) -> SlintTypeDef {
    SlintTypeDef::Struct(declared_identifier_text(&decl.DeclaredIdentifier()))
}

fn parse_enum_def(decl: syntax_nodes::EnumDeclaration) -> SlintTypeDef {
    SlintTypeDef::Enum(declared_identifier_text(&decl.DeclaredIdentifier()))
}

fn parse_type_node(ty: &syntax_nodes::Type) -> SlintType {
    let raw = ty
        .text()
        .to_string()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    match raw.as_str() {
        "string" => SlintType::String,
        "bool" => SlintType::Bool,
        "int" => SlintType::Int,
        "float" => SlintType::Float,
        "color" => SlintType::Color,
        "brush" => SlintType::Brush,
        "image" => SlintType::Image,
        "duration" => SlintType::Duration,
        "length" => SlintType::Length,
        _ => SlintType::Named(raw),
    }
}

fn property_direction(property: &syntax_nodes::PropertyDeclaration) -> PropertyDirection {
    for token in direct_tokens(property).filter(|token| token.kind() == SyntaxKind::Identifier) {
        match token.text() {
            "in-out" | "in_out" => return PropertyDirection::InOut,
            "in" => return PropertyDirection::In,
            "out" => return PropertyDirection::Out,
            "property" => break,
            _ => {}
        }
    }

    PropertyDirection::InOut
}

fn is_global_component(component: &syntax_nodes::Component) -> bool {
    direct_tokens(component)
        .filter(|token| token.kind() == SyntaxKind::Identifier)
        .any(|token| token.text() == "global")
}

fn declared_identifier_text(identifier: &syntax_nodes::DeclaredIdentifier) -> String {
    identifier
        .child_token(SyntaxKind::Identifier)
        .expect("Slint parser guarantees DeclaredIdentifier has an Identifier token")
        .text()
        .to_owned()
}

fn slint_name_to_rust_name(name: &str) -> String {
    name.replace('-', "_")
}

fn direct_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> + use<> {
    node.children_with_tokens()
        .filter_map(|child| child.into_token())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_components_globals_types_and_properties() {
        let parsed = parse_slint_file(include_str!("../ui/test.slint")).unwrap();

        assert_eq!(
            parsed.type_defs,
            vec![
                SlintTypeDef::Struct("ThemeColors".into()),
                SlintTypeDef::Enum("ConnectionStatus".into()),
            ]
        );

        assert_eq!(parsed.globals.len(), 2);
        assert_eq!(parsed.globals[0].name, "AppTheme");
        assert_eq!(
            parsed.globals[0].properties,
            vec![
                property(
                    "colors",
                    SlintType::Named("ThemeColors".into()),
                    PropertyDirection::InOut
                ),
                property("dark-mode", SlintType::Bool, PropertyDirection::InOut),
                property("version-label", SlintType::String, PropertyDirection::In),
            ]
        );

        assert_eq!(parsed.components.len(), 1);
        assert_eq!(parsed.components[0].name, "TestWindow");
        assert_eq!(
            parsed.components[0].properties,
            vec![
                property("host", SlintType::String, PropertyDirection::InOut),
                property("port-text", SlintType::String, PropertyDirection::InOut),
                property("address", SlintType::String, PropertyDirection::InOut),
                property("connected", SlintType::Bool, PropertyDirection::InOut),
                property("retry-count", SlintType::Int, PropertyDirection::InOut),
                property(
                    "status",
                    SlintType::Named("ConnectionStatus".into()),
                    PropertyDirection::InOut
                ),
                property("status-label", SlintType::String, PropertyDirection::In),
                property("click-count", SlintType::Int, PropertyDirection::Out),
            ]
        );
    }

    #[test]
    fn parses_export_blocks_and_default_property_direction() {
        let parsed = parse_slint_file(
            r#"
                export struct User { name: string }
                export enum Mode { Light, Dark }

                export component Settings inherits Window {
                    property <string> title;
                    private property <int> local-counter;
                }
            "#,
        )
        .unwrap();

        assert_eq!(
            parsed.type_defs,
            vec![
                SlintTypeDef::Struct("User".into()),
                SlintTypeDef::Enum("Mode".into()),
            ]
        );
        assert_eq!(parsed.components.len(), 1);
        assert_eq!(
            parsed.components[0].properties,
            vec![
                property("title", SlintType::String, PropertyDirection::InOut),
                property("local-counter", SlintType::Int, PropertyDirection::InOut),
            ]
        );
    }

    #[test]
    fn ignores_nested_element_properties() {
        let parsed = parse_slint_file(
            r#"
                export component App inherits Window {
                    in property <string> top-level;

                    Rectangle {
                        property <string> nested;
                    }
                }
            "#,
        )
        .unwrap();

        assert_eq!(
            parsed.components[0].properties,
            vec![property(
                "top-level",
                SlintType::String,
                PropertyDirection::In
            )]
        );
    }

    #[test]
    fn returns_parse_errors() {
        let err =
            parse_slint_file("export component Broken inherits Window { property <string> ; }")
                .unwrap_err();

        assert!(!err.message.is_empty());
    }

    fn property(name: &str, ty: SlintType, direction: PropertyDirection) -> SlintProperty {
        SlintProperty {
            name: name.into(),
            rust_name: name.replace('-', "_"),
            ty,
            direction,
        }
    }
}
