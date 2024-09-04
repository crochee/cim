use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput,
    Fields, GenericParam, Generics, Ident, Type,
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
        impl #impl_generics cim::Uid for #name #ty_generics #where_clause {
            fn uid(&self) -> String {
                #build_in
            }
        }
    }
}

// Add a bound `T: cim::Uid` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(cim::Uid));
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

#[proc_macro_derive(Opt)]
pub fn option(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let derive_attrs = input.attrs;

    let Data::Struct(data_struct) = input.data else {
        // 不接受除了结构体之外的类型
        return syn::Error::new(
            ident.span(),
            "opt can only be applied to structs",
        )
        .into_compile_error()
        .into();
    };

    let optional_struct_name = &format!("{}ListOptions", ident);
    let optional_struct_ident = Ident::new(optional_struct_name, ident.span());

    let fields = data_struct
        .fields
        .iter()
        .map(|field| {
            // 对每个字段进行映射
            let field_attrs =
                field.attrs.iter().find(|attr| !attr.path().is_ident("opt"));
            let mut ident = field.ident.clone();
            let ty = &field.ty;

            let mut is_skip = false;

            let attr =
                field.attrs.iter().find(|attr| attr.path().is_ident("opt"));

            if let Some(attr) = attr {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        is_skip = true;
                    } else if meta.path.is_ident("rename") {
                        let renamed_ident =
                            meta.value()?.parse::<syn::Ident>()?;
                        ident = Some(renamed_ident);
                    }
                    Ok(())
                });
            }
            match field_attrs {
                Some(field_attr) => {
                    if is_skip || is_option(ty) {
                        return quote! {#(#field_attr)* #ident: #ty,};
                    }
                    quote! {#(#field_attr)* #ident: Option<#ty>}
                }
                None => {
                    if is_skip || is_option(ty) {
                        return quote! {#ident: #ty};
                    }
                    quote!(#ident: Option<#ty>)
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        #(#derive_attrs)*
        struct #optional_struct_ident {
            #(#fields,)*
        }
    }
    .into()
}

fn is_option(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident == "Option")
        .unwrap_or_default()
}
