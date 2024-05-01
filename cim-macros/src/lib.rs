use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput,
    Fields, GenericParam, Generics, Ident,
};

#[proc_macro_derive(Uid)]
pub fn derive_uid(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    let expanded = impl_uid(input);
    expanded.into()
}

fn impl_uid(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let build_in = build_body(&input);
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics cim_uid::Uid for #name #ty_generics #where_clause {
            fn uid(&self) -> String {
                #build_in
            }
        }
    }
}
// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(cim_uid::Uid));
        }
    }
    generics
}

fn build_body(input: &DeriveInput) -> TokenStream {
    let name = input.ident.to_string();

    if let Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            for f in &fields.named {
                let field_name = &f.ident;
                if Some(Ident::new("id", f.span())).eq(field_name) {
                    return quote! {
                        format!("{}/{}", #name, &self.#field_name)
                    };
                }
                if Some(Ident::new("name", f.span())).eq(field_name) {
                    return quote! {
                        format!("{}/{}", #name, &self.#field_name)
                    };
                }
            }
        }
    };

    quote! {
        format!("{}",#name)
    }
}
