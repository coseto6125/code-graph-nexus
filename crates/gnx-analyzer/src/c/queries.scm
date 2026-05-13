;; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name.function)) @function

(function_definition
  declarator: (pointer_declarator
    declarator: (function_declarator
      declarator: (identifier) @name.function))) @function

;; Structs
(struct_specifier
  name: (type_identifier) @name.class) @class

;; Enums
(enum_specifier
  name: (type_identifier) @name.class) @class

;; Includes
(preproc_include
  path: [
    (string_literal) @import.source
    (system_lib_string) @import.source
  ]) @import
