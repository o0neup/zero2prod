use actix_web::web::Form;

use crate::routes::SubscriptionFormData;

use super::{subscriber_email::SubscriberEmail, subscriber_name::SubscriberName};

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<Form<SubscriptionFormData>> for NewSubscriber {
    type Error = String;

    fn try_from(value: Form<SubscriptionFormData>) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.0.name)?;
        let email = SubscriberEmail::parse(value.0.email)?;

        Ok(Self { email, name })
    }
}
