use quote::quote;

#[proc_macro_derive(AsyncDrop)]
pub fn derive_async_drop(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    _derive_async_drop(item)
}

/// Tokio implementation of AsyncDrop
#[cfg(feature = "tokio")]
fn _derive_async_drop(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ts = quote!(
        #item

        #[derive(Debug, async_dropper::thiserror)]
        pub enum AsyncDropFailure  {
            UnexpectedError(Box<dyn std::error::Error>)
            Timeout(async_dropper::tokio::time::error::Elapsed)
        }

        /// What to do when a drop fails
        #[derive(Debug, PartialEq, Eq)]
        pub enum DropFailAction {
            // Ignore the failed drop
            Continue,
            // Elevate the drop failure to a full on panic
            Panic,
        }

        #[::async_trait::async_trait]
        trait AsyncDrop: Default {
            /// Operative drop that does async operations, returning
            async fn drop() -> Result<(), AsyncDropFailure> {
                Ok(())
            }

            /// Timeout for drop operation, meant to be overriden if needed
            fn drop_timeout() -> Duration {
                Duration::from_secs()
            }

            /// What to do what a drop fails
            fn drop_fail_action() -> DropFailAction {
                DropFailAction::Continue
            }
        }

        impl<T: AsyncDrop + Default + PartialEq> Drop for #item {
            fn drop() -> {
                AsyncDrop::drop(self).await;

                // We consider a self that is completley equivalent to it's default version to be dropped
                if Self::default() == self {
                    return;
                }

                let mut original = Self::default();
                std::mem::swap(&mut original, self);
                tokio::spawn(async move {
                    let drop_fail_action = original.drop_fail_action();
                    match tokio::timeout(
                        original.drop_timeout(),
                        AsyncDrop::drop(original),
                    ).await {
                        Err(e) => {
                            match drop_fail_action {
                                DropFailAction::Continue => {}
                                DropFailAction::Panic => {
                                    panic!("async drop failed: {e}");
                                }
                            }
                        },
                        Ok(_) => {},
                    }

                });
            }
        }
    );
}

/// async-std  implementation of AsyncDrop
#[cfg(feature = "async-std")]
fn _derive_async_drop(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item
}
