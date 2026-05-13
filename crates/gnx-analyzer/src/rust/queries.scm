;; Structs
(struct_item
  name: (type_identifier) @name.class) @class

;; Enums
(enum_item
  name: (type_identifier) @name.class) @class

;; Traits
(trait_item
  name: (type_identifier) @name.interface) @interface

;; Functions
(function_item
  name: (identifier) @name.function) @function

;; Methods in impl
(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name.method) @method))

;; Methods in trait
(trait_item
  body: (declaration_list
    (function_signature_item
      name: (identifier) @name.method) @method))

;; Imports (use std::collections::HashMap)
(use_declaration
  argument: (scoped_identifier
    name: (identifier) @import.name) @import.source) @import

;; Imports (use something)
(use_declaration
  argument: (identifier) @import.name @import.source) @import

;; Imports (use std::collections::{HashMap, HashSet})
(use_declaration
  argument: (scoped_use_list
    path: (_) @import.source
    list: (use_list
      (identifier) @import.name))) @import
