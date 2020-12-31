use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(State)]
pub fn derive_state_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let ident_args =
        proc_macro2::Ident::new(&format!("{}Args", ident), proc_macro2::Span::call_site());

    let data = match &input.data {
        syn::Data::Enum(d) => d,
        _ => return TokenStream::from(quote!(compile_error! {"not an enum"})),
    };

    for variant in &data.variants {
        match &variant.fields {
            syn::Fields::Unit => (),
            _ => return quote!(compile_error! {"only unit variants are supported"}).into(),
        }

        if variant.discriminant.is_some() {
            return quote!(compile_error! {"variant discriminants are not supported"}).into();
        }

        if !variant.attrs.is_empty() {
            return quote!(compile_error! {"variant attributes are not supported"}).into();
        }
    }

    let num_variants = data.variants.len();
    let variant_idents1 = data.variants.iter().map(|v| &v.ident);
    let variant_indices1 = 0..data.variants.len();
    let variant_idents2 = variant_idents1.clone();
    let variant_indices2 = variant_indices1.clone();
    let variant_idents3 = variant_idents1.clone();
    let variant_indices3 = variant_indices1.clone();

    let variant_idents_lower1 = data.variants.iter().map(|v| {
        proc_macro2::Ident::new(
            &v.ident.to_string().to_case(Case::Snake),
            proc_macro2::Span::call_site(),
        )
    });
    let variant_idents_lower2 = variant_idents_lower1.clone();

    let variant_idents_lower3 = variant_idents_lower1.clone();
    let variant_indices_lower3 = variant_indices1.clone();

    TokenStream::from(quote! {
        #[automatically_derived]
        impl<S, A> std::ops::Index<#ident> for ndarray::ArrayBase<S, ndarray::Ix1>
        where
            S: ndarray::Data<Elem = A>,
        {
            type Output = A;

            fn index(&self, idx: #ident) -> &A {
                match idx {
                    #(
                        #ident::#variant_idents1 => &self[#variant_indices1],
                    )*
                }
            }
        }

        #[automatically_derived]
        impl<S, A> std::ops::IndexMut<#ident> for ndarray::ArrayBase<S, ndarray::Ix1>
        where
            S: ndarray::DataMut<Elem = A>,
        {
            fn index_mut(&mut self, idx: #ident) -> &mut A {
                match idx {
                    #(
                        #ident::#variant_idents2 => &mut self[#variant_indices2],
                    )*
                }
            }
        }


        #[automatically_derived]
        pub struct #ident_args<A> {
            #(
                pub #variant_idents_lower1: A,
            )*
        }

        #[automatically_derived]
        impl<A> From<#ident_args<A>> for ::ndarray::Array1<A>
        {
            fn from(args: #ident_args<A>) -> Self {
                ::ndarray::array![
                    #(
                        args.#variant_idents_lower2,
                    )*
                ]
            }
        }

        #[automatically_derived]
        impl<S, A> ::sensoreval_utils::AssignState<#ident_args<A>>
            for ::ndarray::ArrayBase<S, ndarray::Ix1>
        where
            S: ndarray::DataMut<Elem = A>,
        {
            fn assign_state(&mut self, args: #ident_args<A>) {
                #(
                    self[#variant_indices_lower3] = args.#variant_idents_lower3;
                )*
            }
        }

        #[automatically_derived]
        impl ::sensoreval_utils::StateUtils for #ident {
            #[inline]
            fn len() -> usize {
                #num_variants
            }

            fn id(&self) -> usize {
                match self {
                    #(
                        #ident::#variant_idents3 => #variant_indices3,
                    )*
                }
            }
        }
    })
}
