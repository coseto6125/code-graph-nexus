;; Classes and Structs
(class_specifier
  name: [
    (type_identifier)
    (template_type)
  ] @name.class) @class

(struct_specifier
  name: [
    (type_identifier)
    (template_type)
  ] @name.class) @class

;; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name.function)) @function

(function_definition
  declarator: (function_declarator
    declarator: (reference_declarator (identifier) @name.function))) @function

(function_definition
  declarator: (function_declarator
    declarator: (pointer_declarator (identifier) @name.function))) @function

;; Methods
(function_definition
  declarator: (function_declarator
    declarator: (field_identifier) @name.method)) @method

(function_definition
  declarator: (function_declarator
    declarator: (reference_declarator (field_identifier) @name.method))) @method

(function_definition
  declarator: (function_declarator
    declarator: (pointer_declarator (field_identifier) @name.method))) @method

(function_definition
  declarator: (function_declarator
    declarator: (scoped_identifier
      name: [
        (identifier)
        (field_identifier)
      ] @name.method))) @method

(function_definition
  declarator: (function_declarator
    declarator: (reference_declarator (scoped_identifier name: (_) @name.method)))) @method

(function_definition
  declarator: (function_declarator
    declarator: (pointer_declarator (scoped_identifier name: (_) @name.method)))) @method

;; Preprocessor Includes
(preproc_include
  path: [
    (string_literal)
    (system_lib_string)
  ] @import.source) @import
