;; Classes
(class_definition
  name: (identifier) @class.name
  (extends_clause (type_not_void (type_identifier) @heritage))?
  (implements_clause (type_list (type_not_void (type_identifier) @heritage)))?
  (with_clause (type_list (type_not_void (type_identifier) @heritage)))?
) @class

;; Enums
(enum_declaration
  name: (identifier) @interface.name
  (implements_clause (type_list (type_not_void (type_identifier) @heritage)))?
) @interface

;; Mixins
(mixin_declaration
  name: (identifier) @interface.name
  (on_clause (type_list (type_not_void (type_identifier) @heritage)))?
  (implements_clause (type_list (type_not_void (type_identifier) @heritage)))?
) @interface

;; Extensions
(extension_declaration
  name: (identifier) @interface.name
) @interface

;; Methods
(method_signature
  return_type: (type_annotation)? @type
  name: (identifier) @method.name) @method

(method_signature
  return_type: (type_annotation)? @type
  (function_signature
    name: (identifier) @method.name)) @method

(method_signature
  (getter_signature
    return_type: (type_annotation)? @type
    name: (identifier) @method.name)) @method

(method_signature
  (setter_signature
    name: (identifier) @method.name)) @method

(method_signature
  (constructor_signature
    name: (identifier) @method.name)) @method

;; Functions
(function_signature
  return_type: (type_annotation)? @type
  name: (identifier) @function.name) @function

;; Imports
(library_import
  uri: (string_literal) @import.source
  (import_prefix (identifier) @import.alias)?
) @import
