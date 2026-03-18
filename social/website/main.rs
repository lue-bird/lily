#![allow(non_upper_case_globals)]

use lily_compile as lily;
fn main() {
    yew::Renderer::<State>::new().render();
}

struct State {
    text_area_content: String,
}
enum Event {
    TextAreaContentChanged(String),
}
impl yew::Component for State {
    type Message = Event;

    type Properties = ();

    fn create(_: &yew::Context<Self>) -> Self {
        State {
            text_area_content: String::from(hello_world),
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, event: Event) -> bool {
        match event {
            Event::TextAreaContentChanged(new_text_area_content) => {
                self.text_area_content = new_text_area_content;
                true
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> yew::Html {
        yew_element(
            "main",
            [],
            [
                yew_element(
                    "h2",
                    [("style", "max-width: 40rem".into())],
                    [yew_text(
                        "a very simple, explicitly boring, purely functional programming language that compiles to rust: lily",
                    )],
                ),
                yew_link_to("https://codeberg.org/lue-bird/lily", "source code"),
                yew_linebreak(),
                yew_sub_heading("try it"),
                yew_playground(&self.text_area_content, context),
                yew_sub_heading("core declarations"),
                lily_core_declarations_html(),
            ],
        )
    }
}

static hello_world: &str = r#"
your-output
    greet "insert your name here"

greet \:str:name >
    strs-flatten [ "Hii, ", name, " ˖᯽"  ]
"#;
fn yew_playground(text_area_content: &str, context: &yew::Context<State>) -> yew::Html {
    let mut interactive_text_area = yew::virtual_dom::VTag::new("textarea");
    interactive_text_area.add_attribute("autocorrect", "off");
    interactive_text_area.add_attribute("spellcheck", "false");
    interactive_text_area.add_attribute("autofocus", "true");
    interactive_text_area.add_attribute("name", "playground");
    // this is all _very_ annoying
    interactive_text_area.add_attribute(
        "style",
        format!(
            "field-sizing: content;
            height: {}em;
            width: 100%;
            background: none;
            color: transparent;
            border: none;
            line-height: 1.2;
            resize: none;
            overflow: hidden;
            font-family: monospace;
            resize: none;
            caret-color: white;
            position: relative;
            top: 0.335em;
            left: -0.2em",
            (text_area_content.lines().count()
                + if text_area_content.ends_with("\n") {
                    2
                } else {
                    1
                }) as f64
                * 1.2
        ),
    );
    interactive_text_area.add_property("value", hello_world);
    let cursor_offset = hello_world.find("insert your name here").unwrap_or(0);
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
            yew_element("pre", [("style", "font: inherit".into())], [
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
    yew_element(
        "a",
        [
            ("href", format!("#{name}").into()),
            ("id", format!("{name}").into()),
        ],
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
    html.add_attribute("style", "line-height: 1.2; margin-top: 0.5em");

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
        lily::SyntaxHighlightKind::TypeVariable => "rgb(90,150,255)",
        lily::SyntaxHighlightKind::Variant => "rgb(60,255,30)",
        lily::SyntaxHighlightKind::Field => "orange",
        lily::SyntaxHighlightKind::Variable | lily::SyntaxHighlightKind::DeclaredVariable => {
            "yellow"
        }
        lily::SyntaxHighlightKind::Comment => "rgb(140,140,140)",
        lily::SyntaxHighlightKind::String | lily::SyntaxHighlightKind::Number => "rgb(170,120,255)",
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
                    // TODO skip if last line
                    html.add_child(yew_linebreak());
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
