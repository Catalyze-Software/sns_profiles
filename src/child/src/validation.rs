use ic_scalable_canister::ic_scalable_misc::{
    enums::{api_error_type::ApiError, validation_type::ValidationType},
    helpers::validation_helper::Validator,
    models::validation_models::ValidateField,
};

use shared::profile_models::{PostProfile, UpdateProfile};

pub fn validate_post_profile(post_profile: PostProfile) -> Result<(), ApiError> {
    let validator_fields = vec![
        ValidateField(
            ValidationType::StringLength(post_profile.username, 3, 64),
            "username".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(post_profile.display_name, 3, 64),
            "display_name".to_string(),
        ),
    ];

    Validator(validator_fields).validate()
}

pub fn validate_update_profile(update_profile: UpdateProfile) -> Result<(), ApiError> {
    let mut validator_fields = vec![
        ValidateField(
            ValidationType::StringLength(update_profile.display_name, 3, 32),
            "display_name".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_profile.about, 0, 1000),
            "about".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_profile.city, 0, 64),
            "city".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_profile.country, 0, 64),
            "country".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_profile.website, 0, 200),
            "website".to_string(),
        ),
        ValidateField(
            ValidationType::Count(update_profile.skills.len(), 0, 50),
            "skills".to_string(),
        ),
        ValidateField(
            ValidationType::Count(update_profile.interests.len(), 0, 50),
            "interests".to_string(),
        ),
        ValidateField(
            ValidationType::Count(update_profile.causes.len(), 0, 50),
            "causes".to_string(),
        ),
    ];

    match update_profile.email {
        None => {}
        Some(_email) => validator_fields.push(ValidateField(
            ValidationType::Email(_email),
            "email".to_string(),
        )),
    }

    Validator(validator_fields).validate()
}
