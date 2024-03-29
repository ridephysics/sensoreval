use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(State)]
pub fn derive_state_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let ident_args =
        proc_macro2::Ident::new(&format!("{ident}Args"), proc_macro2::Span::call_site());

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
        impl<S, A> ::std::ops::Index<#ident> for ::ndarray::ArrayBase<S, ::ndarray::Ix1>
        where
            S: ::ndarray::Data<Elem = A>,
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
        impl<S, A> ::std::ops::IndexMut<#ident> for ::ndarray::ArrayBase<S, ::ndarray::Ix1>
        where
            S: ::ndarray::DataMut<Elem = A>,
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
        impl<A> ::std::convert::From<#ident_args<A>> for ::ndarray::Array1<A>
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

#[derive(Debug)]
enum Attribute {
    FnStruct(String),
    Lifetimes(usize),
    Angle,
}

struct AttributeIter<I> {
    inner: I,
    nested_iter: Option<syn::punctuated::IntoIter<syn::NestedMeta>>,
}

impl<'a, I> AttributeIter<I>
where
    I: Iterator<Item = &'a syn::Attribute>,
{
    fn new(inner: I) -> Self {
        Self {
            inner,
            nested_iter: None,
        }
    }

    fn process_nested(&self, nestedmeta: &syn::NestedMeta) -> Attribute {
        match nestedmeta {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => {
                    if path.is_ident("angle") {
                        Attribute::Angle
                    } else {
                        abort!(path, "unsupported path identifier");
                    }
                }
                syn::Meta::NameValue(nv) => {
                    if nv.path.is_ident("fnstruct") {
                        match &nv.lit {
                            syn::Lit::Str(s) => Attribute::FnStruct(s.value()),
                            _ => abort!(nv.path, "unsupported value for fnstruct"),
                        }
                    } else if nv.path.is_ident("lifetimes") {
                        match &nv.lit {
                            syn::Lit::Int(i) => Attribute::Lifetimes(
                                i.base10_parse().expect_or_abort("invalid number"),
                            ),
                            _ => abort!(nv.path, "unsupported value for lifetimes"),
                        }
                    } else {
                        abort!(nv.path, "unsupported nv path ident");
                    }
                }
                _ => abort!(meta, "unsupported meta type"),
            },
            _ => abort!(nestedmeta, "unsupported nestedmeta type"),
        }
    }

    fn next_nested(&mut self) -> Option<Attribute> {
        if let Some(nested_iter) = &mut self.nested_iter {
            let next = nested_iter.next().map(|o| self.process_nested(&o));
            if next.is_none() {
                self.nested_iter = None;
            }
            next
        } else {
            None
        }
    }
}

impl<'a, I> Iterator for AttributeIter<I>
where
    I: Iterator<Item = &'a syn::Attribute>,
{
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_nested() {
            None => (),
            other => return other,
        }

        let attr = self.inner.find(|attr| attr.path.is_ident("state"))?;

        self.nested_iter = Some(
            match attr.parse_meta() {
                Ok(syn::Meta::List(list)) => list.nested,
                _ => abort!(attr, "invalid state attribute format"),
            }
            .into_iter(),
        );

        self.next_nested()
    }
}

fn gen_angle_map(data: &syn::DataEnum) -> Vec<bool> {
    let mut is_angle_map = Vec::new();
    for variant in &data.variants {
        let mut is_angle = false;
        for attr in AttributeIter::new(variant.attrs.iter()) {
            if let Attribute::Angle = attr {
                is_angle = true;
                break;
            }
        }

        is_angle_map.push(is_angle);
    }
    is_angle_map
}

fn find_fnstruct(
    input: &DeriveInput,
) -> (
    proc_macro2::Ident,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut fnstruct = None;
    let mut num_lifetimes = 0;
    for attr in AttributeIter::new(input.attrs.iter()) {
        match attr {
            Attribute::FnStruct(s) => fnstruct = Some(s),
            Attribute::Lifetimes(n) => num_lifetimes = n,
            _ => (),
        }
    }
    let fnstruct = match fnstruct {
        Some(v) => proc_macro2::Ident::new(&v, proc_macro2::Span::call_site()),
        None => abort!(input, "can't find fnstruct"),
    };

    let lifetimes_iter = (0..num_lifetimes)
        .map(|i| syn::Lifetime::new(&format!("'lt_{i}"), proc_macro2::Span::call_site()));
    let generics = quote! {
        #(#lifetimes_iter),*
    };
    let comma = if num_lifetimes > 0 {
        quote! {,}
    } else {
        quote! {}
    };

    (fnstruct, generics, comma)
}

#[proc_macro_derive(KalmanMath, attributes(state))]
#[proc_macro_error]
pub fn derive_kalman_math(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match &input.data {
        syn::Data::Enum(d) => d,
        _ => abort!(input, "not an enum"),
    };
    let (fnstruct, generics, generics_comma) = find_fnstruct(&input);
    let is_angle_map = gen_angle_map(data);

    let normalize_impls = is_angle_map.iter().enumerate().map(|(i, is_angle)| {
        if *is_angle {
            quote! {x[#i] = math::normalize_angle(x[#i]);}
        } else {
            quote! {}
        }
    });

    TokenStream::from(quote! {
        impl<#generics #generics_comma A> ::kalman::Normalize<A> for #fnstruct<#generics>
        where
            A: ::num_traits::Float + ::num_traits::FloatConst + ::std::ops::SubAssign,
        {
            fn normalize(&self, mut x: ::ndarray::Array1<A>) -> ::ndarray::Array1<A> {
                #(#normalize_impls)*
                x
            }
        }

        impl<#generics #generics_comma A> ::kalman::Subtract<A> for #fnstruct<#generics>
        where
            A: ::num_traits::Float + ::num_traits::FloatConst + ::std::ops::SubAssign,
        {
            fn subtract<Sa, Sb>(
                &self,
                a: &::ndarray::ArrayBase<Sa, ::ndarray::Ix1>,
                b: &::ndarray::ArrayBase<Sb, ::ndarray::Ix1>,
            ) -> ::ndarray::Array1<A>
            where
                Sa: ::ndarray::Data<Elem = A>,
                Sb: ::ndarray::Data<Elem = A>,
            {
                use ::kalman::Normalize;
                self.normalize(a - b)
            }
        }

        impl<#generics #generics_comma A> ::kalman::Add<A> for #fnstruct<#generics>
        where
            A: ::num_traits::Float + ::num_traits::FloatConst + ::std::ops::SubAssign,
        {
            fn add<Sa, Sb>(
                &self,
                a: &::ndarray::ArrayBase<Sa, ::ndarray::Ix1>,
                b: &::ndarray::ArrayBase<Sb, ::ndarray::Ix1>,
            ) -> ::ndarray::Array1<A>
            where
                Sa: ::ndarray::Data<Elem = A>,
                Sb: ::ndarray::Data<Elem = A>,
            {
                use ::kalman::Normalize;
                self.normalize(a + b)
            }
        }
    })
}

#[proc_macro_derive(UKFMath, attributes(state))]
#[proc_macro_error]
pub fn derive_ukf_math(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match &input.data {
        syn::Data::Enum(d) => d,
        _ => abort!(input, "not an enum"),
    };
    let (fnstruct, generics, generics_comma) = find_fnstruct(&input);
    let is_angle_map = gen_angle_map(data);

    let mean_decls = is_angle_map.iter().enumerate().map(|(i, is_angle)| {
        let sum = proc_macro2::Ident::new(&format!("sum_{i}"), proc_macro2::Span::call_site());
        if *is_angle {
            quote! {let mut #sum = math::SinCosSum::default();}
        } else {
            quote! {}
        }
    });
    let mean_impls = is_angle_map.iter().enumerate().map(|(i, is_angle)| {
        let sum = proc_macro2::Ident::new(&format!("sum_{i}"), proc_macro2::Span::call_site());
        if *is_angle {
            quote! {
                assert!(sp[#i] >= -A::PI() && sp[#i] <= A::PI());
                #sum.add(sp[#i], *w);
            }
        } else {
            quote! {}
        }
    });
    let mean_assignments = is_angle_map.iter().enumerate().map(|(i, is_angle)| {
        let sum = proc_macro2::Ident::new(&format!("sum_{i}"), proc_macro2::Span::call_site());
        if *is_angle {
            quote! {ret[#i] = #sum.avg();}
        } else {
            quote! {}
        }
    });

    TokenStream::from(quote! {
        impl<#generics #generics_comma A> ::kalman::ukf::Mean<A> for #fnstruct<#generics>
        where
            A: 'static + ::num_traits::Float + ::num_traits::FloatConst + ::std::ops::AddAssign + ::std::default::Default,
        {
            #[allow(non_snake_case)]
            fn mean<Ss, Swm>(
                &self,
                sigmas: &::ndarray::ArrayBase<Ss, ::ndarray::Ix2>,
                Wm: &::ndarray::ArrayBase<Swm, ::ndarray::Ix1>,
            ) -> ::ndarray::Array1<A>
            where
                Ss: ::ndarray::Data<Elem = A>,
                Swm: ::ndarray::Data<Elem = A>,
            {
                let mut ret = Wm.dot(sigmas);

                #(#mean_decls)*

                ::ndarray::azip!((sp in sigmas.genrows(), w in Wm) {
                    #(#mean_impls)*
                });

                #(#mean_assignments)*

                ret
            }
        }

    })
}
