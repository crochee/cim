use serde::Deserialize;

// Scopes represents additional data requested by the clients about the end user.
#[derive(Debug, Deserialize, Default)]
pub struct Scopes {
    /// The client has requested a refresh token from the server.
    pub offline_access: bool,
    /// The client has requested group information about the end user.
    pub groups: bool,
}

impl From<Vec<String>> for Scopes {
    fn from(v: Vec<String>) -> Self {
        let mut s = Scopes::default();
        for scope in v {
            if scope.eq("offline_access") {
                s.offline_access = true;
            } else if scope.eq("groups") {
                s.groups = true;
            }
        }
        s
    }
}
