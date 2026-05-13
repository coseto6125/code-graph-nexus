; Imports
(import_header
  (identifier) @import.source) @import

; Classes
(class_declaration
  (type_identifier) @name.class) @class

; Objects
(object_declaration
  (type_identifier) @name.class) @class

; Functions
(function_declaration
  (simple_identifier) @name.function) @function
