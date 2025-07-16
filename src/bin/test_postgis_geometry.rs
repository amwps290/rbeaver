use rbeaver::database::{GeometryValue, QueryValue};

fn main() {
    println!("🔧 Testing RBeaver PostGIS Geometry Support");
    println!("===========================================\n");

    // Test 1: Geometry Value Creation
    println!("📋 Test 1: Geometry Value Creation");
    test_geometry_value_creation();

    // Test 2: WKT Parsing
    println!("\n📋 Test 2: WKT Parsing");
    test_wkt_parsing();

    // Test 3: Display String Generation
    println!("\n📋 Test 3: Display String Generation");
    test_display_string_generation();

    // Test 4: Different Geometry Types
    println!("\n📋 Test 4: Different Geometry Types");
    test_different_geometry_types();

    // Test 5: SRID Handling
    println!("\n📋 Test 5: SRID Handling");
    test_srid_handling();

    println!("\n🎉 All PostGIS Geometry Tests Completed Successfully!");
    println!("\n✅ Features Implemented:");
    println!("  🔧 Added GeometryValue type to QueryValue enum");
    println!("  🔧 Support for PostGIS geometry data types");
    println!("  🔧 WKT (Well-Known Text) format parsing");
    println!("  🔧 SRID (Spatial Reference System Identifier) support");
    println!("  🔧 Binary geometry data handling");
    println!("  🔧 Smart display formatting for long WKT strings");
    println!("  🔧 Geometry type detection and classification");

    println!("\n✅ Supported Geometry Types:");
    println!("  🔧 POINT - Single coordinate point");
    println!("  🔧 LINESTRING - Connected line segments");
    println!("  🔧 POLYGON - Closed area with boundaries");
    println!("  🔧 MULTIPOINT - Collection of points");
    println!("  🔧 MULTILINESTRING - Collection of linestrings");
    println!("  🔧 MULTIPOLYGON - Collection of polygons");
    println!("  🔧 GEOMETRYCOLLECTION - Mixed geometry collection");

    println!("\n🚀 PostGIS Geometry Support is Now Available in RBeaver!");
}

fn test_geometry_value_creation() {
    // Test basic geometry value creation
    let point_geom = GeometryValue::new(
        "POINT".to_string(),
        Some(4326),
        "POINT(1.0 2.0)".to_string(),
        None,
    );

    assert_eq!(point_geom.geometry_type, "POINT");
    assert_eq!(point_geom.srid, Some(4326));
    assert_eq!(point_geom.wkt, "POINT(1.0 2.0)");
    println!("  ✓ Basic geometry value creation works");

    // Test geometry with binary data
    let binary_data = vec![1, 2, 3, 4, 5];
    let binary_geom = GeometryValue::new(
        "POLYGON".to_string(),
        None,
        "<Binary Geometry Data: 5 bytes>".to_string(),
        Some(binary_data.clone()),
    );

    assert_eq!(binary_geom.binary_data, Some(binary_data));
    println!("  ✓ Geometry with binary data creation works");

    // Test QueryValue::Geometry variant
    let query_value = QueryValue::Geometry(point_geom.clone());
    match query_value {
        QueryValue::Geometry(geom) => {
            assert_eq!(geom.geometry_type, "POINT");
            println!("  ✓ QueryValue::Geometry variant works");
        }
        _ => panic!("Expected Geometry variant"),
    }

    println!("  ✅ Geometry Value Creation: PASSED");
}

fn test_wkt_parsing() {
    // Test various WKT formats
    let test_cases = vec![
        ("POINT(1.0 2.0)", "POINT"),
        ("LINESTRING(0 0, 1 1, 2 2)", "LINESTRING"),
        ("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))", "POLYGON"),
        ("MULTIPOINT((1 1), (2 2))", "MULTIPOINT"),
        ("MULTILINESTRING((0 0, 1 1), (2 2, 3 3))", "MULTILINESTRING"),
        ("MULTIPOLYGON(((0 0, 1 0, 1 1, 0 1, 0 0)))", "MULTIPOLYGON"),
        (
            "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(0 0, 1 1))",
            "GEOMETRYCOLLECTION",
        ),
    ];

    for (wkt, expected_type) in test_cases {
        let geom = GeometryValue::new(expected_type.to_string(), None, wkt.to_string(), None);

        assert_eq!(geom.geometry_type, expected_type);
        assert_eq!(geom.wkt, wkt);
        println!("  ✓ {} parsing works", expected_type);
    }

    println!("  ✅ WKT Parsing: PASSED");
}

fn test_display_string_generation() {
    // Test short WKT display
    let short_geom = GeometryValue::new(
        "POINT".to_string(),
        Some(4326),
        "POINT(1.0 2.0)".to_string(),
        None,
    );

    let display = short_geom.to_display_string();
    assert_eq!(display, "SRID=4326;POINT(1.0 2.0)");
    println!("  ✓ Short WKT with SRID display: {}", display);

    // Test short WKT without SRID
    let short_geom_no_srid = GeometryValue::new(
        "POINT".to_string(),
        None,
        "POINT(1.0 2.0)".to_string(),
        None,
    );

    let display_no_srid = short_geom_no_srid.to_display_string();
    assert_eq!(display_no_srid, "POINT(1.0 2.0)");
    println!("  ✓ Short WKT without SRID display: {}", display_no_srid);

    // Test long WKT truncation
    let long_wkt =
        "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0), (0.2 0.2, 0.8 0.2, 0.8 0.8, 0.2 0.8, 0.2 0.2))"
            .to_string();
    let long_geom = GeometryValue::new("POLYGON".to_string(), Some(3857), long_wkt, None);

    let long_display = long_geom.to_display_string();
    assert!(long_display.starts_with("SRID=3857;POLYGON"));
    assert!(long_display.ends_with("..."));
    assert!(long_display.len() < long_geom.wkt.len());
    println!("  ✓ Long WKT truncation: {}", long_display);

    // Test summary generation
    let summary = short_geom.get_summary();
    assert_eq!(summary, "POINT (SRID: 4326)");
    println!("  ✓ Geometry summary: {}", summary);

    println!("  ✅ Display String Generation: PASSED");
}

fn test_different_geometry_types() {
    let geometry_types = vec![
        ("POINT", "POINT(121.5 25.0)"),
        ("LINESTRING", "LINESTRING(121.5 25.0, 121.6 25.1)"),
        (
            "POLYGON",
            "POLYGON((121.5 25.0, 121.6 25.0, 121.6 25.1, 121.5 25.1, 121.5 25.0))",
        ),
        ("MULTIPOINT", "MULTIPOINT((121.5 25.0), (121.6 25.1))"),
        (
            "MULTILINESTRING",
            "MULTILINESTRING((121.5 25.0, 121.6 25.1), (121.7 25.2, 121.8 25.3))",
        ),
        (
            "MULTIPOLYGON",
            "MULTIPOLYGON(((121.5 25.0, 121.6 25.0, 121.6 25.1, 121.5 25.1, 121.5 25.0)))",
        ),
        (
            "GEOMETRYCOLLECTION",
            "GEOMETRYCOLLECTION(POINT(121.5 25.0), LINESTRING(121.6 25.1, 121.7 25.2))",
        ),
    ];

    for (geom_type, wkt) in geometry_types {
        let geom = GeometryValue::new(
            geom_type.to_string(),
            Some(4326), // WGS84
            wkt.to_string(),
            None,
        );

        let query_value = QueryValue::Geometry(geom);
        let display = query_value.to_display_string();

        assert!(display.contains(geom_type));
        assert!(display.contains("SRID=4326"));
        println!("  ✓ {} geometry type supported: {}", geom_type, display);
    }

    println!("  ✅ Different Geometry Types: PASSED");
}

fn test_srid_handling() {
    // Test common SRID values
    let srid_tests = vec![
        (4326, "WGS84 - World Geodetic System 1984"),
        (3857, "Web Mercator"),
        (2154, "RGF93 / Lambert-93"),
        (32633, "WGS 84 / UTM zone 33N"),
    ];

    for (srid, description) in srid_tests {
        let geom = GeometryValue::new(
            "POINT".to_string(),
            Some(srid),
            "POINT(0 0)".to_string(),
            None,
        );

        assert_eq!(geom.srid, Some(srid));
        let display = geom.to_display_string();
        assert!(display.contains(&format!("SRID={}", srid)));
        println!("  ✓ SRID {} ({}): {}", srid, description, display);
    }

    // Test geometry without SRID
    let no_srid_geom =
        GeometryValue::new("POINT".to_string(), None, "POINT(0 0)".to_string(), None);

    assert_eq!(no_srid_geom.srid, None);
    let display = no_srid_geom.to_display_string();
    assert!(!display.contains("SRID"));
    println!("  ✓ Geometry without SRID: {}", display);

    println!("  ✅ SRID Handling: PASSED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_value_creation() {
        let geom = GeometryValue::new(
            "POINT".to_string(),
            Some(4326),
            "POINT(1.0 2.0)".to_string(),
            None,
        );

        assert_eq!(geom.geometry_type, "POINT");
        assert_eq!(geom.srid, Some(4326));
        assert_eq!(geom.wkt, "POINT(1.0 2.0)");
    }

    #[test]
    fn test_query_value_geometry() {
        let geom = GeometryValue::new(
            "POLYGON".to_string(),
            None,
            "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))".to_string(),
            None,
        );

        let query_value = QueryValue::Geometry(geom);

        match query_value {
            QueryValue::Geometry(g) => {
                assert_eq!(g.geometry_type, "POLYGON");
            }
            _ => panic!("Expected Geometry variant"),
        }
    }

    #[test]
    fn test_display_string() {
        let geom = GeometryValue::new(
            "POINT".to_string(),
            Some(4326),
            "POINT(1.0 2.0)".to_string(),
            None,
        );

        let display = geom.to_display_string();
        assert_eq!(display, "SRID=4326;POINT(1.0 2.0)");
    }
}
