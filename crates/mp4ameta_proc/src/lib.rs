use proc_macro::TokenStream;

#[proc_macro]
pub fn individual_string_value_accessor(input: TokenStream) -> TokenStream {
    let str = input.to_string();

    let mut tokens_strings = str.split(',');

    let function_ident = tokens_strings.next().expect("Expected function ident").trim_start().replace("\"", "");
    let name = function_ident.replace('_', " ");

    let mut name_chars = name.chars();
    let headline = format!("{}{}", name_chars.next().unwrap().to_uppercase(), name_chars.collect::<String>());

    let atom_ident = format!("atom::{}", function_ident.to_uppercase());

    let atom_ident_string = tokens_strings.next().expect("Expected atom ident string").trim_start().replace("\"", "");

    format!("
/// ### {0}
impl Tag {{
    /// Returns the {1} ({2}).
    pub fn {3}(&self) -> Option<&str> {{
        self.string({4}).next()
    }}

    /// Sets the {1} ({2}).
    pub fn set_{3}(&mut self, {3}: impl Into<String>) {{
        self.set_data({4}, Data::Utf8({3}.into()));
    }}

    /// Removes the {1} ({2}).
    pub fn remove_{3}(&mut self) {{
        self.remove_data({4});
    }}
}}
    ",
            headline,
            name,
            atom_ident_string,
            function_ident,
            atom_ident,
    ).parse().unwrap()
}

#[proc_macro]
pub fn multiple_string_values_accessor(input: TokenStream) -> TokenStream {
    let str = input.to_string();

    let mut tokens_strings = str.split(',');

    let function_ident = tokens_strings.next().expect("Expected function ident").trim_start().replace("\"", "");
    let mut function_ident_plural = function_ident.clone();
    if function_ident_plural.ends_with("y") {
        let _ = function_ident_plural.split_off(function_ident_plural.len());
        function_ident_plural.push_str("ies");
    } else {
        function_ident_plural.push_str("s");
    };

    let name = function_ident.replace('_', " ");
    let name_plural = function_ident_plural.replace('_', " ");

    let mut name_chars = name.chars();
    let headline = format!("{}{}", name_chars.next().unwrap().to_uppercase(), name_chars.collect::<String>());

    let atom_ident = format!("atom::{}", function_ident.to_uppercase());

    let atom_ident_string = tokens_strings.next().expect("Expected atom ident string").trim_start().replace("\"", "");

    format!("
/// ### {0}
impl Tag {{
    /// Returns all {2} ({3}).
    pub fn {5}(&self) -> impl Iterator<Item=&str> {{
        self.string({6})
    }}

    /// Returns the first {1} ({3}).
    pub fn {4}(&self) -> Option<&str> {{
        self.{5}().next()
    }}

    /// Sets the {1} ({3}). This will remove all other {2}.
    pub fn set_{4}(&mut self, {4}: impl Into<String>) {{
        self.set_data({6}, Data::Utf8({4}.into()));
    }}

    /// Adds an {1} ({3}).
    pub fn add_{4}(&mut self, {4}: impl Into<String>) {{
        self.add_data({6}, Data::Utf8({4}.into()));
    }}

    /// Removes all {2} ({3}).
    pub fn remove_{5}(&mut self) {{
        self.remove_data({6});
    }}
}}
    ",
            headline,
            name,
            name_plural,
            atom_ident_string,
            function_ident,
            function_ident_plural,
            atom_ident,
    ).parse().unwrap()
}
