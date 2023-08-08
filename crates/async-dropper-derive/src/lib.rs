use proc_macro2::TokenStream;
use syn::{DataEnum, DataStruct, DataUnion, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(AsyncDrop)]
pub fn derive_async_drop(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse2::<DeriveInput>(items.into()) {
        Ok(derive_input) => proc_macro2::TokenStream::from_iter(
            [gen_preamble(&derive_input), gen_impl(&derive_input).into()].into_iter(),
        )
        .into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn make_shared_default_name(ident: &proc_macro2::Ident) -> proc_macro2::Ident {
    quote::format_ident!("_shared_default_{}", ident)
}

/// Default implementation of deriving async drop that does nothing
/// you're expected to use either the 'tokio' feature or 'async-std'
fn gen_preamble(di: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &di.ident;
    let shared_default_name = make_shared_default_name(ident);

    // Retrieve the struct data fields from the derive input
    let mut df_setters: Vec<TokenStream> = Vec::new();
    match &di.data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            if let Fields::Unit = fields {
                df_setters.push(
                    syn::Error::new(ident.span(), "unit sturcts cannot be async dropped")
                        .to_compile_error(),
                );
            }
            for f in fields.iter() {
                df_setters.push(f.ident.as_ref().map_or_else(
                    || {
                        syn::parse_str(
                            format!("self.{} = Default::default()", df_setters.len()).as_str(),
                        )
                        .unwrap_or_else(|_| {
                            syn::Error::new(
                                ident.span(),
                                "failed to generate default setter for field",
                            )
                            .to_compile_error()
                        })
                    },
                    |id| quote::quote! { self.#id = Default::default(); },
                ));
            }
        }
        syn::Data::Enum(DataEnum { variants, .. }) => {
            for v in variants.iter() {
                for vf in v.fields.iter() {
                    df_setters.push(vf.ident.as_ref().map_or_else(
                        || {
                            syn::parse_str(
                                format!("self.{} = Default::default()", df_setters.len()).as_str(),
                            )
                            .unwrap_or_else(|_| {
                                syn::Error::new(
                                    ident.span(),
                                    "failed to generate default setter for field",
                                )
                                .to_compile_error()
                            })
                        },
                        |id| quote::quote! { self.#id = Default::default(); },
                    ))
                }
            }
        }
        syn::Data::Union(DataUnion {
            fields: FieldsNamed { named, .. },
            ..
        }) => {
            for f in named.iter() {
                if let Some(id) = &f.ident {
                    df_setters.push(quote::quote! { self.#id = Default::default(); });
                }
            }
        }
    };

    quote::quote!(
        #[derive(Debug)]
        pub enum AsyncDropError {
            UnexpectedError(Box<dyn std::error::Error>),
            Timeout,
        }

        /// What to do when a drop fails
        #[derive(Debug, PartialEq, Eq)]
        pub enum DropFailAction {
            // Ignore the failed drop
            Continue,
            // Elevate the drop failure to a full on panic
            Panic,
        }

        /// Types that can reset themselves to T::default()
        trait ResetDefault {
            fn reset_to_default(&mut self);
        }

        /// Automatically generated implementation of reset to default for #ident
        #[automatically_derived]
        impl ResetDefault for #ident {
            fn reset_to_default(&mut self) {
                #(
                    #df_setters;
                )*
            }
        }

        /// The operative trait that enables AsyncDrop functionality.
        /// Normally, implementing only async_drop(&mut self) and reset(&mut self) is necessary.
        #[async_trait]
        trait AsyncDrop: Default + PartialEq + Eq + ResetDefault {
            /// Operative drop that does async operations, returning
            async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
                Ok(())
            }

            /// A way to reset the object (set all it's internal members to their default).
            /// This method is used after a successful AsyncDrop, to ensure that future drops do not
            /// perform async_drop behavior twice.
            fn reset(&mut self) {
                self.reset_to_default();
            }

            /// Timeout for drop operation, meant to be overriden if needed
            fn drop_timeout(&self) -> Duration {
                Duration::from_secs(3)
            }

            /// What to do what a drop fails
            fn drop_fail_action(&self) -> DropFailAction {
                DropFailAction::Continue
            }
        }


        /// Utility function unique to #ident which retrieves a shared mutable single default instance of it
        /// that single default instance is compared to other instances and indicates whether async drop
        /// should be called
        #[allow(non_snake_case)]
        fn #shared_default_name() -> &'static std::sync::Mutex<#ident> {
            #[allow(non_upper_case_globals)]
            static #shared_default_name: std::sync::OnceLock<std::sync::Mutex<#ident>> = std::sync::OnceLock::new();
            #shared_default_name.get_or_init(|| std::sync::Mutex::new(#ident::default()))
        }

    )
    .into()
}

#[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
fn gen_impl(_: &DeriveInput) -> proc_macro::TokenStream {
    compile_error!(
        "either 'async-std' or 'tokio' features must be enabled for the async-dropper crate"
    );
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
fn gen_impl(_: &DeriveInput) -> proc_macro::TokenStream {
    compile_error!(
        "both 'async-std' and 'tokio' features must not be enabled for the async-dropper crate"
    )
}

/// Tokio implementation of AsyncDrop
#[cfg(all(feature = "tokio", not(feature = "async-std")))]
fn gen_impl(DeriveInput { ident, .. }: &DeriveInput) -> proc_macro2::TokenStream {
    let shared_default_name = make_shared_default_name(ident);
    quote::quote!(
        #[automatically_derived]
        #[async_trait]
        impl Drop for #ident {
            fn drop(&mut self) {
                // We consider a self that is completley equivalent to it's default version to be dropped
                let thing = #shared_default_name();
                if *thing.lock().unwrap() == *self {
                    return;
                }

                // Ensure that the default_version is manually dropped
                let mut original = std::mem::take(self);

                // Spawn a task to do the drop
                let task = ::tokio::spawn(async move {
                    let drop_fail_action = original.drop_fail_action();
                    match ::tokio::time::timeout(
                        original.drop_timeout(),
                        original.async_drop(),
                    ).await {
                        Err(e) => {
                            match drop_fail_action {
                                DropFailAction::Continue => original,
                                DropFailAction::Panic => {
                                    panic!("async drop failed: {e}");
                                }
                            }
                        },
                        Ok(_) => original,
                    }
                });

                // Perform a synchronous wait
                let mut original = ::tokio::task::block_in_place(|| ::tokio::runtime::Handle::current().block_on(task).unwrap());

                // After the async wait, we must reset all fields to the default (so future checks will fail)
                original.reset();
                if *thing.lock().unwrap() != original {
                    panic!("after calling AsyncDrop::reset(), the object does *not* equal T::default()");
                }
            }
        }
    )
    .into()
}

/// async-std  implementation of AsyncDrop
#[cfg(all(feature = "async-std", not(feature = "tokio")))]
fn gen_impl(DeriveInput { ident, .. }: &DeriveInput) -> proc_macro2::TokenStream {
    let shared_default_name = make_shared_default_name(ident);
    quote::quote!(
        #[automatically_derived]
        #[async_trait]
        impl Drop for #ident {
            fn drop(&mut self) {
                // We consider a self that is completley equivalent to it's default version to be dropped
                let thing = #shared_default_name();
                if *thing.lock().unwrap() == *self {
                    return;
                }

                // Swap out the existing with a completely default
                let mut original = std::mem::take(self);

                // Spawn a task to do the drop
                let task = ::async_std::task::spawn(async move {
                    let drop_fail_action = original.drop_fail_action();
                    match ::async_std::future::timeout(
                        original.drop_timeout(),
                        original.async_drop(),
                    ).await {
                        Err(e) => {
                            match drop_fail_action {
                                DropFailAction::Continue => original,
                                DropFailAction::Panic => {
                                    panic!("async drop failed: {e}");
                                }
                            }
                        },
                        Ok(_) => original,
                    }
                });

                // Perform synchronous wait
                let mut original = ::futures::executor::block_on(task);

                // Reset the task to ensure it won't trigger async drop behavior again
                original.reset();
                if *thing.lock().unwrap() != original {
                    panic!("after calling AsyncDrop::reset(), the object does *not* equal T::default()");
                }
            }
        }
    )
    .into()
}
