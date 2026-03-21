#![allow(non_upper_case_globals)]

use lily_compile as lily;
fn main() {
    yew::Renderer::<State>::new().render();
}

struct State {
    text_area_content: String,
    selected_example: Example,
}
enum Event {
    TextAreaContentChanged(String),
    ExampleSelected(Example),
}
impl yew::Component for State {
    type Message = Event;

    type Properties = ();

    fn create(_: &yew::Context<Self>) -> Self {
        let selected_example: Example = web_sys::window()
            .and_then(|window| window.location().search().ok())
            .and_then(|search| {
                let example_name = search.trim_start_matches("?example=");
                example_infos
                    .into_iter()
                    .find(|(_, example_info)| example_info.name.replace(' ', "-") == example_name)
                    .map(|(example, _)| example)
            })
            .unwrap_or(Example::HelloWorld);
        State {
            text_area_content: example_source(selected_example).to_string(),
            selected_example: selected_example,
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, event: Event) -> bool {
        match event {
            Event::TextAreaContentChanged(new_text_area_content) => {
                self.text_area_content = new_text_area_content;
            }
            Event::ExampleSelected(selected_example) => {
                self.selected_example = selected_example;
                self.text_area_content = example_source(selected_example).to_string();
                if let Some(window) = web_sys::window()
                    && let Ok(history) = window.history()
                {
                    let _ = history.push_state_with_url(
                        &web_sys::wasm_bindgen::JsValue::NULL,
                        "",
                        Some(&format!(
                            "{}?example={}",
                            window
                                .location()
                                .pathname()
                                .unwrap_or_else(|_| String::new()),
                            example_name(selected_example).replace(' ', "-")
                        )),
                    );
                }
            }
        }
        true
    }

    fn view(&self, context: &yew::Context<Self>) -> yew::Html {
        yew_element(
            "main",
            [],
            [
                yew_element(
                    "h2",
                    [("style", "white-space: pre-line;".into())],
                    [yew_text(
                        "a very simple, explicitly boring,
purely functional programming language
that compiles to rust: lily",
                    )],
                ),
                yew_link_to("https://codeberg.org/lue-bird/lily", "source code"),
                yew_text(". Try an example: "),
                yew_element(
                    "p",
                    [
                        ("id", "example-select".into()),
                        ("style", "display: inline".into()),
                    ],
                    example_infos
                        .into_iter()
                        .map(|(example_kind, example_info)| {
                            let mut button = yew::virtual_dom::VTag::new("button");
                            let link = context.link().clone();
                            button.add_listener(std::rc::Rc::new(yew_listener(
                                yew::virtual_dom::ListenerKind::onpointerdown,
                                move |_| {
                                    link.send_message(Event::ExampleSelected(example_kind));
                                },
                            )));
                            button.add_child(yew_text(example_info.name));
                            yew::Html::from(button)
                        }),
                ),
                yew_linebreak(),
                yew_playground(self.selected_example, &self.text_area_content, context),
                yew_sub_heading("core declarations"),
                lily_core_declarations_html(),
            ],
        )
    }
}
fn yew_playground(
    selected_example: Example,
    text_area_content: &str,
    context: &yew::Context<State>,
) -> yew::Html {
    let mut interactive_text_area = yew::virtual_dom::VTag::new("textarea");
    interactive_text_area.add_attribute("autocorrect", "off");
    interactive_text_area.add_attribute("spellcheck", "false");
    interactive_text_area.add_attribute("autofocus", "true");
    interactive_text_area.add_attribute("name", "playground");
    // this is all _very_ annoying
    interactive_text_area.add_attribute(
        "style",
        format!(
            r#"field-sizing: content;
            height: {}em;
            width: 100%;
            background: none;
            color: transparent;
            border: none;
            line-height: inherit;
            resize: none;
            overflow: hidden;
            font-family: "Liga NovaMono", monospace, sans-serif;
            font-size: medium;
            resize: none;
            caret-color: white;
            position: relative;
            top: 0.335em;
            left: -0.15em"#,
            (text_area_content.lines().count()
                + if text_area_content.ends_with("\n") {
                    1
                } else {
                    0
                }) as f64
                // problematic: why does this need to be done at all,
                // given that em units are used?
                * 1.15
                * 1.2
                + 0.5
        ),
    );
    interactive_text_area.add_property("value", text_area_content);
    let cursor_offset = text_area_content.find("insert your name here").unwrap_or(0);
    interactive_text_area.add_property("selectionStart", cursor_offset);
    interactive_text_area.add_property("selectionEnd", cursor_offset);
    // this seems a bit over the top for a simple event handler
    let link = context.link().clone();
    interactive_text_area.add_listener(std::rc::Rc::new(yew_listener(
        yew::virtual_dom::ListenerKind::oninput,
        move |event: web_sys::Event| {
            let Some(event_target) = event.target() else {
                return;
            };
            let text_area_object: web_sys::HtmlTextAreaElement = web_sys::HtmlTextAreaElement::from(
                web_sys::wasm_bindgen::JsValue::from(event_target),
            );
            link.send_message(Event::TextAreaContentChanged(text_area_object.value()));
        },
    )));

    let syntax_project = lily::parse_syntax_project(&text_area_content);
    let mut highlights = Vec::new();
    lily::syntax_highlight_project_into(&mut highlights, &syntax_project);

    yew_element(
        "div",
        [("style", "position: relative;".into())],
        [
            interactive_text_area.into(),
            yew_element(
                "div",
                [
                    ("aria-hidden", "true".into()),
                    (
                        "style",
                        "position: absolute; top: 0; left: 0; pointer-events: none; user-select: none;".into(),
                    ),
                ],
                [highlighted_lily_source_to_html(&text_area_content, &mut highlights.into_iter())],
            ),
            yew_element("p", [("style", "font: inherit; white-space: pre-line;".into())], [
                yew_text_dynamic({
                    let mut so_far = String::new();
                    let mut errors = Vec::new();
                    let compiled_project = lily::project_compile_to_rust(&mut errors, &syntax_project);
                    let evaluated_variables = lily::evaluate_syntax_project(&compiled_project.type_aliases, &compiled_project.choice_types, &compiled_project.variable_graph, &compiled_project.variable_declaration_by_graph_node);
                    let mut evaluated_variables = evaluated_variables.iter().collect::<Vec<_>>();
                    evaluated_variables.sort_by_key(|(name, _)| *name);
                    for (evaluated_variable_name, evaluated_variable_expression) in evaluated_variables {
                        match evaluated_variable_expression {
                            // skip functions, displaying them is not useful
                            lily::EvaluatedExpression::Closure { .. } |
                            lily::EvaluatedExpression::CoreFunction(_) => {}
                            _ => {
                                so_far.push_str("↪ ");
                                so_far.push_str(&evaluated_variable_name);
                                so_far.push_str(" is ");
                                lily::evaluated_expression_info_into(&mut so_far, &evaluated_variable_expression);
                                so_far.push('\n');
                            }
                        }
                    }
                    for error in errors {
                        use std::fmt::Write as _;
                        let _ = write!(so_far, "\n⚠︎ line {} char {}: {}", error.range.start.line, error.range.start.character, error.message);
                    }
                    so_far
                })
            ]),
            yew_element("p", [("style", "font: inherit; white-space: pre-line;".into())], [
                yew_text("💡 "),
                yew_text(example_explainer(selected_example))
            ])
        ],
    )
}
fn lily_core_declarations_html() -> yew::Html {
    let mut container = yew::virtual_dom::VTag::new("section");
    let mut choice_types_sorted = lily::core_choice_type_infos.iter().collect::<Vec<_>>();
    choice_types_sorted.sort_unstable_by_key(|(name, _)| *name);
    container.add_children(choice_types_sorted.into_iter().map(
        |(core_choice_type_name, core_choice_type_info)| {
            lily_choice_type_to_html(core_choice_type_name, core_choice_type_info)
        },
    ));
    let mut variable_declarations_sorted = lily::core_variable_declaration_infos
        .iter()
        .collect::<Vec<(&lily::Name, _)>>();
    variable_declarations_sorted.sort_unstable_by_key(|(name, _)| *name);
    container.add_children(variable_declarations_sorted.into_iter().map(
        |(core_variable_name, core_variable_info)| {
            lily_project_variable_to_html(core_variable_name, core_variable_info)
        },
    ));
    container.into()
}
fn lily_choice_type_to_html(
    name: &lily::Name,
    choice_type_info: &lily::ChoiceTypeInfo,
) -> yew::Html {
    let mut section_container = yew::virtual_dom::VTag::new("section");
    section_container.add_child(documentation_heading_html(name));
    if !choice_type_info.type_variants.is_empty() {
        let mut choice_type_info_string: String = String::new();
        lily::syntax_choice_type_declaration_into(
            &mut choice_type_info_string,
            Some(name),
            &choice_type_info.parameters,
            &choice_type_info.variants,
        );
        section_container.add_child(lily_project_source_to_html(&choice_type_info_string))
    }
    if let Some(documentation) = &choice_type_info.documentation {
        section_container.add_child(lily_documentation_markdown_to_html(documentation));
    }
    section_container.into()
}
fn lily_project_source_to_html(project_source: &str) -> yew::Html {
    let syntax_project = lily::parse_syntax_project(&project_source);
    let mut highlights = Vec::new();
    lily::syntax_highlight_project_into(&mut highlights, &syntax_project);
    highlighted_lily_source_to_html(&project_source, &mut highlights.into_iter())
}
fn lily_project_variable_to_html(
    name: &lily::Name,
    project_variable_info: &lily::CompiledVariableDeclarationInfo,
) -> yew::Html {
    let mut section_html = yew::virtual_dom::VTag::new("section");
    section_html.add_child(documentation_heading_html(name));
    if let Some(type_) = &project_variable_info.type_ {
        let mut type_string = String::from(":");
        lily::type_info_into(&mut type_string, 1, type_);
        type_string.push(':');
        if let Some(syntax_type_node) = lily::parse_syntax_type(&mut lily::ParseState {
            source: &type_string,
            indent: 0,
            lower_indents_stack: vec![],
            offset_utf8: 1,
            position: lsp_types::Position {
                line: 0,
                character: 1,
            },
        }) {
            let mut highlights = Vec::new();
            lily::syntax_highlight_type_into(
                &mut highlights,
                lily::syntax_node_as_ref(&syntax_type_node),
            );
            section_html.add_child(highlighted_lily_source_to_html(
                &type_string,
                &mut highlights.into_iter(),
            ));
        }
    }
    if let Some(documentation) = &project_variable_info.documentation {
        section_html.add_child(lily_documentation_markdown_to_html(documentation));
    }
    section_html.into()
}
fn documentation_heading_html(name: &str) -> yew::Html {
    yew_element("h4", [], [yew_link_to_self(name)])
}
fn yew_link_to_self(name: &str) -> yew::Html {
    let id = name.replace(" ", "-");
    yew_element(
        "a",
        [("href", format!("#{id}").into()), ("id", id.into())],
        [yew_text_dynamic(format!("#{name}"))],
    )
}
fn yew_sub_heading(name: &str) -> yew::Html {
    yew_element("h3", [], [yew_link_to_self(name)])
}
fn highlighted_lily_source_to_html(
    source: &str,
    mut highlights: impl Iterator<Item = lily::SyntaxNode<lily::SyntaxHighlightKind>>,
) -> yew::Html {
    let mut html = yew::virtual_dom::VTag::new("pre");
    html.add_attribute("style", r#"line-height: inherit; font-size: medium; font-family: "Liga NovaMono", monospace, sans-serif; margin-top: 0.5em"#);

    let mut maybe_next_highlight = highlights.next();
    for (source_line_index, source_line) in source.lines().enumerate() {
        let mut current_offset_in_line: usize = 0;
        while let Some(next_highlight) = maybe_next_highlight
            && source_line_index as u32 == next_highlight.range.start.line
        {
            let highlight_start_offset_in_line =
                utf16_offset_to_utf8_in(source_line, next_highlight.range.start.character as usize);
            let highlight_end_offset_in_line =
                utf16_offset_to_utf8_in(source_line, next_highlight.range.end.character as usize);

            html.add_child(yew_element(
                "code",
                [],
                [yew_text_dynamic(
                    &source_line[current_offset_in_line..highlight_start_offset_in_line],
                )],
            ));
            html.add_child(yew_element(
                "code",
                [(
                    "style",
                    format!(
                        "color: {}",
                        lily_syntax_highlight_kind_to_css_color(next_highlight.value)
                    )
                    .into(),
                )],
                [yew_text_dynamic(
                    &source_line[highlight_start_offset_in_line..highlight_end_offset_in_line],
                )],
            ));

            current_offset_in_line = highlight_end_offset_in_line;
            maybe_next_highlight = highlights.next();
        }
        html.add_child(yew_element(
            "code",
            [],
            [yew_text_dynamic(&source_line[current_offset_in_line..])],
        ));
        html.add_child(yew_element("code", [], [yew_text("\n")]));
    }
    html.into()
}
fn lily_syntax_highlight_kind_to_css_color(kind: lily::SyntaxHighlightKind) -> &'static str {
    match kind {
        lily::SyntaxHighlightKind::Type => "rgb(0,255,255)",
        lily::SyntaxHighlightKind::TypeVariable => "rgb(0,170,255)",
        lily::SyntaxHighlightKind::Variant => "rgb(120,235,30)",
        lily::SyntaxHighlightKind::Field => "rgb(255, 145, 0)",
        lily::SyntaxHighlightKind::Variable | lily::SyntaxHighlightKind::DeclaredVariable => {
            "rgb(255, 225, 140)"
        }
        lily::SyntaxHighlightKind::Comment => "rgb(140,140,140)",
        lily::SyntaxHighlightKind::String | lily::SyntaxHighlightKind::Number => "rgb(180,100,255)",
        lily::SyntaxHighlightKind::KeySymbol => "rgb(255,35,190)",
    }
}
fn lily_documentation_markdown_to_html(lily_documentation_markdown: &str) -> yew::Html {
    let mut html = yew::virtual_dom::VTag::new("p");
    let mut maybe_current_code_block_start_line_index: Option<usize> = None;
    for (lily_documentation_markdown_line_index, lily_documentation_markdown_line) in
        lily_documentation_markdown.lines().enumerate()
    {
        match lily_documentation_markdown_line {
            "```" | "```lily" => match maybe_current_code_block_start_line_index {
                None => {
                    maybe_current_code_block_start_line_index =
                        Some(lily_documentation_markdown_line_index);
                }
                Some(current_code_block_start_line_index) => {
                    maybe_current_code_block_start_line_index = None;
                    html.add_child(lily_project_source_to_html(
                        &lily_documentation_markdown
                            .lines()
                            .skip(current_code_block_start_line_index + 1)
                            .take(
                                lily_documentation_markdown_line_index
                                    - current_code_block_start_line_index
                                    - 1,
                            )
                            .collect::<Vec<&str>>()
                            .join("\n"),
                    ));
                }
            },
            "" => {
                if maybe_current_code_block_start_line_index.is_none() {
                    html.add_child(yew_linebreak());
                }
            }
            _ => {
                if maybe_current_code_block_start_line_index.is_none() {
                    // insert space before because otherwise if the previous line ends in
                    // punctuation like , the text in the next line would be attached directly after it
                    html.add_child(yew_text(" "));
                    html.add_child(yew_text_dynamic(lily_documentation_markdown_line));
                }
            }
        }
    }
    html.into()
}

#[derive(Copy, Clone)]
struct ExampleInfo {
    source: &'static str,
    explainer: &'static str,
    name: &'static str,
}
fn example_source(example: Example) -> &'static str {
    example_info(example).source
}
fn example_explainer(example: Example) -> &'static str {
    example_info(example).explainer
}
const fn example_name(example: Example) -> &'static str {
    example_info(example).name
}
#[derive(Copy, Clone)]
enum Example {
    HelloWorld,
    Variable,
    Numbers,
    Text,
    FunctionCall,
    Types,
    Function,
    Record,
    Choice,
    Match,
    Vec,
    Comment,
    Extras,
}
static example_infos: [(Example, ExampleInfo); 13] = {
    const fn entry(example: Example) -> (Example, ExampleInfo) {
        (example, example_info(example))
    }
    [
        entry(Example::HelloWorld),
        entry(Example::Variable),
        entry(Example::Numbers),
        entry(Example::Text),
        entry(Example::FunctionCall),
        entry(Example::Types),
        entry(Example::Function),
        entry(Example::Match),
        entry(Example::Record),
        entry(Example::Choice),
        entry(Example::Vec),
        entry(Example::Comment),
        entry(Example::Extras),
    ]
};
const fn example_info(example: Example) -> ExampleInfo {
    match example {
        Example::HelloWorld => ExampleInfo {
            name:"hello world",
            source: r#"
your-output
    greet "insert your name here"

greet \:str:name >
    strs-flatten [ "Hii, ", name, " ˖᯽"  ]
"#,
            explainer: "We declare a greet variable that takes a string argument and concatenates it with other strings to form a message.
For more details, click through the examples above and try changing things.",
        },
        Example::Variable => ExampleInfo {
            name: "variable",
            source: r#"
your-project-variable-name
    dec-pi
"#,
            explainer: "A name can hold values or functions.
You can add a new one to your project by putting a name consisting of a-z, A-Z, 0-9 or - at the start of a new line, followed by its value (for example 0 or \":)\").
Lily also has \"core\" variables like int-add that any project can reference.
To see the full list, scroll down or search the site for #some-name-to-search-for.",
        },
        Example::Numbers => ExampleInfo {
            name: "numbers",
            source: r#"
number-with-a-decimal-point -2.7
number-ending-in-decimal-point 4.

unsigned-integer 2
unsigned-integer-zero 0

signed-integer +2
signed-integer-zero 00
"#,
            explainer: "3 types of numbers:
- dec: floating point number; must have a decimal point and can have a sign
- int: integer; must have a sign (-, + or 0). So zero is represented as 00
- unt: unsigned integer: any number without a sign or decimal point.
  Used mainly for lengths, indexes, counting and similar",
        },
        Example::Text => ExampleInfo {
            name: "text",
            source: r#"
single-character 'a'
escaped-quote '\''
escaped-backslash '\\'
escaped-tab '\t'
escaped-linebreak '\n'
escaped-carriage-return '\r'
by-code-point-hex '\u{1F648}'

cat "₍^. .^₎⟆"
escaped-double-quote "\"hello\""
raw-string
    `Welcome ladies, enbies, genties, everyone!
    `I wish you a "wonderatious", \wild\, 'raw', `crazy`, \{cool} \n\r\t week.
    `
"#,
            explainer: "single characters are wrapped in '...', strings are wrapped in \"...\".
To avoid escaping a bunch for long, literal strings, you can start with ` and the rest of the line is used literally.
If multiple literal string lines are after each other, they are combined into one string with linebreaks (\\n) between each line.
To end such a raw literal string with a linebreak, add ` as the last line",
        },
        Example::FunctionCall => ExampleInfo {
            name: "function call",
            source: r#"
regular-call
    dec-add 1.2 2.3

equivalent-dot-call
    1.2 .dec-add 2.3

nested-call
    str-attach-char
        (str-attach
            (str-attach-unt
                "I'm "
                82
            )
            " years old"
        )
        '!'

dot-call-chain
    "I'm "
        .str-attach-unt 82
        .str-attach " years old"
        .str-attach-char '!'
"#,
            explainer: "Variables holding a function can be followed by whitespace-separated arguments to call that function (In other languages this is often done with: function(arg, uments)).
For builders, pipelines and more, this can get a bit messy.
Therefore, you can also call functions as: first-argument .function second-argument-etc
(in other languages this style is often tied to methods on objects).",
        },
        Example::Types => ExampleInfo {
            name: "types",
            source: r#"
long-call
    :str:
    +4
    .int-negate
    .int-mul +10
    .int-to-str

integer :int:+4
unsigned-integer :unt:0
decimal-point-number :dec:23.4
unicode-rune :char:'%'

integer-add
    :\int, int > int:
    int-add

type string = str
type uint = unt
type stringify From = \From > str

integer-to-str
    :stringify int:int-to-str
"#,
            explainer: "Any expression can have its explicit type in front which is checked to match.
This can make it easier to know what for example a long chain of operations returns.
More importantly though, some syntax requires explicit types, like function parameters or empty vectors.
Special types:
- functions: \\, comma-separated parameter types, > result type
- variable: uppercase name, represents a type that is filled in when used

You can also create short name aliases for types you often use using type, type name, space-separated variables, result type.
These will become especially useful in the example for records
",
        },
        Example::Function => ExampleInfo {
            name: "function",
            source: r#"
int-subtract \:int:number, :int:to-subtract >
    int-add number (int-negate to-subtract)

use-int-subtract
    int-subtract +3 +4

same-in-same-out \:Anything:anything >
    anything

use-same-in-same-out
    same-in-same-out "oo ee oo"

ignoring-the-incoming-value \:AnyQuestion:_ >
    42

chain-functions \:A:a, :\A > B:a-to-b, :\B > C:b-to-c >
    a .a-to-b .b-to-c

use-chain-functions
    +3 .chain-functions int-negate int-to-str
"#,
            explainer: "To introduce a new function to your project, add a new variable with a function expression.
You can create and use these kinds of functions everywhere in your code and even pass functions as arguments to functions.
A function consists of \\, a comma-separated list of parameter patterns, > and the result.
The simplest pattern is a :type: followed by a local variable name. See the match example for other kinds of patterns.
These local functions can also reference local variables not introduced in the parameters so other languages call this a \"closure\".",
        },
        Example::Match => ExampleInfo {
            name: "match",
            source: r#"
binary-char-to-unt \:char:char >
    char
    | '0' > 0
    | :char:_ > 1

with-intermediate-local-variable
    dec-mul dec-pi 2.07
    | :dec:intermediate >
    dec-mul intermediate intermediate
"#,
            explainer: "To decide what to do based on the shape of some value, followup with | pattern > result cases after that value.
It's checked for missing cases so you don't forget some shape.
Works for the core types str, char, dec, int, unt as well as \"record\" and \"choice\" whose examples show how to pattern match them.
To avoid increasing levels of indentation, you can keep the last case result unindented
(In other languages, this usually done with an early return, elseif or let else)."
        },
        Example::Record => ExampleInfo {
            name: "record",
            source: r#"
multiple-shapes-of-data-bundled-together
    { weight 1.0
    , color { r 255, g 100, b 40 }
    , position { x 0.0, y 0.0 }
    }

default-config
    { line-separator "\r\n"
    , element-separator ";"
    , version 2
    }

our-specific-derived-config
    { ..default-config, element-separator "," }

type vector =
    { x dec, y dec }

example-vector
    { x 2.0, y 3.0 }

vector-length \{ x :dec:x, y :dec:y } >
    dec-add (dec-mul x x) (dec-mul y y) .dec-to-power-of 0.5

use-vector-length
    vector-length example-vector

simple-record-access
    example-vector
    | { x :dec:x, y :dec:_ } >
    x
"#,
            explainer: "Passing some infos which are connected to each other as separate arguments is inconvenient and error-prone.
A \"record\" gives the individual values a field name and combines them into one value
(other languages usually call this struct(ure) or constant data object).
When you end up passing this record a bunch, it's probably a good idea to make a type alias for it.",
        },
        Example::Choice => ExampleInfo {
            name: "choice",
            source: r#"
choice bool
    | True
    | False

bool-order \:bool:a, :bool:b >
    { a a, b b }
    | { a :bool:False, b :bool:True } > :order:Less
    | { a :bool:True, b :bool:True } > :order:Equal
    | { a :bool:False, b :bool:False } > :order:Equal
    | { a :bool:True, b :bool:False } > :order:Greater

choice lily-type
    | Variable str
    | Construct { name str, arguments vec lily-type }
    | Record vec { name str, value lily-type }
    | Function { inputs vec lily-type, output lily-type }

lily-type-to-record \:lily-type:type >
    type
    | :lily-type:Record :vec { name str, value lily-type }:fields >
        :opt (vec { name str, value lily-type }):Present fields
    | :lily-type:_ >
        :opt (vec { name str, value lily-type }):Absent

choice input-number
    | Int int
    | Dec dec

str-to-input-number \:str:str >
    str-to-int str
    | :opt int:Present :int:int >
        :opt input-number:Present :input-number:Int int
    | :opt int:Absent >
    str-to-dec str
    | :opt dec:Present :dec:dec >
        :opt input-number:Present :input-number:Dec dec
    | :opt dec:Absent >
    :opt input-number:Absent
"#,
            explainer: "Some info can come in multiple shapes (variants).
For example there could be an error or a value, nothing or something, different state per page etc.
To construct/pattern match on a variant (which needs to be uppercase), an explicit type is required.
Core choice types are #order, #opt, #go-on-or-exit. Go take a peek, they are quite useful :)
To add a new choice: choice, type name, space-separated variables, then |, Variant-name, optionally an associated value type, for each variant.
In other languages, this is typically done with object hierarchies or a kind enum + union of value types.",
        },
        Example::Vec => ExampleInfo {
            name: "vector",
            source: r#"
all-elements-known
    [ 3, 2, 1 ]

empty-requires-a-type
    :vec unt:[]

initialized-by-index
    vec-by-index-for-length 10 (\:unt:index > unt-add index 10)

sorted-descending
    vec-sort initialized-by-index (\:unt:a, :unt:b > unt-order b a)

unts-sum \:vec unt:vec >
    vec-walk-from vec
        0
        (\:unt:sum-so-far, :unt:element >
            :go-on-or-exit unt unt:Go-on
                unt-add sum-so-far element
        )
    | :go-on-or-exit unt unt:Go-on :unt:result > result
    | :go-on-or-exit unt unt:Exit :unt:result > result
"#,
            explainer: "a vec hold a variable amount of elements of the same element type.
vec-walk-from goes through all elements and summarizes.
It also has the ability to exit early but we don't use that here.",
        },
        Example::Comment => ExampleInfo {
            name: "comment",
            source: r#"
# variable documentation.
# All comments can span
# multiple lines.
in-front-of-any-declaration
    [ 3, 2, 1 ]

# type documentation
type string = str

# choice documentation
choice length
    | Meters
        # in front of any type
        dec

order-by
    \ # in front of any pattern
      :\A > Key:to-key
    , :\Key, Key > order:key-order
    , :A:a
    , :A:b
    >
    key-order
        (to-key a)
        (# in front of any expression
         to-key b
        )
"#,
            explainer: "vec-walk-from goes through all elements and summarizes.
It also has the ability to exit early but we don't use that here.",
        },
        Example::Extras => ExampleInfo {
            name: "extras",
            source: r#"
local-variables-and-rebinding
    = local-variable 0
    = local-variable^
        unt-add local-variable 1
    local-variable
"#,
            explainer: "2 nicities you may not need
- local variable declarations: =, name, expression
- shadow a local variable name: attach ^ after a pattern or local variable declaration name

None are strictly necessary but are probably stylistically.",
        }
    }
}

// //

fn yew_text(content: &'static str) -> yew::Html {
    yew::Html::VText(yew::virtual_dom::VText {
        text: yew::AttrValue::Static(content),
    })
}
fn yew_text_dynamic(content: impl ToString) -> yew::Html {
    yew::Html::VText(yew::virtual_dom::VText::from(content))
}
fn yew_link_to(resource: &str, name: &str) -> yew::Html {
    yew_element("a", [("href", resource.into())], [yew_text_dynamic(name)])
}
fn yew_linebreak() -> yew::Html {
    yew_element("br", [], [])
}
fn yew_element(
    tag: &'static str,
    modifiers: impl IntoIterator<Item = (&'static str, yew::AttrValue)>,
    subs: impl IntoIterator<Item = yew::Html>,
) -> yew::Html {
    let mut yew_element = yew::virtual_dom::VTag::new(tag);
    for (modifier_key, modifier_value) in modifiers {
        yew_element.add_attribute(modifier_key, modifier_value);
    }
    yew_element.add_children(subs);
    yew::Html::VTag(std::rc::Rc::new(yew_element))
}
fn yew_listener(
    kind: yew::virtual_dom::ListenerKind,
    handle: impl Fn(web_sys::Event),
) -> impl yew::virtual_dom::Listener {
    YewGenericListener {
        kind: kind,
        handle: handle,
    }
}
struct YewGenericListener<Handle> {
    kind: yew::virtual_dom::ListenerKind,
    handle: Handle,
}
impl<Handle: Fn(web_sys::Event)> yew::virtual_dom::Listener for YewGenericListener<Handle> {
    fn kind(&self) -> yew::virtual_dom::ListenerKind {
        self.kind.clone()
    }
    fn handle(&self, event: web_sys::Event) {
        (self.handle)(event)
    }
    fn passive(&self) -> bool {
        true
    }
}

// //

fn utf16_offset_to_utf8_in(source: &str, utf16_offset: usize) -> usize {
    let mut utf8_length: usize = 0;
    let mut so_far_length_utf16: usize = 0;
    'traversing_utf16_length: for char in source.chars() {
        if so_far_length_utf16 >= utf16_offset {
            break 'traversing_utf16_length;
        }
        utf8_length += char.len_utf8();
        so_far_length_utf16 += char.len_utf16();
    }
    utf8_length
    // below does not work for string containing 2-part UTF-16 characters
    // source
    //     .encode_utf16()
    //     .take(utf16_offset)
    //     .map(|utf16_char| {
    //         char::decode_utf16([utf16_char])
    //             .map(|r| r.map(char::len_utf8).unwrap_or(0))
    //             .sum::<usize>()
    //     })
    //     .sum()
}
