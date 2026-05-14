;; Classes
(class_declaration
  name: (identifier) @class.name
  superclass: (superclass type: (type (_) @heritage))?
  (mixins (type (_) @heritage))?
  interfaces: (interfaces (type (_) @heritage))?) @class

;; Enums
(enum_declaration
  name: (identifier) @interface.name) @interface

;; Mixins
(mixin_declaration
  name: (identifier) @interface.name
  interfaces: (interfaces (type (_) @heritage))?) @interface

;; Methods
(method_declaration
  signature: (method_signature
    (function_signature
      return_type: (type)? @type
      name: (identifier) @method.name))) @method

;; Functions
(function_signature
  return_type: (type)? @type
  name: (identifier) @function.name) @function

;; Properties
(declaration
  (initialized_identifier_list
    (initialized_identifier name: (identifier) @property.name))) @property

;; Imports
(library_import
  (_)
  (_) @import.source
  (_) @import.alias ?) @import
