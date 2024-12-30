use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ReplaceInfNan)]
pub fn replace_inf_nan_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_replace_inf_nan(&ast).into()
}

fn is_f64_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "f64";
        }
    }
    false
}

fn impl_replace_inf_nan(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("ReplaceInfNan can only be derived for structs"),
    };

    let replace_fields = match fields {
        Fields::Named(fields) => fields.named.iter().filter_map(|field| {
            if is_f64_type(&field.ty) {
                let ident = &field.ident;
                Some(quote! {
                    self.#ident.replace_inf_nan();
                })
            } else {
                None
            }
        }),
        _ => panic!("ReplaceInfNan only supports named fields"),
    };

    quote! {
        impl ReplaceInfNan for #name {
            fn replace_inf_nan(&mut self) {
                #(#replace_fields)*
            }
        }
    }
}
