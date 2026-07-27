use super::*;

fn schema_route(schema: &str, path: &str) -> SchemaRouteAssertion {
    SchemaRouteAssertion {
        method: "GET".to_owned(),
        path: path.to_owned(),
        schema: schema.to_owned(),
        required_properties: Vec::new(),
    }
}

#[test]
fn path_method_accepts_case_insensitive_method_at_exact_indent_with_crlf() {
    // Arrange
    let document = "paths:\r\n  /api/system/info:\r\n    get:\r\n      responses:\r\n";

    // Act
    let present = openapi_has_path_method(document, "/api/system/info", "GET");

    // Assert
    assert!(present);
}

#[test]
fn path_method_rejects_missing_path_and_wrong_method_indent() {
    // Arrange
    let document = "paths:\n  /api/system/info:\n      get:\n  /api/system/logs:\n    get:\n";

    // Act
    let missing_path = openapi_has_path_method(document, "/api/system/missing", "GET");
    let wrong_indent = openapi_has_path_method(document, "/api/system/info", "GET");

    // Assert
    assert!(!missing_path);
    assert!(!wrong_indent);
}

#[test]
fn path_block_stops_at_the_next_peer_path() {
    // Arrange
    let document = "paths:\n  /api/first:\n    post:\n  /api/second:\n    get:\n";

    // Act
    let leaked_method = openapi_has_path_method(document, "/api/first", "GET");

    // Assert
    assert!(!leaked_method);
}

#[test]
fn schema_property_uses_the_named_schema_when_it_exists() {
    // Arrange
    let document =
        "components:\n  schemas:\n    SystemInfo:\n      properties:\n        ASICModel:\n";
    let route = schema_route("SystemInfo", "/api/system/info");

    // Act
    let present = openapi_route_schema_has_property(document, &route, "ASICModel");
    let missing = openapi_route_schema_has_property(document, &route, "frequency");

    // Assert
    assert!(present);
    assert!(!missing);
}

#[test]
fn schema_block_does_not_borrow_a_property_from_its_peer() {
    // Arrange
    let document = "components:\n  schemas:\n    First:\n      properties:\n        own:\n    Second:\n      properties:\n        leaked:\n";
    let route = schema_route("First", "/api/first");

    // Act
    let present = openapi_route_schema_has_property(document, &route, "leaked");

    // Assert
    assert!(!present);
}

#[test]
fn existing_schema_takes_precedence_over_inline_path_properties() {
    // Arrange
    let document = "components:\n  schemas:\n    SystemInfo:\n      properties:\n        schemaOnly:\npaths:\n  /api/system/info:\n    get:\n      inlineOnly:\n";
    let route = schema_route("SystemInfo", "/api/system/info");

    // Act
    let present = openapi_route_schema_has_property(document, &route, "inlineOnly");

    // Assert
    assert!(!present);
}

#[test]
fn missing_schema_falls_back_to_an_inline_path_property() {
    // Arrange
    let document =
        "paths:\n  /api/system/info:\n    get:\n      properties:\n        responseTime:\n";
    let route = schema_route("MissingSchema", "/api/system/info");

    // Act
    let present = openapi_route_schema_has_property(document, &route, "responseTime");

    // Assert
    assert!(present);
}

#[test]
fn missing_schema_follows_quoted_references_with_supported_name_characters() {
    // Arrange
    let document = "components:\n  schemas:\n    System_Info-v2:\n      properties:\n        frequency:\npaths:\n  /api/system/info:\n    get:\n      $ref: '#/components/schemas/System_Info-v2'\n";
    let route = schema_route("MissingSchema", "/api/system/info");

    // Act
    let present = openapi_route_schema_has_property(document, &route, "frequency");

    // Assert
    assert!(present);
}

#[test]
fn missing_schema_rejects_missing_paths_empty_references_and_unknown_references() {
    // Arrange
    let missing_path_document = "paths:\n  /api/other:\n    get:\n";
    let unresolved_document = "paths:\n  /api/system/info:\n    get:\n      one: '#/components/schemas/'\n      two: '#/components/schemas/Unknown'\n";
    let route = schema_route("MissingSchema", "/api/system/info");

    // Act
    let missing_path =
        openapi_route_schema_has_property(missing_path_document, &route, "frequency");
    let unresolved = openapi_route_schema_has_property(unresolved_document, &route, "frequency");

    // Assert
    assert!(!missing_path);
    assert!(!unresolved);
}

#[test]
fn yaml_named_block_retains_blank_lines_and_the_final_unterminated_line() {
    // Arrange
    let document = "root:\n  target:\n\n    child:\n      value:";

    // Act
    let block = yaml_named_block(document, 2, "target:");

    // Assert
    assert_eq!(block, Some("\n    child:\n      value:"));
}

#[test]
fn yaml_named_block_requires_the_exact_name_and_indent() {
    // Arrange
    let document = "root:\n   target:\n    child:\n";

    // Act
    let block = yaml_named_block(document, 2, "target:");

    // Assert
    assert_eq!(block, None);
    assert_eq!(line_indentation("    child:"), 4);
}

#[test]
fn referenced_schema_extraction_ignores_lines_without_a_nonempty_schema_name() {
    // Arrange
    let block = "plain: value\nempty: '#/components/schemas/'\nvalid: \"#/components/schemas/Good_Name-v2\"\n";

    // Act
    let schemas = openapi_referenced_schemas(block);

    // Assert
    assert_eq!(schemas, vec!["Good_Name-v2"]);
}
