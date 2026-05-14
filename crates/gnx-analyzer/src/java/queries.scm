;; Classes
(class_declaration
  (modifiers [
    "public"
    "protected"
  ])? @export
  name: (identifier) @class.name
  interfaces: (super_interfaces (type_list (_) @heritage))?
  superclass: (superclass (_) @heritage)?
) @class

;; Interfaces
(interface_declaration
  (modifiers [
    "public"
    "protected"
  ])? @export
  name: (identifier) @interface.name
  interfaces: (extends_interfaces (type_list (_) @heritage))?
) @interface

;; Methods
(method_declaration
  (modifiers [
    "public"
    "protected"
  ])? @export
  type: (_) @type
  name: (identifier) @method.name
) @method

;; Constructors
(constructor_declaration
  (modifiers [
    "public"
    "protected"
  ])? @export
  name: (identifier) @method.name
) @method

;; Imports
(import_declaration
  [
    (scoped_identifier
      name: (identifier) @import.name) @import.source
    (identifier) @import.name @import.source
  ]
) @import
