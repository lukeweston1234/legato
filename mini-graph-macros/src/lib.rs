use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Port)]
pub fn derive_audio_port(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let ident = input.ident;

    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut to_index_blocks = vec![];
    let mut from_index_blocks = vec![];

    let data_enum = match input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return quote! {
                compile_error!("`#[derive(Port)]` can only be used on enums");
            }
            .into();
        }
    };

    for v in &data_enum.variants {
        match &v.fields {
            syn::Fields::Unit => {}
            _ => {
                let span = v.ident.span();
                return syn::Error::new(span, "`#[derive(Port)]` requires unit variants")
                    .to_compile_error()
                    .into();
            }
        }
    }

    for (i, variant) in data_enum.variants.iter().enumerate() {
        let v = &variant.ident;

        from_index_blocks.push(quote! { Self::#v => #i, });
        to_index_blocks.push(quote! {#i => Ok(Self::#v),});
    }

    let expanded = quote! {
        impl #impl_generics Port for #ident #ty_generics #where_clause {            
            #[inline(always)]
            fn into_index(&self) -> usize {
                match self {
                    #(#from_index_blocks)*
                }
            }
            #[inline(always)]
            fn from_index(index: usize) -> Result<Self, PortError> {
                match index {
                    #(#to_index_blocks)*
                    _ => Err(PortError::InvalidPort),
                }
            }
        }
    };

    TokenStream::from(expanded)
}