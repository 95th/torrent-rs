use std::fmt;

macro_rules! new_ty {
    ($ty: ident; $size: expr) => {
        #[derive(Clone)]
        pub struct $ty([u8; $size]);

        impl Default for $ty {
            fn default() -> Self {
                Self([0; $size])
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        impl std::ops::Deref for $ty {
            type Target = [u8; $size];
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

new_ty!(PublicKey; 32);
new_ty!(SecretKey; 64);
new_ty!(Signature; 64);

pub type SequenceNumber = i64;
