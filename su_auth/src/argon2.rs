use argon2::{PasswordHasher as _, PasswordVerifier as _};

use crate::password::{DUMMY_USER_PASSWORD, Password, PasswordHash};

impl PasswordHash {
    pub fn argon2_verify(
        ctx: &argon2::Argon2<'static>,
        password: &Password,
        password_hash: &PasswordHash,
    ) -> Result<(), argon2::password_hash::Error> {
        let pbs = Into::<&[u8]>::into(password);
        let hash = argon2::PasswordHash::new(password_hash.as_ref())?;
        ctx.verify_password(pbs, &hash)
    }

    #[must_use]
    pub fn argon2_dummy_hash(ctx: &argon2::Argon2) -> Self {
        Self(
            ctx.hash_password(DUMMY_USER_PASSWORD.as_bytes())
                .expect(":D")
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::password::PasswordHash;
    use argon2::{Algorithm, Argon2, Params, Version};

    #[test]
    fn valid_dummy_hash() {
        let ctx = argon2::Argon2::new(Algorithm::default(), Version::default(), Params::default());
        let _ = PasswordHash::argon2_dummy_hash(&ctx);
    }
}
