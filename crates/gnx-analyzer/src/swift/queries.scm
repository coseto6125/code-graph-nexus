;; Classes
(class_declaration
  name: (type_identifier) @name.class) @class

;; Structs
(struct_declaration
  name: (type_identifier) @name.interface) @interface

;; Protocols
(protocol_declaration
  name: (type_identifier) @name.interface) @interface

;; Enums
(enum_declaration
  name: (type_identifier) @name.interface) @interface

;; Functions
(function_declaration
  name: (simple_identifier) @name.function) @function

;; Methods in class/struct/protocol/enum body
(class_body
  (function_declaration
    name: (simple_identifier) @name.method) @method)

(struct_body
  (function_declaration
    name: (simple_identifier) @name.method) @method)

(protocol_body
  (function_declaration
    name: (simple_identifier) @name.method) @method)

(extension_body
  (function_declaration
    name: (simple_identifier) @name.method) @method)

;; Imports
(import_declaration) @import.source @import.name
