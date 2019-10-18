use crate::{
    api::{
        result_audit_diff, result_audit_err, result_audit_subject, validate, ValidateRequest,
        ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Core, CoreResult, Driver, Key, User, UserCreate,
    UserListFilter, UserListQuery, UserPasswordMeta, UserRead, UserUpdate,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct UserListRequest {
    #[builder(default = "None")]
    gt: Option<Uuid>,
    #[builder(default = "None")]
    lt: Option<Uuid>,
    #[builder(default = "None")]
    #[validate(custom = "validate::limit")]
    limit: Option<i64>,
    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,
    #[builder(default = "None")]
    #[validate(email)]
    email_eq: Option<String>,
}

impl ValidateRequest<UserListRequest> for UserListRequest {}
impl ValidateRequestQuery<UserListRequest> for UserListRequest {}

impl UserListRequest {
    pub fn into_query_filter(self) -> (UserListQuery, UserListFilter) {
        let limit = self.limit.unwrap_or_else(Core::default_limit);
        let query = match (self.gt, self.lt) {
            (Some(gt), Some(_lt)) => UserListQuery::IdGt(gt, limit),
            (Some(gt), None) => UserListQuery::IdGt(gt, limit),
            (None, Some(lt)) => UserListQuery::IdLt(lt, limit),
            (None, None) => UserListQuery::Limit(limit),
        };

        let filter = UserListFilter {
            id: self.id,
            email_eq: self.email_eq,
        };

        (query, filter)
    }

    pub fn from_query_filter(query: UserListQuery, filter: UserListFilter) -> Self {
        match query {
            UserListQuery::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                id: filter.id,
                email_eq: filter.email_eq,
            },
            UserListQuery::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
                id: filter.id,
                email_eq: filter.email_eq,
            },
            UserListQuery::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
                id: filter.id,
                email_eq: filter.email_eq,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserListRequest,
    pub data: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateRequest {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::locale")]
    pub locale: String,
    #[validate(custom = "validate::timezone")]
    pub timezone: String,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
    #[validate(custom = "validate::password")]
    pub password: Option<String>,
}

impl ValidateRequest<UserCreateRequest> for UserCreateRequest {}

impl UserCreateRequest {
    pub fn new<S1, S2, S3, S4>(
        is_enabled: bool,
        name: S1,
        email: S2,
        locale: S3,
        timezone: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            locale: locale.into(),
            timezone: timezone.into(),
            password_allow_reset: None,
            password_require_update: None,
            password: None,
        }
    }

    pub fn with_password<S1: Into<String>>(
        mut self,
        password_allow_reset: bool,
        password_require_update: bool,
        password: S1,
    ) -> Self {
        self.password_allow_reset = Some(password_allow_reset);
        self.password_require_update = Some(password_require_update);
        self.password = Some(password.into());
        self
    }
}

impl From<UserCreateRequest> for UserCreate {
    fn from(request: UserCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            email: request.email,
            locale: request.locale,
            timezone: request.timezone,
            password_allow_reset: request.password_allow_reset.unwrap_or(false),
            password_require_update: request.password_require_update.unwrap_or(false),
            password_hash: request.password,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateResponse {
    pub meta: UserPasswordMeta,
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReadResponse {
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
    #[validate(custom = "validate::locale")]
    pub locale: Option<String>,
    #[validate(custom = "validate::timezone")]
    pub timezone: Option<String>,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
}

impl ValidateRequest<UserUpdateRequest> for UserUpdateRequest {}

pub fn user_list(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: UserListRequest,
) -> CoreResult<UserListResponse> {
    UserListRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserList);
    let (query, filter) = request.into_query_filter();

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| User::list(driver, service.as_ref(), &query, &filter));
    result_audit_err(driver, &audit, res).map(|data| UserListResponse {
        meta: UserListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn user_create(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    password_meta: UserPasswordMeta,
    request: UserCreateRequest,
) -> CoreResult<UserCreateResponse> {
    UserCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserCreate);
    let mut create: UserCreate = request.into();

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| User::create(driver, service.as_ref(), &mut create));
    result_audit_subject(driver, &audit, res).map(|data| UserCreateResponse {
        meta: password_meta,
        data,
    })
}

pub fn user_read(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    user_id: Uuid,
) -> CoreResult<UserReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserRead);
    let read = UserRead::Id(user_id);

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| User::read(driver, service.as_ref(), &read));
    result_audit_err(driver, &audit, res).map(|data| UserReadResponse { data })
}

pub fn user_update(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    user_id: Uuid,
    request: UserUpdateRequest,
) -> CoreResult<UserReadResponse> {
    UserUpdateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserUpdate);
    let read = UserRead::Id(user_id);

    let res = Key::authenticate(driver, &mut audit, key_value).and_then(|service| {
        User::read(driver, service.as_ref(), &read).and_then(|previous_user| {
            let update = UserUpdate {
                is_enabled: request.is_enabled,
                name: request.name,
                locale: request.locale,
                timezone: request.timezone,
                password_allow_reset: request.password_allow_reset,
                password_require_update: request.password_require_update,
            };
            User::update(driver, service.as_ref(), user_id, &update)
                .map(|next_user| (previous_user, next_user))
        })
    });
    result_audit_diff(driver, &audit, res).map(|data| UserReadResponse { data })
}

pub fn user_delete(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    user_id: Uuid,
) -> CoreResult<()> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserDelete);
    let read = UserRead::Id(user_id);

    let res = Key::authenticate(driver, &mut audit, key_value).and_then(|service| {
        User::read(driver, service.as_ref(), &read)
            .and_then(|user| User::delete(driver, service.as_ref(), user_id).map(|_| user))
    });
    result_audit_subject(driver, &audit, res).map(|_| ())
}
