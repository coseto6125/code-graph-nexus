;; Functions
(function_definition
  type: (_) @type
  declarator: [
    (function_declarator
      declarator: (identifier) @function.name)
    (pointer_declarator
      declarator: (function_declarator
        declarator: (identifier) @function.name))
  ]) @function

;; Structs & Enums
(struct_specifier
  name: (type_identifier) @struct.name) @struct

(enum_specifier
  name: (type_identifier) @struct.name) @struct

;; Includes
(preproc_include
  path: [
    (string_literal) @import.source
    (system_lib_string) @import.source
  ]) @import
