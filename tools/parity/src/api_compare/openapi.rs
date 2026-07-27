use super::*;

pub(super) fn openapi_has_path_method(openapi_yaml: &str, path: &str, method: &str) -> bool {
    let Some(path_block) = openapi_path_block(openapi_yaml, path) else {
        return false;
    };
    let method_marker = format!("{}:", method.to_ascii_lowercase());

    path_block
        .lines()
        .any(|line| line_indentation(line) == 4 && line.trim_end().trim_start() == method_marker)
}

pub(super) fn openapi_route_schema_has_property(
    openapi_yaml: &str,
    schema_route: &SchemaRouteAssertion,
    property: &str,
) -> bool {
    if let Some(schema_block) = openapi_schema_block(openapi_yaml, &schema_route.schema) {
        return openapi_block_has_property(schema_block, property);
    }

    let Some(path_block) = openapi_path_block(openapi_yaml, &schema_route.path) else {
        return false;
    };

    if openapi_block_has_property(path_block, property) {
        return true;
    }

    openapi_referenced_schemas(path_block).iter().any(|schema| {
        openapi_schema_block(openapi_yaml, schema)
            .is_some_and(|schema_block| openapi_block_has_property(schema_block, property))
    })
}

fn openapi_path_block<'a>(openapi_yaml: &'a str, path: &str) -> Option<&'a str> {
    yaml_named_block(openapi_yaml, 2, &format!("{path}:"))
}

fn openapi_schema_block<'a>(openapi_yaml: &'a str, schema: &str) -> Option<&'a str> {
    yaml_named_block(openapi_yaml, 4, &format!("{schema}:"))
}

fn yaml_named_block<'a>(document: &'a str, indent: usize, name: &str) -> Option<&'a str> {
    let marker = format!("{}{name}", " ".repeat(indent));
    let mut offset = 0;
    let mut maybe_content_start = None;

    for line in document.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches(['\r', '\n']);
        if line_without_newline == marker {
            maybe_content_start = Some(offset + line.len());
            break;
        }

        offset += line.len();
    }

    let content_start = maybe_content_start?;
    let mut block_end = document.len();
    offset = content_start;

    for line in document[content_start..].split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches(['\r', '\n']);
        if !line_without_newline.trim().is_empty()
            && line_indentation(line_without_newline) <= indent
        {
            block_end = offset;
            break;
        }

        offset += line.len();
    }

    Some(&document[content_start..block_end])
}

fn line_indentation(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
}

fn openapi_block_has_property(openapi_block: &str, property: &str) -> bool {
    let property_marker = format!("{property}:");
    openapi_block
        .lines()
        .any(|line| line.trim_start().starts_with(&property_marker))
}

fn openapi_referenced_schemas(openapi_block: &str) -> Vec<String> {
    openapi_block
        .lines()
        .filter_map(|line| {
            let (_, schema_name) = line.split_once("#/components/schemas/")?;
            let schema_name = schema_name
                .trim_matches(|character| character == '\'' || character == '"')
                .chars()
                .take_while(|character| {
                    character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
                })
                .collect::<String>();
            (!schema_name.is_empty()).then_some(schema_name)
        })
        .collect()
}
