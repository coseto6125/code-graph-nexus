;; Classes
(class_declaration
  name: (identifier) @name.class) @class

;; Structs
(struct_declaration
  name: (identifier) @name.class) @class

;; Interfaces
(interface_declaration
  name: (identifier) @name.interface) @interface

;; Enums
(enum_declaration
  name: (identifier) @name.class) @class

;; Records
(record_declaration
  name: (identifier) @name.class) @class

;; Methods
(method_declaration
  name: (identifier) @name.method) @method

;; Constructors
(constructor_declaration
  name: (identifier) @name.method) @method

;; Local Functions
(local_function_statement
  name: (identifier) @name.function) @function

;; Using directives (Imports)
(using_directive
  [
    (identifier)
    (qualified_name)
  ] @import.name @import.source) @import
