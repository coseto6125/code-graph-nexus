;; Classes
(class_definition
  name: (identifier) @name.class) @class

;; Enums
(enum_declaration
  name: (identifier) @name.interface) @interface

;; Mixins
(mixin_declaration
  name: (identifier) @name.interface) @interface

;; Extensions
(extension_declaration
  name: (identifier) @name.interface) @interface

;; Methods
(method_signature
  name: (identifier) @name.method) @method

(method_signature
  (function_signature
    name: (identifier) @name.method)) @method

(method_signature
  (getter_signature
    name: (identifier) @name.method)) @method

(method_signature
  (setter_signature
    name: (identifier) @name.method)) @method

(method_signature
  (constructor_signature
    name: (identifier) @name.method)) @method

;; Functions
(function_signature
  name: (identifier) @name.function) @function

;; Imports
(library_import
  uri: (string_literal) @import.source @import.name) @import
