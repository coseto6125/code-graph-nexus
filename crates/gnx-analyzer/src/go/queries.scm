;; Structs
(type_spec
  name: (type_identifier) @struct.name
  type: (struct_type
    (field_declaration_list
      (field_declaration
        !name
        type: [
          (type_identifier) @heritage
          (pointer_type (type_identifier) @heritage)
          (qualified_type) @heritage
        ]
      )*
    )?
  )
) @struct

;; Interfaces
(type_spec
  name: (type_identifier) @interface.name
  type: (interface_type
    (method_spec_list
      [
        (method_spec name: (field_identifier))
        (type_identifier) @heritage
        (qualified_type) @heritage
      ]*
    )?
  )
) @interface

;; Methods
(method_declaration
  receiver: (parameter_list
    (parameter_declaration
      type: [
        (type_identifier) @type
        (pointer_type (type_identifier) @type)
        (qualified_type) @type
      ]
    )
  )
  name: (field_identifier) @method.name
  result: [
    (type_identifier) @type
    (pointer_type (type_identifier) @type)
    (qualified_type) @type
    (parameter_list) @type
  ]?
) @method

;; Functions
(function_declaration
  name: (identifier) @function.name
  result: [
    (type_identifier) @type
    (pointer_type (type_identifier) @type)
    (qualified_type) @type
    (parameter_list) @type
  ]?
) @function

;; Imports
(import_spec
  name: (package_identifier) @import.alias
  path: (string_literal) @import.source) @import

(import_spec
  path: (string_literal) @import.source) @import
