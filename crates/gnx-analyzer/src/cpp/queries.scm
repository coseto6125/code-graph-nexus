;; Classes and Structs
(class_specifier
  name: [
    (type_identifier)
    (template_type)
  ] @name.class
  (base_class_clause
    (_ (type_identifier) @heritage))?
) @class

(struct_specifier
  name: [
    (type_identifier)
    (template_type)
  ] @name.class
  (base_class_clause
    (_ (type_identifier) @heritage))?
) @class

;; Functions
(function_definition
  type: (_)? @type
  declarator: (function_declarator
    declarator: [
      (identifier) @name.function
      (reference_declarator (identifier) @name.function)
      (pointer_declarator (identifier) @name.function)
    ]
  )
) @function

;; Methods
(function_definition
  type: (_)? @type
  declarator: (function_declarator
    declarator: [
      (field_identifier) @name.method
      (reference_declarator (field_identifier) @name.method)
      (pointer_declarator (field_identifier) @name.method)
      (scoped_identifier
        name: [
          (identifier)
          (field_identifier)
        ] @name.method
      )
      (reference_declarator (scoped_identifier name: (_) @name.method))
      (pointer_declarator (scoped_identifier name: (_) @name.method))
    ]
  )
) @method

;; C++20 Modules & Exports
(export_declaration
  [
    (class_specifier)
    (struct_specifier)
    (function_definition)
    (declaration)
    (namespace_definition)
  ] @export
)

;; Preprocessor Includes
(preproc_include
  path: [
    (string_literal)
    (system_lib_string)
  ] @import.source
) @import

;; Namespace Aliases
(namespace_alias_definition
  name: (identifier) @alias
  value: [
    (identifier)
    (scoped_identifier)
  ] @import.source
) @import
