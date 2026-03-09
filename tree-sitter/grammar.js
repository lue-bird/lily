/**
 * @file Lily grammar for tree-sitter
 * @author lue-bird
 * @license Unlicense
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "lily",
  extras: ($) => [/\s/],
  rules: {
    source_file: ($) =>
      seq(
        // the first comment lines are not guaranteed to be formatted with linebreaks in front
        repeat($.comment),
        // this is more strict than lily's parser but works for formatted code
        repeat(
          seq(
            $.indent0,
            // optional to allow extraneous linebreaks or trailing linebreaks
            seq(repeat($.comment), optional($.declaration)),
          ),
        ),
      ),
    comment: ($) => /#[^\n]*\n/,
    indent0: ($) => token.immediate("\n\n"),

    declaration: ($) =>
      choice(
        $.variable_declaration,
        $.choice_type_declaration,
        $.type_alias_declaration,
      ),

    type_alias_declaration: ($) => seq("type", $.type_construct, "=", $.type),

    choice_type_declaration: ($) =>
      seq(
        "choice",
        $.type_name,
        repeat($.type_variable),
        repeat($.variant_declaration),
      ),
    variant_declaration: ($) => seq("|", $.variant_name, optional($.type)),

    variable_declaration: ($) => seq($.expression_variable_name, $.expression),

    expression: ($) =>
      choice(
        $.expression_commented,
        $.expression_parenthesized,
        $.expression_typed,
        $.number,
        $.string,
        $.char,
        $.expression_variable_or_call,
        $.expression_dot_call,
        $.expression_variant,
        $.expression_vec,
        $.expression_record_or_record_update,
        $.expression_lambda,
        $.expression_match,
        $.expression_with_local_variable,
      ),
    expression_grouped: ($) =>
      choice(
        $.expression_parenthesized,
        $.number,
        $.string,
        $.char,
        $.expression_variable_name,
        $.variant_name,
        $.expression_vec,
        $.expression_record_or_record_update,
      ),
    expression_parenthesized: ($) => seq("(", $.expression, ")"),
    expression_commented: ($) => seq($.comment, $.expression),
    expression_typed: ($) => seq(":", $.type, ":", $.expression),
    // impossible without indentation-aware parsing.
    // This estimation might add the result as an additional argument to the variable declaration result instead
    expression_with_local_variable: ($) =>
      prec.right(
        seq(
          "=",
          $.introduced_local_variable,
          $.expression,
          optional($.expression),
        ),
      ),
    expression_variant: ($) =>
      prec.right(seq($.variant_name, optional($.expression))),
    expression_variable_or_call: ($) =>
      prec.right(seq($.expression_variable_name, repeat($.expression_grouped))),
    expression_variable_call: ($) =>
      seq($.expression_variable_name, repeat1($.expression_grouped)),
    expression_variable_name: ($) => $.lower_name,
    expression_dot_call: ($) =>
      seq(
        choice(
          $.expression_grouped,
          $.expression_variable_call,
          $.expression_dot_call,
        ),
        ".",
        $.expression_variable_or_call,
      ),
    // impossible without indentation-aware parsing.
    // This estimation will add additional cases to the case result instead
    expression_match: ($) =>
      seq(
        choice(
          $.expression_grouped,
          $.expression_variable_call,
          $.expression_dot_call,
        ),
        "|",
        $.pattern,
        ">",
        $.expression,
      ),
    expression_vec: ($) =>
      seq(
        "[",
        optional(seq($.expression, repeat(seq(",", $.expression)))),
        "]",
      ),
    expression_record_or_record_update: ($) =>
      seq(
        "{",
        optional(seq("..", $.expression, ",")),
        optional(seq($.expression_field, repeat(seq(",", $.expression_field)))),
        "}",
      ),
    expression_field: ($) => seq($.field_name, $.expression),
    expression_lambda: ($) =>
      seq("\\", $.pattern, repeat(seq(",", $.pattern)), ">", $.expression),

    pattern: ($) =>
      choice(
        $.pattern_commented,
        $.pattern_typed,
        $.number,
        $.string,
        $.char,
        $.pattern_ignored,
        $.introduced_local_variable,
        $.pattern_variant,
        $.pattern_record,
      ),
    pattern_commented: ($) => seq($.comment, $.pattern),
    pattern_typed: ($) => seq(":", $.type, ":", $.pattern),
    pattern_ignored: ($) => "_",
    pattern_variant: ($) => seq($.variant_name, optional($.pattern)),
    pattern_record: ($) =>
      seq(
        "{",
        optional(seq($.pattern_field, repeat(seq(",", $.pattern_field)))),
        "}",
      ),
    pattern_field: ($) => seq($.field_name, $.pattern),

    type: ($) =>
      choice(
        $.type_parenthesized,
        $.type_commented,
        $.type_variable,
        $.type_construct,
        $.type_record,
        $.type_function,
      ),
    type_grouped: ($) =>
      choice($.type_parenthesized, $.type_variable, $.type_name, $.type_record),
    type_commented: ($) => seq($.comment, $.type),
    type_parenthesized: ($) => seq("(", $.type, ")"),
    type_variable: ($) => $.upper_name,
    type_construct: ($) => seq($.type_name, repeat($.type_grouped)),
    type_function: ($) =>
      seq("\\", $.type, repeat(seq(", ", $.type)), ">", $.type),
    type_record: ($) =>
      seq(
        "{",
        optional(seq($.type_field, repeat(seq(",", $.type_field)))),
        "}",
      ),
    type_field: ($) => seq($.field_name, $.type),

    introduced_local_variable: ($) =>
      seq($.introduced_local_variable_name, optional("^")),
    introduced_local_variable_name: ($) => $.lower_name,
    field_name: ($) => $.lower_name,
    type_name: ($) => $.lower_name,
    variant_name: ($) => $.upper_name,
    char: ($) => seq("'", choice("\\\\", "\\'", /[^']/), "'"),
    string: ($) => choice($.string_quoted, $.string_ticked_lines),
    string_quoted: ($) => seq('"', repeat(choice("\\\\", '\\"', /[^"]/)), '"'),
    string_ticked_lines: ($) => prec.right(repeat1(seq("`", /[^\n]*\n/))),
    number: ($) =>
      // more strict than the lily parser, which allows .xyz
      // formatted code will only contains 0.xyz so this is okay
      /-?\+?\d+\.?\d*/,
    upper_name: ($) => /[A-Z][a-zA-Z0-9-]*/,
    lower_name: ($) => /[a-z][a-zA-Z0-9-]*/,
  },
});
