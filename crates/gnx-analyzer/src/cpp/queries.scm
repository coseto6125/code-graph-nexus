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
      (qualified_identifier
        name: [
          (identifier)
          (field_identifier)
        ] @name.method
      )
      (reference_declarator (qualified_identifier name: (_) @name.method))
      (pointer_declarator (qualified_identifier name: (_) @name.method))
    ]
  )
) @method

;; Preprocessor Includes
(preproc_include
  path: [
    (string_literal)
    (system_lib_string)
  ] @import.source
) @import

;; Namespace Aliases
(namespace_alias_definition
  name: (namespace_identifier) @alias
  (namespace_identifier) @import.source
) @import
