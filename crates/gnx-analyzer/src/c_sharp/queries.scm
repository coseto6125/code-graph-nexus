;; Classes
(class_declaration
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Structs
(struct_declaration
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Interfaces
(interface_declaration
  (modifier)* @export
  name: (identifier) @name.interface
  (base_list (_)* @heritage)?
) @interface

;; Enums
(enum_declaration
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Records
(record_declaration
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Methods
(method_declaration
  (modifier)* @export
  type: (_) @type
  name: (identifier) @name.method
) @method

;; Constructors
(constructor_declaration
  (modifier)* @export
  name: (identifier) @name.method
) @method

;; Local Functions
(local_function_statement
  (modifier)* @export
  type: (_) @type
  name: (identifier) @name.function
) @function

;; Using directives (Imports)
(using_directive
  alias: (identifier)? @import.alias
  name: [
    (identifier)
    (qualified_name)
  ] @import.name @import.source
) @import
