//! Unit tests for the saga_pairs post-process detection helpers.

use ecp_analyzer::post_process::saga_pairs::{strip_compensator_root, CompensatorMatch};

#[test]
fn test_strip_root_snake_camel_pascal() {
    // snake_case
    assert_eq!(
        strip_compensator_root("undo_book_room"),
        Some(CompensatorMatch {
            operation_name: "book_room".to_string()
        })
    );
    // camelCase
    assert_eq!(
        strip_compensator_root("undoBookRoom"),
        Some(CompensatorMatch {
            operation_name: "bookRoom".to_string()
        })
    );
    // PascalCase
    assert_eq!(
        strip_compensator_root("UndoBookRoom"),
        Some(CompensatorMatch {
            operation_name: "BookRoom".to_string()
        })
    );
    // rollback / compensate roots
    assert_eq!(
        strip_compensator_root("rollback_charge"),
        Some(CompensatorMatch {
            operation_name: "charge".to_string()
        })
    );
    assert_eq!(
        strip_compensator_root("compensateReserve"),
        Some(CompensatorMatch {
            operation_name: "reserve".to_string()
        })
    );
    // non-compensator
    assert_eq!(strip_compensator_root("book_room"), None);
    // root but no suffix → not a pair
    assert_eq!(strip_compensator_root("undo"), None);
}
