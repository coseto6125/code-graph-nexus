;; Classes
(class_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Structs
(struct_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Interfaces
(interface_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.interface
  (base_list (_)* @heritage)?
) @interface

;; Enums
(enum_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Records
(record_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.class
  (base_list (_)* @heritage)?
) @class

;; Methods
(method_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  returns: (_) @type
  name: (identifier) @name.method
) @method

;; Constructors
(constructor_declaration
  (attribute_list)* @decorator
  (modifier)* @export
  name: (identifier) @name.method
) @method

;; Local Functions
(local_function_statement
  (attribute_list)* @decorator
  (modifier)* @export
  returns: (_) @type
  name: (identifier) @name.function
) @function

;; Using directives (Imports). Three patterns:
;; - `using X;` / `using X.Y;` — plain
;; - `using static X.Alpha;` — static-member import (the `static` modifier
;;   is anonymous; the actual qualified-name child holds the path)
;; - `using A = X.Alpha;` — alias
;;
;; The `name:` field is unreliable across c_sharp grammar versions for
;; non-alias forms, so the plain/static patterns match the unnamed
;; qualified-name / identifier child directly.
(using_directive
  (qualified_name) @import.name @import.source
) @import

(using_directive
  (identifier) @import.name @import.source
) @import

(using_directive
  name: (identifier) @import.alias
  (_) @import.name @import.source
) @import
