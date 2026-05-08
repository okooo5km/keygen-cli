#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Admin,
    Environment,
    Product,
    User,
    License,
    Activation,
}

impl TokenKind {
    pub fn from_jsonapi_type(t: &str) -> Option<Self> {
        Some(match t {
            "admin-tokens" | "admins" => Self::Admin,
            "environment-tokens" | "environments" => Self::Environment,
            "product-tokens" | "products" => Self::Product,
            "user-tokens" | "users" => Self::User,
            "license-tokens" | "licenses" => Self::License,
            "activation-tokens" => Self::Activation,
            _ => return None,
        })
    }
}
