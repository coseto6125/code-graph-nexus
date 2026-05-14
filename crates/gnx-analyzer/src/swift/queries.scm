;; Classes
(class_declaration
  (modifiers (visibility_modifier) @export)?
  name: (type_identifier) @name.class
  (type_inheritance_clause (type_identifier) @heritage)?
) @class

;; Structs
(struct_declaration
  (modifiers (visibility_modifier) @export)?
  name: (type_identifier) @name.interface
  (type_inheritance_clause (type_identifier) @heritage)?
) @interface

;; Protocols
(protocol_declaration
  (modifiers (visibility_modifier) @export)?
  name: (type_identifier) @name.interface
  (type_inheritance_clause (type_identifier) @heritage)?
) @interface

;; Enums
(enum_declaration
  (modifiers (visibility_modifier) @export)?
  name: (type_identifier) @name.interface
  (type_inheritance_clause (type_identifier) @heritage)?
) @interface

;; Functions
(function_declaration
  (modifiers (visibility_modifier) @export)?
  name: (simple_identifier) @name.function
  (function_signature result: (type_identifier) @type)?
) @function

;; Methods
(function_declaration
  (modifiers (visibility_modifier) @export)?
  name: (simple_identifier) @name.method
  (function_signature result: (type_identifier) @type)?
) @method

;; Imports
(import_declaration
  path: (import_path) @import.name @import.source
)
