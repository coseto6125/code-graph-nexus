;; Functions
(function_definition
  name: (name) @name.function) @function

;; Classes
(class_declaration
  name: (name) @name.class) @class

;; Interfaces
(interface_declaration
  name: (name) @name.interface) @interface

;; Methods
(method_declaration
  name: (name) @name.method) @method

;; Namespaces
(namespace_definition
  name: (namespace_name) @name.namespace) @namespace

;; Imports
(namespace_use_clause
  name: (name) @import.source
  alias: (name)? @import.name) @import

(namespace_use_group
  (namespace_use_clause
    name: (name) @import.source
    alias: (name)? @import.name)) @import
